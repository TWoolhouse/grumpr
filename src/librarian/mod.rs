mod library;
use std::collections::HashMap;

use itertools::Itertools;
pub use library::Library;
mod error;
pub use error::{Error, Result};
mod grams;
mod search;
pub use search::query::{QueryAnagram, QueryDistance, QueryNearest, QuerySearch};
#[cfg(test)]
mod test;
pub use grams::Gram;
use grams::LibGram;
use regex::Regex;
use regex_automata::{dfa::Automaton, util::primitives::StateID};

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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Finds the nearest word to the given pattern using the Levenshtein distance.
    pub fn nearest(&self, query: &QueryNearest<'_>) -> Result<(Self, usize)> {
        let trie = Trie::from(self);
        let (dfa, dist_fn) = search::automata::levenshtein(query.pattern, 0..=query.distance)?;
        let lgrams = self.search_trie_state(&trie, &dfa, 0)?;
        let distance_id = lgrams
            .iter()
            .min_by_key(|(_, state)| dist_fn(&dfa, *state))
            .ok_or(Error::NoNearest(query.distance))?
            .1;

        Ok((
            Self {
                library: self.library,
                grams: lgrams
                    .into_iter()
                    .filter_map(|(lgram, state_id)| (state_id == distance_id).then_some(lgram))
                    .collect(),
            },
            dist_fn(&dfa, distance_id) as usize,
        ))
    }

    /// Find seeds with a Levenshtein distance to the given pattern.
    pub fn distance(&self, query: &QueryDistance<'_>) -> Result<Self> {
        let trie = Trie::from(self);

        // Strict requires us to match all distances, then filter out for the query distances.
        // because it matches using the shortest distance.
        let grams = if query.strict {
            let (dfa, dist_fn) = search::automata::levenshtein(
                query.pattern,
                0..=query.distances.iter().max().copied().unwrap_or(0),
            )?;
            let lgrams = self.search_trie_state(&trie, &dfa, 0)?;
            lgrams
                .into_iter()
                .filter_map(|(lgram, state)| {
                    let distance = dist_fn(&dfa, state);
                    (query.distances.contains(&distance)).then_some(lgram)
                })
                .collect()
        } else {
            let (dfa, _) =
                search::automata::levenshtein(query.pattern, query.distances.iter().copied())?;
            self.search_trie(&trie, &dfa, 0)?
        };

        Ok(Self {
            library: self.library,
            grams,
        })
    }

    /// Find annagrams
    pub fn anagrams(&self, query: &QueryAnagram<'_>) -> Result<Self> {
        let grams = if query.repeats > 0 {
            if query.wildcards > 0 || query.len() >= 8 {
                let first_pass = self.anagrams_deep(query)?;
                Self::anagrams_flat(self.library, &first_pass, &query.clone().repeating(0))
                    .cloned()
                    .collect()
            } else {
                self.anagrams_fast(query)?
            }
        } else if query.partial {
            self.anagrams_partial(query)?
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
            .map(|(node, _)| {
                node.chain()
                    .into_iter()
                    .map(|t| t.value.expect("Returned Nodes are leaves"))
                    .collect()
            })
            .collect())
    }

    fn search_trie_state(
        &self,
        trie: &Trie<String, &LibGram<'l>>,
        dfa: &impl Automaton,
        depth: usize,
    ) -> Result<Vec<(LibGram<'l>, StateID)>> {
        let search = MultiHeadDFA::new(dfa, Nest::new(trie, depth))?;

        Ok(search
            .map(|(node, state_id)| {
                (
                    node.chain()
                        .into_iter()
                        .map(|t| t.value.expect("Returned Nodes are leaves"))
                        .collect(),
                    state_id,
                )
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
        debug_assert_eq!(
            query.wildcards, 0,
            "Not supported yet cause it would dupe the whole trie"
        );
        debug_assert_eq!(query.partial, false, "Partial anagram search not supported");
        let trie = Trie::from(self);
        let dfa = search::automata::anagram_filter(query.pattern)?;
        let partial = self.search_trie(&trie, &dfa, query.repeats)?;

        Ok(partial)
    }

    fn anagrams_fast(&self, query: &QueryAnagram<'_>) -> Result<Vec<LibGram<'l>>> {
        debug_assert!(
            query.len() < 8,
            "Anagram search is not optimized for long patterns"
        );
        debug_assert_eq!(query.wildcards, 0, "idk how to handle this yet");
        debug_assert_eq!(query.partial, false, "Partial anagram search not supported");

        let trie = Trie::from(self);
        let dfa = search::automata::anagram(query.pattern)?;
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
        debug_assert_eq!(
            query.partial, false,
            "Flat anagram search does not support partial"
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

    fn anagrams_partial(&self, query: &QueryAnagram<'_>) -> Result<Vec<LibGram<'l>>> {
        debug_assert_eq!(
            query.wildcards, 0,
            "Partial anagram search does not support wildcards"
        );
        debug_assert_eq!(
            query.repeats, 0,
            "Partial anagram search does not support repeats"
        );

        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Anagram<'l> {
            histogram: HashMap<char, usize>,
            grams: Vec<LibGram<'l>>,
        }

        let mut anagrams: HashMap<String, Anagram<'l>> = HashMap::new();
        for lgram in &self.grams {
            let key = match lgram {
                LibGram::Word(idx, ..) => self.library.seeds[*idx].root.chars().sorted().collect(),
                LibGram::Sequence(indices, ..) => indices
                    .iter()
                    .flat_map(|&i| self.library.seeds[i].root.chars())
                    .collect(),
            };
            anagrams
                .entry(key)
                .or_insert_with_key(|key| Anagram {
                    histogram: key.chars().fold(HashMap::new(), |mut acc, c| {
                        *acc.entry(c).or_insert(0) += 1;
                        acc
                    }),
                    grams: Vec::with_capacity(1),
                })
                .grams
                .push(lgram.clone());
        }

        let pattern = query.pattern.chars().sorted().collect::<String>();
        let pattern_histogram = pattern.chars().fold(HashMap::new(), |mut acc, c| {
            *acc.entry(c).or_insert(0) += 1;
            acc
        });

        Ok(anagrams
            .into_values()
            .filter(|anagram| {
                let mut wildcards = query.wildcards as isize;
                for (c, count) in anagram.histogram.iter() {
                    let pcount = pattern_histogram.get(c).unwrap_or(&0);
                    if pcount < count {
                        wildcards -= (count - pcount) as isize;
                        if wildcards < 0 {
                            return false; // Too many characters
                        }
                    }
                }
                true // All characters matched (or partially matched) / we have enough wildcards
            })
            .flat_map(|anagram| anagram.grams)
            .collect())
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
