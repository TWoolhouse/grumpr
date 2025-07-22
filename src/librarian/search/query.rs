#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuerySearch<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) repeats: usize,
}

impl<'a> QuerySearch<'a> {
    pub fn new(pattern: &'a str) -> Self {
        Self {
            pattern,
            repeats: 0,
        }
    }

    pub fn repeating(mut self, times: usize) -> Self {
        self.repeats = times;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryAnagram<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) wildcards: usize,
    pub(in crate::librarian) repeats: usize,
}

impl QueryAnagram<'_> {
    pub(crate) fn len(&self) -> usize {
        self.pattern.len() + self.wildcards
    }
}

impl<'a> QueryAnagram<'a> {
    pub fn new(pattern: &'a str) -> Self {
        Self {
            pattern,
            wildcards: 0,
            repeats: 0,
        }
    }

    pub fn wildcards(mut self, wildcards: usize) -> Self {
        self.wildcards = wildcards;
        self
    }

    pub fn repeating(mut self, times: usize) -> Self {
        self.repeats = times;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryNearest<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) distance: u8,
}

impl<'a> QueryNearest<'a> {
    pub fn new(pattern: &'a str, distance: u8) -> Self {
        Self { pattern, distance }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryDistance<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) distances: Vec<u8>,
    pub(in crate::librarian) strict: bool,
}

impl<'a> QueryDistance<'a> {
    pub fn new(pattern: &'a str, distances: impl IntoIterator<Item = u8>) -> Self {
        Self {
            pattern,
            distances: distances.into_iter().collect(),
            strict: false,
        }
    }

    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}
