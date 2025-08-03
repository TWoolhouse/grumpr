use crate::librarian::{Library, Seed};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gram<'l> {
    /// A single word
    Word(&'l Seed),
    /// A sequence of words
    Sequence(Vec<&'l Seed>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LibGram<'l> {
    /// A single word
    Word(usize, PhantomData<&'l ()>),
    /// A sequence of words
    Sequence(Vec<usize>, PhantomData<&'l ()>),
}

impl<'l> Gram<'l> {
    #[must_use]
    pub fn word(&self) -> Option<&'l Seed> {
        match self {
            Gram::Word(seed) => Some(seed),
            Gram::Sequence(_) => None,
        }
    }
    #[must_use]
    pub fn sequence(self) -> Option<Vec<&'l Seed>> {
        match self {
            Gram::Word(_) => None,
            Gram::Sequence(seeds) => Some(seeds),
        }
    }

    #[must_use]
    pub fn seeds(self) -> Vec<&'l Seed> {
        match self {
            Gram::Word(seed) => vec![seed],
            Gram::Sequence(seeds) => seeds,
        }
    }

    #[must_use]
    fn degrade(self) -> Self {
        use Gram::*;
        match self {
            Sequence(words) if words.len() == 1 => Word(words[0]),
            _ => self,
        }
    }
}

impl<'l> LibGram<'l> {
    #[inline]
    #[must_use]
    pub fn into_gram(self, library: &'l Library) -> Gram<'l> {
        match self {
            LibGram::Word(index, _) => Gram::Word(&library.seeds[index]),
            LibGram::Sequence(indices, _) => {
                Gram::Sequence(indices.into_iter().map(|i| &library.seeds[i]).collect())
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn as_gram(&self, library: &'l Library) -> Gram<'l> {
        match self {
            LibGram::Word(index, _) => Gram::Word(&library.seeds[*index]),
            LibGram::Sequence(indices, _) => {
                Gram::Sequence(indices.iter().map(|&i| &library.seeds[i]).collect())
            }
        }
    }

    /// Ensure that a sequence with a single word is represented as a `Word`.
    #[must_use]
    fn degrade(self) -> Self {
        use LibGram::*;
        match self {
            Sequence(words, ..) if words.len() == 1 => Word(words[0], PhantomData),
            _ => self,
        }
    }
}

impl<'l> From<&'l Seed> for Gram<'l> {
    fn from(seed: &'l Seed) -> Self {
        Gram::Word(seed)
    }
}

impl<'l> FromIterator<&'l Seed> for Gram<'l> {
    fn from_iter<T: IntoIterator<Item = &'l Seed>>(iter: T) -> Self {
        Gram::Sequence(iter.into_iter().collect()).degrade()
    }
}

impl<'l> From<Gram<'l>> for LibGram<'l> {
    fn from(gram: Gram<'l>) -> Self {
        match gram {
            Gram::Word(seed) => LibGram::Word(seed.index, PhantomData),
            Gram::Sequence(seeds) => {
                LibGram::Sequence(seeds.into_iter().map(|s| s.index).collect(), PhantomData)
            }
        }
    }
}

impl<'l> From<&'l Seed> for LibGram<'l> {
    fn from(seed: &'l Seed) -> Self {
        LibGram::Word(seed.index, PhantomData)
    }
}

impl<'l> FromIterator<LibGram<'l>> for LibGram<'l> {
    fn from_iter<T: IntoIterator<Item = LibGram<'l>>>(iter: T) -> Self {
        LibGram::Sequence(
            iter.into_iter()
                .flat_map(|lgram| match lgram {
                    LibGram::Word(index, _) => vec![index],
                    LibGram::Sequence(indices, _) => indices,
                })
                .collect(),
            PhantomData,
        )
        .degrade()
    }
}

impl<'a, 'l> FromIterator<&'a LibGram<'l>> for LibGram<'l> {
    fn from_iter<T: IntoIterator<Item = &'a LibGram<'l>>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let mut vec = Vec::with_capacity(iter.size_hint().0);
        for lgram in iter {
            match lgram {
                Self::Word(word, ..) => vec.push(*word),
                Self::Sequence(indices, ..) => vec.extend(indices.iter().copied()),
            }
        }
        Self::Sequence(vec, PhantomData).degrade()
    }
}

impl<'l> FromIterator<&'l Seed> for LibGram<'l> {
    fn from_iter<T: IntoIterator<Item = &'l Seed>>(iter: T) -> Self {
        Self::Sequence(iter.into_iter().map(|s| s.index).collect(), PhantomData).degrade()
    }
}

impl FromIterator<usize> for LibGram<'_> {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Self::Sequence(iter.into_iter().collect(), PhantomData).degrade()
    }
}
