use std::{collections::HashMap, marker::PhantomData};

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    pub string: String,
    pub index: usize,
    pub count: u64,
}

impl Root {
    pub fn new(string: String, count: u64) -> Self {
        Self {
            string,
            index: 0,
            count,
        }
    }
}

impl PartialOrd for Root {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.index.partial_cmp(&other.index) {
            Some(std::cmp::Ordering::Equal) => match other.count.partial_cmp(&self.count) {
                Some(std::cmp::Ordering::Equal) => self.string.partial_cmp(&other.string),
                otherwise => otherwise,
            },
            otherwise => otherwise,
        }
    }
}
impl Ord for Root {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.index.cmp(&other.index) {
            std::cmp::Ordering::Equal => match other.count.cmp(&self.count) {
                std::cmp::Ordering::Equal => self.string.cmp(&other.string),
                otherwise => otherwise,
            },
            otherwise => otherwise,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Book {
    pub dataset: Vec<Root>,
}

impl Book {
    pub fn new() -> Self {
        Self {
            dataset: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gram<'a> {
    pub root: &'a Root,
    pub string: &'a str,
    pub frequency: f64,
}

impl<'a> PartialEq for Gram<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.root.eq(other.root)
    }
}
impl<'a> Eq for Gram<'a> {}

impl<'a> PartialOrd for Gram<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.root.partial_cmp(other.root)
    }
}
impl<'a> Ord for Gram<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.root.cmp(other.root)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Display {
    pub string: bool,
    pub rank: bool,
    pub count: bool,
    pub frequency: bool,
}

impl<'a> Gram<'a> {
    pub fn new(gram: &'a Root, frequency: f64) -> Self {
        Self {
            root: gram,
            string: &gram.string,
            frequency,
        }
    }

    pub fn display(&'a self, flags: Display) -> GramDisplay<'a> {
        GramDisplay(self, flags)
    }
}

pub struct GramDisplay<'a>(&'a Gram<'a>, Display);

impl<'a> std::fmt::Display for GramDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output: Vec<String> = Vec::with_capacity(3);
        if self.1.string {
            output.push(self.0.root.string.clone());
        }
        if self.1.rank {
            output.push(format!("#{}", self.0.root.index));
        }
        if self.1.count {
            output.push(format!("@{}", self.0.root.count));
        }
        if self.1.frequency {
            output.push(format!("{:.5}%", self.0.frequency * 100.0));
        }
        f.write_str(&output.join("\t"))
    }
}

#[derive(Debug, Default)]
pub struct Corpus<'a> {
    pub grams: Vec<Gram<'a>>,
    pub count: u64,
    phantom: PhantomData<&'a Book>,
}

impl<'a> IntoIterator for &'a Corpus<'a> {
    type Item = &'a Gram<'a>;
    type IntoIter = core::slice::Iter<'a, Gram<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.grams.iter()
    }
}

impl<'a> FromIterator<&'a Gram<'a>> for Corpus<'a> {
    fn from_iter<T: IntoIterator<Item = &'a Gram<'a>>>(iter: T) -> Self {
        let mut corpus = Corpus {
            grams: iter.into_iter().map(|gram| gram.root.into()).collect(),
            ..Default::default()
        };
        corpus.count().freq();
        corpus
    }
}

impl<'a> Corpus<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn count(&mut self) -> &mut Self {
        self.count = self.grams.iter().map(|gram| gram.root.count).sum();
        self
    }

    pub fn freq(&mut self) {
        let count = self.count as f64;
        for gram in &mut self.grams {
            gram.frequency = gram.root.count as f64 / count;
        }
    }

    pub fn truncate(&'a self, size: usize) -> Corpus<'a> {
        self.iter().take(size).collect()
    }

    pub fn len(&self) -> usize {
        self.grams.len()
    }

    pub fn is_empty(&self) -> bool {
        self.grams.is_empty()
    }

    pub fn as_slice(&self) -> &[Gram<'a>] {
        self.grams.as_slice()
    }

    pub fn sort(&mut self) {
        self.grams.sort();
    }

    pub fn nths(&'a self) -> HashMap<usize, Corpus<'a>> {
        let mut dict: HashMap<usize, Corpus<'a>> = HashMap::new();
        for gram in self {
            dict.entry(gram.string.len())
                .or_default()
                .grams
                .push(gram.root.into())
        }
        for corpus in &mut dict.values_mut() {
            corpus.count().freq();
        }
        dict
    }

    pub fn iter(&self) -> std::slice::Iter<Gram> {
        self.into_iter()
    }

    pub fn find(&'a self, ngram: &str) -> Option<&'a Gram<'a>> {
        self.grams.iter().find(|gram| gram.string == ngram)
    }

    pub fn anagrams(&'a self, ngram: &str) -> Corpus<'a> {
        use itertools::Itertools;
        let original = ngram.chars().sorted().collect::<String>();
        self.iter()
            .filter(|gram| gram.string.chars().sorted().collect::<String>() == original)
            .collect()
    }

    pub fn wildcard(&'a self, pattern: Regex) -> Corpus<'a> {
        self.iter()
            .filter(|gram| pattern.is_match(gram.string))
            .collect()
    }

    pub fn fuzzy_find(&'a self) -> Corpus<'a> {
        todo!("Fuzzy Find words based on levenshtein distance & freq")
    }

    pub fn display(&'a self, flags: Display) -> CorpusDisplay<'a> {
        CorpusDisplay(self, flags)
    }
}

pub struct CorpusDisplay<'a>(&'a Corpus<'a>, Display);

impl<'a> std::fmt::Display for CorpusDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use itertools::Itertools;
        f.write_str(
            &self
                .0
                .grams
                .iter()
                .map(|gram| gram.display(self.1).to_string())
                .join("\n"),
        )
    }
}

impl Book {
    pub fn corpus(&self) -> Corpus {
        let mut corpus = Corpus::new();
        for root in &self.dataset {
            corpus.grams.push(root.into())
        }
        corpus.count().freq();
        corpus.sort();
        corpus
    }
}

impl<'a> From<&'a Root> for Gram<'a> {
    fn from(value: &'a Root) -> Self {
        Gram::new(value, 0.0)
    }
}
