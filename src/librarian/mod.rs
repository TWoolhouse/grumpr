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
    librarian::search::{query::QuerySearch, MultiHeadDFA, Nest},
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
    pub fn len(&self) -> usize {
        self.grams.len()
    }
    /// Returns an iterator over the grams in the librarian.
    pub fn iter(&self) -> Iter<'_, 'l> {
        Iter {
            library: self.library,
            grams: self.grams.iter(),
        }
    }

    /// Returns the seed associated with this root.
    pub fn root(&self, root: &str) -> Option<&'l Seed> {
        self.library.seeds.iter().find(|s| s.root == root)
    }
    /// Returns the seed at the given index of the library.
    pub fn index(&self, index: usize) -> Option<&'l Seed> {
        self.library.seeds.get(index)
    }

    /// Find seeds matching a regex pattern.
    pub fn search<'a>(&self, query: QuerySearch<'a>) -> Self {
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

    fn search_trie(&self, query: QuerySearch<'_>) -> Vec<LibGram<'l>> {
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

    fn search_flat(&self, query: QuerySearch<'_>) -> Vec<LibGram<'l>> {
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
        Trie::from_iter(librarian.grams.iter().map(
            |lgram| match lgram.as_gram(librarian.library) {
                Gram::Word(seed, ..) => (seed.root.clone(), lgram),
                Gram::Sequence(words, ..) => {
                    (words.iter().map(|s| &s.root).join("").to_string(), lgram)
                }
            },
        ))
    }
}

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
