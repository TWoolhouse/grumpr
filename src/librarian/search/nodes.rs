use crate::{
    librarian::search::Node,
    trie::{Key, Trie, iter::Bytes},
};

impl<'a, K: Key + 'a, V: 'a> Node<u8> for &'a Trie<K, V> {
    type Children = Bytes<'a, K, V>;

    fn children(&self) -> Self::Children {
        self.bytes()
    }
    fn is_leaf(&self) -> bool {
        self.value.is_some()
    }
}
