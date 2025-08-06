use std::collections::HashMap;

use crate::{
    Library,
    librarian::{
        LibGram, Result,
        search::{MultiHeadDFA, Nest, automata},
    },
    trie::Trie,
};
use itertools::Itertools;

type Histogram = HashMap<char, usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Anagram<'a, 'l: 'a> {
    pub histogram: Histogram,
    pub grams: Vec<&'a LibGram<'l>>,
}

/// Create a histogram from a pattern string.
pub(crate) fn histogram(pattern: &str) -> Histogram {
    histogram_sorted(pattern.chars().sorted())
}

/// Create a histogram from an iterator of sorted characters.
fn histogram_sorted(pattern: impl IntoIterator<Item = char>) -> Histogram {
    pattern.into_iter().fold(HashMap::new(), |mut acc, c| {
        *acc.entry(c).or_insert(0) += 1;
        acc
    })
}

pub(crate) fn histograms<'a, 'l: 'a>(
    library: &'l Library,
    lgrams: impl IntoIterator<Item = &'a LibGram<'l>>,
) -> HashMap<String, Anagram<'a, 'l>> {
    histograms_by_key(lgrams.into_iter().map(|lgram| {
        let key = match lgram {
            LibGram::Word(idx, ..) => library.seeds[*idx].root.chars().sorted().collect(),
            LibGram::Sequence(indices, ..) => indices
                .iter()
                .flat_map(|&i| library.seeds[i].root.chars())
                .collect(),
        };
        (lgram, key)
    }))
}

pub(crate) fn histograms_by_key<'a, 'l: 'a>(
    keys: impl IntoIterator<Item = (&'a LibGram<'l>, String)>,
) -> HashMap<String, Anagram<'a, 'l>> {
    let mut anagrams: HashMap<String, Anagram<'a, 'l>> = HashMap::new();
    for (lgram, key) in keys {
        anagrams
            .entry(key)
            .or_insert_with_key(|key| Anagram {
                histogram: histogram_sorted(key.chars()),
                grams: Vec::with_capacity(1),
            })
            .grams
            .push(lgram);
    }
    anagrams
}

pub(crate) fn sorted<'a, 'l: 'a>(
    library: &Library,
    lgrams: impl IntoIterator<Item = &'a LibGram<'l>>,
    pattern: &str,
) -> impl Iterator<Item = &'a LibGram<'l>> {
    let pattern: String = pattern.chars().sorted().collect();
    lgrams.into_iter().filter(move |lgram| match lgram {
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

pub(crate) fn trie_dfa<'l>(
    trie: &Trie<String, &LibGram<'l>>,
    pattern: &str,
    depth: usize,
) -> Result<Vec<LibGram<'l>>> {
    debug_assert!(
        pattern.chars().count() < 8,
        "Anagram search is not optimized for long patterns"
    );
    let dfa = automata::anagram(pattern)?;
    let search = MultiHeadDFA::new(&dfa, Nest::new(trie, depth))?;

    Ok(search
        .map(|(node, _)| {
            node.chain()
                .into_iter()
                .map(|t| t.value.expect("Returned Nodes are leaves"))
                .collect()
        })
        .collect())
}

pub(crate) fn partial<'a, 'l: 'a>(
    library: &'l Library,
    lgrams: impl IntoIterator<Item = &'a LibGram<'l>>,
    pattern: &str,
    wildcards: usize,
) -> impl Iterator<Item = &'a LibGram<'l>> {
    let pattern_histogram = histogram(pattern);
    let anagrams = histograms(library, lgrams);

    anagrams
        .into_values()
        .filter(move |anagram| {
            let mut wildcards = wildcards as isize;
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
}

pub(crate) fn exact<'a, 'l: 'a>(
    library: &'l Library,
    lgrams: impl IntoIterator<Item = &'a LibGram<'l>>,
    pattern: &str,
    wildcards: usize,
) -> impl Iterator<Item = &'a LibGram<'l>> {
    let pattern_histogram = histogram(pattern);
    let anagrams = histograms(library, lgrams);

    anagrams
        .into_values()
        .filter(move |anagram| {
            let mut wildcards = wildcards as isize;
            for (c, count) in anagram.histogram.iter() {
                let pcount = pattern_histogram.get(c).unwrap_or(&0);
                if pcount < count {
                    wildcards -= (count - pcount) as isize;
                    if wildcards < 0 {
                        return false; // Too many characters
                    }
                } else if count < pcount {
                    return false; // Not enough characters for an exact match
                }
            }
            wildcards == 0 // All characters matched / we have the exact right amount of wildcards
        })
        .flat_map(|anagram| anagram.grams)
}

pub(crate) fn atleast<'a, 'l: 'a>(
    library: &'l Library,
    lgrams: impl IntoIterator<Item = &'a LibGram<'l>>,
    pattern: &str,
) -> impl Iterator<Item = &'a LibGram<'l>> {
    let pattern_histogram = histogram(pattern);
    let anagrams = histograms(library, lgrams);

    anagrams
        .into_values()
        .filter(move |anagram| {
            for (c, pcount) in pattern_histogram.iter() {
                if let Some(count) = anagram.histogram.get(c) {
                    if count < pcount {
                        return false; // Not enough characters
                    }
                } else {
                    return false; // Character not found
                }
            }
            true // All characters matched
        })
        .flat_map(|anagram| anagram.grams)
}
