use super::{Key, Trie};

#[derive(Debug, Clone)]
pub struct Bytes<'a, K: Key, V> {
    base: &'a Trie<K, V>,
    bottom: u8,
    top: u8,
}
const _: () = assert!(
    super::CHILDREN <= (u8::MAX as usize + 1),
    "Trie children must fit within a u8"
);

impl<'a, K: Key, V> Bytes<'a, K, V> {
    pub fn new(trie: &'a Trie<K, V>) -> Self {
        Self {
            base: trie,
            bottom: 0u8.wrapping_sub(1),
            top: trie.children.len() as u8,
        }
    }
}

impl<'a, K: Key, V> Iterator for Bytes<'a, K, V> {
    type Item = (u8, &'a Trie<K, V>);

    fn next(&mut self) -> Option<Self::Item> {
        'start: loop {
            while self.top < self.base.children.len() as u8 {
                if let Some(ref child) = self.base.children[self.bottom as usize]
                    .as_ref()
                    .unwrap()
                    .children[self.top as usize]
                {
                    self.top += 1;
                    return Some(((self.bottom << 4) + (self.top - 1), child));
                } else {
                    self.top += 1;
                    continue;
                }
            }
            while self.bottom != (self.base.children.len() - 1) as u8 {
                self.bottom = self.bottom.wrapping_add(1);
                if self.base.children[self.bottom as usize].is_some() {
                    self.top = 0;
                    continue 'start;
                }
            }
            return None;
        }
    }
}

impl<K: Key, V> FromIterator<(K, V)> for Trie<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut trie = Trie::new();
        for (key, value) in iter {
            trie.insert(key, value);
        }
        trie
    }
}
