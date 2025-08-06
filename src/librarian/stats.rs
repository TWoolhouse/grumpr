use std::collections::HashSet;

use super::{Gram, Librarian};
use serde::Serialize;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct StatsOccurrences {
    /// Total number of unique seeds in the library
    pub seeds: u64,
    /// Total number of elements in the library
    pub ngrams: u64,
    /// Total number of characters over all seeds
    pub chars_seeds: u64,
    /// Total number of characters over all ngrams
    pub chars_ngrams: u64,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Stats {
    /// Total number of unique seeds in the library
    pub seeds: usize,
    /// Total number of elements in the library
    pub ngrams: usize,
    /// Total number of seeds over all ngrams
    pub ngram_seeds: usize,
    /// Total number of characters over all seeds
    pub chars_seeds: usize,
    /// Total number of characters over all ngrams
    pub chars_ngrams: usize,
    pub occurrences: StatsOccurrences,
}

impl Stats {
    fn new<'a, 'l>(librarian: &'a Librarian<'l>) -> Self {
        let mut stats = Self::default();

        let mut unique_seeds = HashSet::<usize>::new();
        for gram in librarian.iter() {
            stats.ngrams += 1;
            match gram {
                Gram::Word(seed) => {
                    stats.ngram_seeds += 1;
                    stats.occurrences.ngrams += seed.count;
                    let chars = seed.root.chars().count();
                    if unique_seeds.insert(seed.index) {
                        stats.seeds += 1;
                        stats.chars_seeds += chars;
                        stats.occurrences.seeds += seed.count;
                        stats.occurrences.chars_seeds += chars as u64 * seed.count;
                    }
                    stats.chars_ngrams += chars;
                    stats.occurrences.chars_ngrams += chars as u64 * seed.count;
                }
                Gram::Sequence(seeds) => {
                    stats.ngram_seeds += seeds.len();
                    for seed in seeds {
                        stats.occurrences.ngrams += seed.count;
                        let chars = seed.root.chars().count();
                        if unique_seeds.insert(seed.index) {
                            stats.seeds += 1;
                            stats.chars_seeds += chars;
                            stats.occurrences.seeds += seed.count;
                            stats.occurrences.chars_seeds += chars as u64 * seed.count;
                        }
                        stats.chars_ngrams += chars;
                        stats.occurrences.chars_ngrams += chars as u64 * seed.count;
                    }
                }
            }
        }

        stats
    }
}

impl<'a, 'l: 'a> From<&'a Librarian<'l>> for Stats {
    fn from(librarian: &'a Librarian<'l>) -> Self {
        Self::new(librarian)
    }
}
