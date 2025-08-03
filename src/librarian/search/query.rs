//! Query types for searching within a [Librarian](crate::librarian::Librarian).
//! These queries are used to specify the criteria for searching grams in the [Librarian](crate::librarian::Librarian).
//!
//! Some queries share common options which are documented below.
//!
//! ## Depth
//! Determines how many times grams can be repeated in the search.
//!
//! A depth of 0 means no repeats, a depth of 1 means each gram can be used twice,
//! and so on.
//!
//! Formally, let `S` be the set of grams in the [Librarian](crate::librarian::Librarian), then
//! a depth of `d` means that the search will be performed over `S^(d+1)`
//! (the `d+1`-[fold Cartesian product](https://en.wikipedia.org/wiki/Cartesian_product#Cartesian_products_of_several_sets) of `S` with itself).
//!
//! A depth of 0 is the default.

/// A query that matches a regex pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Match<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) depth: usize,
}

impl<'a> From<&'a str> for Match<'a> {
    fn from(pattern: &'a str) -> Self {
        Self::new(pattern)
    }
}

impl<'a> Match<'a> {
    pub fn new(pattern: &'a str) -> Self {
        Self { pattern, depth: 0 }
    }

    /// Set the depth of the search. See the [module](self) documentation for details.
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }
}

/// Search for anagrams given a pattern of characters.
/// An anagram is a rearrangement of the characters in the pattern.
///
/// The query may contain [wildcards](Self::wildcards), which are unknown characters
/// in the pattern that can match any character.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Anagram<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) wildcards: usize,
    pub(in crate::librarian) depth: usize,
    pub(in crate::librarian) partial: bool,
}

impl<'a> From<&'a str> for Anagram<'a> {
    fn from(pattern: &'a str) -> Self {
        Self::new(pattern)
    }
}

impl Anagram<'_> {
    /// The number of characters in the pattern, including wildcards.
    pub(crate) fn len(&self) -> usize {
        self.pattern.len() + self.wildcards
    }
}

impl<'a> Anagram<'a> {
    /// Create a new anagram query with the given pattern.
    ///
    /// The pattern is a string of exact characters to match.
    /// By default, the query has no [wildcards](Self::wildcards), a [depth](Self::depth) of 0, and does **not** allow [partial](Self::partial) matches.
    pub fn new(pattern: &'a str) -> Self {
        Self {
            pattern,
            wildcards: 0,
            depth: 0,
            partial: false,
        }
    }

    /// Set the number of wildcards in the anagram.
    /// Wildcards are unknown characters that can match any character.
    pub fn wildcards(mut self, wildcards: usize) -> Self {
        self.wildcards = wildcards;
        self
    }

    /// Set the depth of the search. See the [module](self) documentation for details.
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Allow partial anagrams, which are anagrams that can be formed from a (non-strict) subset of the letters.
    /// By default, partial anagrams are not allowed.
    /// This means that the anagram must use all characters in the pattern.
    pub fn partial(mut self, partial: bool) -> Self {
        self.partial = partial;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nearest<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) distance: u8,
}

impl<'a> Nearest<'a> {
    pub fn new(pattern: &'a str, distance: u8) -> Self {
        Self { pattern, distance }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Distance<'a> {
    pub(in crate::librarian) pattern: &'a str,
    pub(in crate::librarian) distances: Vec<u8>,
    pub(in crate::librarian) strict: bool,
}

impl<'a> Distance<'a> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Has<'a> {
    pub(in crate::librarian) characters: &'a str,
}

impl<'a> From<&'a str> for Has<'a> {
    fn from(characters: &'a str) -> Self {
        Self::new(characters)
    }
}

impl<'a> Has<'a> {
    pub fn new(characters: &'a str) -> Self {
        Self { characters }
    }
}
