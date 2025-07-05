use regex_automata::dfa::Automaton;

mod library;
pub use library::Library;
mod search;
mod subset;
pub use subset::Subset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    /// The root string
    pub root: String,
    /// The index of the root in the library
    index: usize,
    /// The number of occurrences of this root in the text
    count: u64,
}

pub trait Librarian {
    type Mask<'a>: Librarian
    where
        Self: 'a;
    /// Returns the root associated with this gram.
    fn root(&self, root: &str) -> Option<&Root>;
    /// Returns the root at the given index.
    fn index(&self, index: usize) -> Option<&Root>;
    /// Returns a new librarian that contains only the roots that match the given query.
    fn mask(&self, query: &impl Automaton) -> Self::Mask<'_>;
    /// Returns the number of roots in the librarian.
    fn len(&self) -> usize;
}
