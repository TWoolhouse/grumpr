mod library;
use itertools::Itertools;
pub use library::Library;
mod error;
pub use error::{Error, Result};
mod grams;
mod search;
pub use search::query::{QueryAnagram, QuerySearch};
#[cfg(test)]
mod test;
pub use grams::Gram;
use grams::LibGram;
use regex::Regex;
use regex_automata::dfa::Automaton;

use crate::{
    librarian::search::{MultiHeadDFA, Nest},
    trie::Trie,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seed {
    /// The root string
    root: String,
    /// The index of the root in the library
    index: usize,
    /// The number of occurrences of this root in the text
    count: u64,
}

pub struct Librarian<'l> {
    library: &'l Library,
    grams: Vec<LibGram<'l>>,
}

impl<'l> Librarian<'l> {
    /// Returns the number of seeds in the librarian.
    #[must_use]
    pub fn len(&self) -> usize {
        self.grams.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.grams.is_empty()
    }
    /// Returns an iterator over the grams in the librarian.
    pub fn iter(&self) -> Iter<'_, 'l> {
        Iter {
            library: self.library,
            grams: self.grams.iter(),
        }
    }

    /// Returns the seed associated with this root.
    #[must_use]
    pub fn root(&self, root: &str) -> Option<&'l Seed> {
        self.library.seeds.iter().find(|s| s.root == root)
    }
    /// Returns the seed at the given index of the library.
    #[must_use]
    pub fn index(&self, index: usize) -> Option<&'l Seed> {
        self.library.seeds.get(index)
    }

    /// Find seeds matching a regex pattern.
    pub fn search(&self, query: &QuerySearch<'_>) -> Result<Self> {
        let grams = if query.repeats > 0 {
            self.search_deep(query)?
        } else {
            self.search_flat(query)?
        };

        Ok(Self {
            library: self.library,
            grams,
        })
    }
    /// Nearest word search
    // fn nearest(&self, pattern: &str) -> Option<&Seed>; // impl Librarian + '_;
    pub fn nearest(&self) {}

    /// Find annagrams
    pub fn anagrams(&self, query: &QueryAnagram<'_>) -> Result<Self> {
        let grams = if query.repeats > 0 {
            if query.wildcards > 0 || query.len() >= 8 {
                let partial = self.anagrams_deep(query)?;
                Self::anagrams_flat(self.library, &partial, &QueryAnagram::new(query.pattern))
                    .cloned()
                    .collect()
            } else {
                self.anagrams_fast(query)?
            }
        } else {
            Self::anagrams_flat(self.library, &self.grams, query)
                .cloned()
                .collect()
        };

        Ok(Self {
            library: self.library,
            grams,
        })
    }
}

impl<'l> From<&'l Library> for Librarian<'l> {
    fn from(library: &'l Library) -> Self {
        let grams = library.seeds.iter().map(LibGram::from).collect();
        Self { library, grams }
    }
}

#[must_use]
pub struct Iter<'a, 'l: 'a> {
    library: &'l Library,
    grams: std::slice::Iter<'a, LibGram<'l>>,
}

impl<'a, 'l: 'a> Iterator for Iter<'a, 'l> {
    type Item = Gram<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        self.grams.next().map(|lgram| lgram.as_gram(self.library))
    }
}

impl<'a, 'l> IntoIterator for &'a Librarian<'l> {
    type Item = Gram<'l>;
    type IntoIter = Iter<'a, 'l>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[must_use]
pub struct IntoIter<'l> {
    library: &'l Library,
    grams: std::vec::IntoIter<LibGram<'l>>,
}

impl<'l> Iterator for IntoIter<'l> {
    type Item = Gram<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        self.grams.next().map(|lgram| lgram.into_gram(self.library))
    }
}

impl<'l> IntoIterator for Librarian<'l> {
    type Item = Gram<'l>;
    type IntoIter = IntoIter<'l>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            library: self.library,
            grams: self.grams.into_iter(),
        }
    }
}

