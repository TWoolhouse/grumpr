use super::{Key, Leaf, Leaflet, Trie};
use smallvec::{smallvec, SmallVec};
use std::{any::type_name_of_val, iter::Flatten};

pub(super) type Children<'a, K, V> = Flatten<core::slice::Iter<'a, Option<Box<Trie<K, V>>>>>;

pub struct Leaflets<'a, K: Key, V> {
    stack: SmallVec<[Children<'a, K, V>; 4]>,
}

impl<'a, K: Key + std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for Leaflets<'a, K, V>
where
    K::Notch: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name_of_val(self))
            .field("stack", &self.stack)
            .finish()
    }
}

impl<'a, K: Key, V> Leaflets<'a, K, V> {
    pub fn new(root: &'a Trie<K, V>) -> Self {
        Self {
            stack: smallvec![root.children()],
        }
    }
}

impl<'a, K: Key, V> Iterator for Leaflets<'a, K, V> {
    type Item = Leaflet<'a, K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(children) = self.stack.last_mut() {
            if let Some(child) = children.next() {
                let leaflet = child.try_as_leaflet();
                if leaflet.is_some() {
                    return leaflet;
                }
                self.stack.push(child.children());
            } else {
                self.stack.pop();
            }
        }
        None
    }
}

pub struct Leaves<'a, K: Key, V> {
    stack: SmallVec<[Children<'a, K, V>; 4]>,
}

impl<'a, K: Key + std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for Leaves<'a, K, V>
where
    K::Notch: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name_of_val(self))
            .field("stack", &self.stack)
            .finish()
    }
}

impl<'a, K: Key, V> Leaves<'a, K, V> {
    pub fn new(root: &'a Trie<K, V>) -> Self {
        Self {
            stack: smallvec![root.children()],
        }
    }
}

impl<'a, K: Key, V> Iterator for Leaves<'a, K, V> {
    type Item = Leaf<'a, K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(children) = self.stack.last_mut() {
            if let Some(child) = children.next() {
                let leaf = child.try_as_leaf();
                if leaf.is_some() {
                    return leaf;
                }
                self.stack.push(child.children());
            } else {
                self.stack.pop();
            }
        }
        None
    }
}
