use super::{Key, Trie, iter::Bytes, keys::KeyNibbles};
use std::borrow::Borrow;

impl<K: Key, V> Default for Trie<K, V> {
    fn default() -> Self {
        Trie {
            value: None,
            children: Default::default(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<K: Key, V> Trie<K, V> {
    #[must_use]
    pub fn new() -> Self {
        Trie::default()
    }

    pub fn insert<Q>(&mut self, key: &Q, value: V) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Key + ?Sized,
    {
        let mut current_node = self;
        let mut nibbles = key.as_nibbles().into_iter();
        while let Some(index) = nibbles.next() {
            if let Some(ref mut next) = current_node.children[index as usize] {
                current_node = next;
            } else {
                current_node.children[index as usize] =
                    Some(Box::new(Self::insert_fast(nibbles, value)));
                return None;
            }
        }

        current_node.value.replace(value)
    }

    #[must_use]
    fn insert_fast(nibbles: impl Iterator<Item = u8>, value: V) -> Self {
        let mut top = Trie::new();
        let mut trie = &mut top;
        for index in nibbles {
            trie.children[index as usize] = Some(Box::new(Trie::new()));
            trie = trie.children[index as usize].as_deref_mut().unwrap();
        }
        trie.value = Some(value);
        top
    }

    #[must_use]
    pub fn _get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Key + ?Sized,
    {
        let mut current_node = self;
        for index in key.as_nibbles() {
            current_node = current_node.children[index as usize].as_deref()?;
        }
        current_node.value.as_ref()
    }

    pub fn bytes(&self) -> Bytes<'_, K, V> {
        Bytes::new(self)
    }
}