impl<'l> Librarian<'l> {
    fn search_deep(&self, query: &QuerySearch<'_>) -> Result<Vec<LibGram<'l>>> {
        let trie = Trie::from(self);
        let dfa = regex_automata::dfa::dense::Builder::new().build(query.pattern)?;
        self.search_trie(&trie, &dfa, query.repeats)
    }

    fn search_trie(
        &self,
        trie: &Trie<String, &LibGram<'l>>,
        dfa: &impl Automaton,
        depth: usize,
    ) -> Result<Vec<LibGram<'l>>> {
        let search = MultiHeadDFA::new(dfa, Nest::new(trie, depth))?;

        Ok(search
            .map(|node| {
                node.chain()
                    .into_iter()
                    .map(|t| t.value.expect("Returned Nodes are leaves"))
                    .collect()
            })
            .collect())
    }

    fn search_flat(&self, query: &QuerySearch<'_>) -> Result<Vec<LibGram<'l>>> {
        debug_assert_eq!(query.repeats, 0, "Flat search does not support repeats");
        let re = Regex::new(query.pattern)?;
        Ok(self
            .grams
            .iter()
            .filter_map(move |lgram| {
                let word: String;
                let text = match lgram {
                    LibGram::Word(i, ..) => self.library.seeds[*i].root.as_str(),
                    LibGram::Sequence(indices, ..) => {
                        word = indices
                            .iter()
                            .map(|&i| &self.library.seeds[i].root)
                            .join("");
                        word.as_str()
                    }
                };
                re.is_match(text).then(|| lgram.clone())
            })
            .collect())
    }

    fn anagrams_deep(&self, query: &QueryAnagram<'_>) -> Result<Vec<LibGram<'l>>> {
        let trie = Trie::from(self);
        let dfa = search::permutation::dfa_partial(query.pattern, query.wildcards)?;
        let partial = self.search_trie(&trie, &dfa, query.repeats)?;

        Ok(partial)
    }

    fn anagrams_fast(&self, query: &QueryAnagram<'_>) -> Result<Vec<LibGram<'l>>> {
        debug_assert!(
            query.len() < 8,
            "Anagram search is not optimized for long patterns"
        );
        debug_assert_eq!(query.wildcards, 0, "idk how to handle this yet");

        let trie = Trie::from(self);
        let dfa = search::permutation::dfa_exact(query.pattern)?;
        self.search_trie(&trie, &dfa, query.repeats)
    }

    fn anagrams_flat<'a>(
        library: &'l Library,
        grams: impl IntoIterator<Item = &'a LibGram<'l>>,
        query: &QueryAnagram<'_>,
    ) -> impl Iterator<Item = &'a LibGram<'l>>
    where
        'l: 'a,
    {
        debug_assert_eq!(
            query.wildcards, 0,
            "Flat anagram search does not support wildcards"
        );
        debug_assert_eq!(
            query.repeats, 0,
            "Flat anagram search does not support repeats"
        );
        let pattern: String = query.pattern.chars().sorted().collect();
        grams.into_iter().filter(move |lgram| match lgram {
            LibGram::Word(idx, ..) => library.seeds[*idx]
                .root
                .chars()
                .sorted()
                .eq(pattern.chars()),
            LibGram::Sequence(indices, ..) => {
                let key = indices.iter().flat_map(|&i| library.seeds[i].root.bytes());
                key.sorted().eq(pattern.bytes())
            }
        })
    }
}

impl<'a, 'l> From<&'a Librarian<'l>> for Trie<String, &'a LibGram<'l>> {
    fn from(librarian: &'a Librarian<'l>) -> Self {
        let mut trie = Trie::new();
        for lgram in librarian.grams.iter() {
            match lgram.as_gram(librarian.library) {
                Gram::Word(seed) => {
                    trie.insert(&seed.root, lgram);
                }
                Gram::Sequence(seeds) => {
                    let key = seeds.into_iter().map(|seed| &seed.root).join("");
                    trie.insert(&key, lgram);
                }
            }
        }
        trie
    }
}
