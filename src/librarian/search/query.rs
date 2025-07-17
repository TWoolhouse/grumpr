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
