use crate::librarian::{Librarian, Library};

#[derive(Debug)]
pub struct Subset<'a> {
    /// The parent librarian
    pub(super) parent: &'a Library,
    /// The indices of the roots in the parent librarian
    pub(super) indices: Vec<usize>,
}

impl<'a> Librarian for Subset<'a> {
    type Mask<'b>
        = Subset<'b>
    where
        Self: 'b;

    fn root(&self, root: &str) -> Option<&crate::librarian::Root> {
        self.parent.root(root)
    }

    fn index(&self, index: usize) -> Option<&crate::librarian::Root> {
        self.indices.get(index).and_then(|&i| self.parent.index(i))
    }

    fn mask(&self, query: &impl regex_automata::dfa::Automaton) -> Self::Mask<'a> {
        let new = self
            .parent
            .mask(query)
            .indices
            .into_iter()
            .filter(|&i| self.indices.contains(&i));

        Subset {
            parent: self.parent,
            indices: new.collect(),
        }
    }

    fn len(&self) -> usize {
        self.indices.len()
    }
}
