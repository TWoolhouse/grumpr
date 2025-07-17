mod library;
use itertools::Itertools;
pub use library::Library;
mod grams;
mod search;
#[cfg(test)]
mod test;
pub use grams::Gram;
use grams::LibGram;
use regex::Regex;

use crate::{
    librarian::search::{MultiHeadDFA, Nest, query::QuerySearch},
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
    #[must_use]
    pub fn search(&self, query: &QuerySearch<'_>) -> Self {
        let grams = if query.repeats > 0 {
            self.search_trie(query)
        } else {
            self.search_flat(query)
        };

        Self {
            library: self.library,
            grams,
        }
    }
    /// Find annagrams // TODO: Multiple words i.e. ngrams
    // fn anagrams(&self);
    /// Nearest word search
    // fn nearest(&self, pattern: &str) -> Option<&Seed>; // impl Librarian + '_;

    #[must_use]
    fn search_trie(&self, query: &QuerySearch<'_>) -> Vec<LibGram<'l>> {
        let trie = Trie::from(self);

        let dfa = regex_automata::dfa::dense::Builder::new()
            .build(query.pattern)
            .expect("Failed to build DFA");

        let search = MultiHeadDFA::new(&dfa, Nest::new(&trie, query.repeats))
            .expect("Failed to create MultiHeadDFA");

        search
            .map(|node| node.chain().map(|t| t.value.unwrap()).collect())
            .collect()
    }

    #[must_use]
    fn search_flat(&self, query: &QuerySearch<'_>) -> Vec<LibGram<'l>> {
        let re = Regex::new(query.pattern).unwrap();
        self.grams
            .iter()
            .filter_map(move |lgram| {
                let word: String;
                let text = match lgram.as_gram(self.library) {
                    Gram::Word(word, ..) => word.root.as_str(),
                    Gram::Sequence(words, ..) => {
                        word = words.iter().map(|s| &s.root).join("");
                        word.as_str()
                    }
                };
                re.is_match(text).then(|| lgram.clone())
            })
            .collect()
    }
}

impl<'l> From<&'l Library> for Librarian<'l> {
    fn from(library: &'l Library) -> Self {
        let grams = library.seeds.iter().map(LibGram::from).collect();
        Self { library, grams }
    }
}

impl<'a, 'l> From<&'a Librarian<'l>> for Trie<String, &'a LibGram<'l>> {
    fn from(librarian: &'a Librarian<'l>) -> Self {
        let mut trie = Trie::new();
        for lgram in librarian.grams.iter() {
            match lgram.as_gram(&librarian.library) {
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
