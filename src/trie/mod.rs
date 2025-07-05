pub mod iter;
mod keys;
mod node;
// mod refs;
#[cfg(test)]
mod test;

pub trait Key {
    fn as_bytes(&self) -> impl IntoIterator<Item = u8> + '_;
}

const CHILDREN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trie<K: Key, V> {
    pub value: Option<V>,
    // TODO: Option the whole array?
    children: [Option<Box<Trie<K, V>>>; CHILDREN],
    _marker: std::marker::PhantomData<K>,
}
