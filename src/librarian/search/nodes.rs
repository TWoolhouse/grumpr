use crate::{
    librarian::search::Node,
    trie::{iter::Bytes, Key, Trie},
};

impl<'a, K: Key + 'a, V: 'a> Node for &'a Trie<K, V> {
    type Children = Bytes<'a, K, V>;

    fn children(&self) -> Self::Children {
        self.bytes()
    }
    fn is_leaf(&self) -> bool {
        self.value.is_some()
    }
}
