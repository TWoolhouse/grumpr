use regex_automata::dfa::Automaton;

use crate::{
    librarian::{
        search::{MultiHeadDFA, Node},
        Librarian, Root, Subset,
    },
    trie::{iter::Leaflets, Key, Leaflet, Notch, Trie},
};

#[derive(Debug)]
pub struct Library {
    pub(super) trie: Trie<String, usize>,
    // pub(super) roots: Vec<Root>,
    pub roots: Vec<Root>,
}

impl FromIterator<(String, u64)> for Library {
    fn from_iter<T: IntoIterator<Item = (String, u64)>>(iter: T) -> Self {
        let mut trie = Trie::new();
        let mut roots = Vec::new();
        let mut index = 0;

        for (root, count) in iter {
            trie.insert(root.clone(), index);
            roots.push(Root { root, index, count });
            index += 1;
        }

        Library { trie, roots }
    }
}

impl Librarian for Library {
    type Mask<'a> = Subset<'a>;

    fn root(&self, root: &str) -> Option<&Root> {
        self.trie.get(root).map(|&index| &self.roots[index])
    }

    fn index(&self, index: usize) -> Option<&Root> {
        self.roots.get(index)
    }

    fn mask(&self, query: &impl Automaton) -> Self::Mask<'_> {
        let search = MultiHeadDFA::new(query, Leaflet { trie: &self.trie }).unwrap();
        let leaves = search.into_iter().map(|leaflet| leaflet.value.unwrap());

        Subset {
            parent: self,
            indices: leaves.collect(),
        }
    }

    fn len(&self) -> usize {
        self.roots.len()
    }
}

impl<'a, K: Key + 'a, V: 'a> Node for Leaflet<'a, K, V> {
    type Children = Leaflets<'a, K, V>;
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_ {
        self.notch.as_bytes()
    }
    fn children(&self) -> Self::Children {
        self.trie.leaflets()
    }
    fn is_leaf(&self) -> bool {
        self.trie.try_as_leaf().is_some()
    }
}
