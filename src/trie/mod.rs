pub mod iter;
mod keys;
mod node;
mod refs;
#[cfg(test)]
mod test;

pub trait Key {
    type Notch: Notch;
    fn notches(&self) -> impl IntoIterator<Item = Self::Notch>;
}
pub trait Notch {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_;
}

const CHILDREN: usize = 16;

pub struct Trie<K: Key, V> {
    pub label: Option<Box<Label<K, V>>>,
    children: [Option<Box<Trie<K, V>>>; CHILDREN],
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label<K: Key, V> {
    pub value: Option<V>,
    pub notch: K::Notch,
}

/// A reference to a node, which has a notch and may contain a value.
#[derive(Clone)]
pub struct Leaflet<'a, K: Key, V> {
    pub trie: &'a Trie<K, V>,
}

/// A reference to leaflet, that definitely contains a value.
#[derive(Clone)]
pub struct Leaf<'a, K: Key, V> {
    pub leaflet: Leaflet<'a, K, V>,
}
