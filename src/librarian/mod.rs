mod library;
use itertools::Itertools;
pub use library::Library;
use std::{collections::HashSet, iter::FusedIterator};
mod error;
pub use error::{Error, Result};
mod grams;
mod search;
mod stats;
pub use search::query;
pub use stats::Stats;
mod anagram;
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
    pub root: String,
    /// The index of the root in the library
    pub index: usize,
    /// The number of occurrences of this root in the text
    pub count: u64,
}

// TODO: Become trait over the library and sublib-ref
// Would allow for mmap the library remove the overhead of
// cloning the library into a librarian.

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

    /// Return the gram at the given index of the librarian.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<Gram<'l>> {
        self.grams
            .get(index)
            .map(|lgram| lgram.as_gram(self.library))
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
    pub fn search(&self, query: &query::Match<'_>) -> Result<Self> {
        let grams = if query.depth > 0 {
            self.search_deep(query)?
        } else {
            self.search_flat(query)?
        };

        Ok(self.child(grams))
    }
    /// Nearest word search
    /// Finds the nearest word to the given pattern using the Levenshtein distance.
    pub fn nearest(&self, query: &query::Nearest<'_>) -> Result<(Self, usize)> {
        let trie = Trie::from(self);
        let (dfa, dist_fn) = search::automata::levenshtein(query.pattern, 0..=query.distance)?;
        let lgrams = self.search_trie_state(&trie, &dfa, 0)?;
        let distance_id = lgrams
            .iter()
            .min_by_key(|(_, state)| dist_fn(&dfa, *state))
            .ok_or(Error::NoNearest(query.distance))?
            .1;

        Ok((
            self.child(
                lgrams
                    .into_iter()
                    .filter_map(|(lgram, state_id)| (state_id == distance_id).then_some(lgram))
                    .collect(),
            ),
            dist_fn(&dfa, distance_id) as usize,
        ))
    }

    /// Find seeds with a Levenshtein distance to the given pattern.
    pub fn distance(&self, query: &query::Distance<'_>) -> Result<Self> {
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

        Ok(self.child(grams))
    }

    /// Find anagrams
    pub fn anagrams(&self, query: &query::Anagram<'_>) -> Result<Self> {
        // Choose the anagram search method based on the query parameters.

        let grams = if query.depth > 0 {
            if query.wildcards > 0 || query.len() >= 8 {
                // Perform a first pass to build the deep tree whilst filtering some
                // of the certainly not matching anagrams.

                let trie = Trie::from(self);
                let dfa = search::automata::anagram_filter(query.pattern)?;
                let first_pass = self.search_trie(&trie, &dfa, query.depth)?;

                // Perform an expensive anagram search on the first pass results.
                if query.partial {
                    anagram::partial(self.library, &first_pass, query.pattern, query.wildcards)
                        .cloned()
                        .collect()
                } else {
                    anagram::exact(self.library, &first_pass, query.pattern, query.wildcards)
                        .cloned()
                        .collect()
                }
            } else {
                let trie = Trie::from(self);
                anagram::trie_dfa(&trie, query.pattern, query.depth)?
            }
        } else if query.wildcards > 0 {
            anagram::exact(self.library, &self.grams, query.pattern, query.wildcards)
                .cloned()
                .collect()
        } else if query.partial {
            anagram::partial(
                self.library,
                self.grams.iter(),
                query.pattern,
                query.wildcards,
            )
            .cloned()
            .collect()
        } else {
            anagram::sorted(self.library, &self.grams, query.pattern)
                .cloned()
                .collect()
        };

        Ok(self.child(grams))
    }

    pub fn whitelist<'a>(&self, it: impl IntoIterator<Item = &'a str>) -> Self {
        let whitelist = it.into_iter().collect::<HashSet<_>>();
        self.filter(|seed| whitelist.contains(seed.root.as_str()))
    }

    pub fn blacklist<'a>(&self, it: impl IntoIterator<Item = &'a str>) -> Self {
        let blacklist = it.into_iter().collect::<HashSet<_>>();
        self.filter(|seed| !blacklist.contains(seed.root.as_str()))
    }

    pub fn filter<'a>(&self, f: impl FnMut(&'l Seed) -> bool) -> Self {
        self.child(self.filter_seed(f).collect())
    }

    pub fn has(&self, query: &query::Has<'_>) -> Result<Self> {
        Ok(self.child(
            anagram::atleast(self.library, self.grams.iter(), query.characters)
                .cloned()
                .collect(),
        ))
    }

    pub fn stats(&self) -> Stats {
        self.into()
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
impl<'a, 'l: 'a> DoubleEndedIterator for Iter<'a, 'l> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.grams
            .next_back()
            .map(|lgram| lgram.as_gram(self.library))
    }
}
impl<'a, 'l: 'a> ExactSizeIterator for Iter<'a, 'l> {
    fn len(&self) -> usize {
        self.grams.len()
    }
}
impl<'a, 'l: 'a> FusedIterator for Iter<'a, 'l> {}

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
impl<'l> DoubleEndedIterator for IntoIter<'l> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.grams
            .next_back()
            .map(|lgram| lgram.into_gram(self.library))
    }
}
impl<'l> ExactSizeIterator for IntoIter<'l> {
    fn len(&self) -> usize {
        self.grams.len()
    }
}
impl<'l> FusedIterator for IntoIter<'l> {}

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
    fn child(&self, grams: Vec<LibGram<'l>>) -> Self {
        Self {
            library: self.library,
            grams,
        }
    }

    fn search_deep(&self, query: &query::Match<'_>) -> Result<Vec<LibGram<'l>>> {
        let trie = Trie::from(self);
        let dfa = regex_automata::dfa::dense::Builder::new().build(query.pattern)?;
        self.search_trie(&trie, &dfa, query.depth)
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

    fn search_flat(&self, query: &query::Match<'_>) -> Result<Vec<LibGram<'l>>> {
        debug_assert_eq!(query.depth, 0, "Flat search does not support repeats");
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

    fn filter_seed(
        &self,
        mut f: impl FnMut(&'l Seed) -> bool,
    ) -> impl Iterator<Item = LibGram<'l>> {
        self.grams
            .iter()
            .filter(move |lgram| match lgram.as_gram(self.library) {
                Gram::Word(seed) => f(seed),
                Gram::Sequence(seeds) => seeds.iter().all(|&s| f(s)),
            })
            .cloned()
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
