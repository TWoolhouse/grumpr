use super::{
    iter::{Children, Leaflets, Leaves},
    keys::NotchNibbles,
    Key, Leaf, Leaflet, Trie,
};
use std::any::type_name;
use std::borrow::Borrow;

impl<K: Key, V> core::fmt::Debug for Trie<K, V>
where
    K: core::fmt::Debug,
    K::Notch: core::fmt::Debug,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Trie<K, V>>())
            .field("label", &self.label)
            .field("children", &self.children)
            .finish()
    }
}

impl<K: Key, V> Default for Trie<K, V> {
    fn default() -> Self {
        Trie {
            label: None,
            children: Default::default(),
        }
    }
}

impl<K: Key, V> Trie<K, V> {
    pub fn new() -> Self {
        Trie::default()
    }

    pub fn try_as_leaflet(&self) -> Option<Leaflet<K, V>> {
        self.try_into().ok()
    }
    pub fn try_as_leaf(&self) -> Option<Leaf<K, V>> {
        self.try_into().ok()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut current_node = self;
        for notch in key.notches() {
            for index in notch.as_nibbles().into_iter() {
                if let Some(ref mut next) = current_node.children[index as usize] {
                    current_node = next;
                } else {
                    current_node.children[index as usize] = Some(Box::new(Trie {
                        label: None,
                        children: Default::default(),
                    }));
                    current_node = current_node.children[index as usize]
                        .as_deref_mut()
                        .unwrap();
                }
            }
            if let Some(ref mut leaf) = current_node.label {
                leaf.notch = notch;
            } else {
                current_node.label = Some(Box::new(super::Label { value: None, notch }));
            }
        }
        current_node
            .label
            .as_mut()
            .expect("Leaf should always be present after inserting segments")
            .value
            .replace(value)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Key<Notch = K::Notch> + ?Sized,
    {
        let mut current_node = self;
        for segment in key.notches() {
            for index in segment.as_nibbles() {
                current_node = current_node.children[index as usize].as_deref()?;
            }
        }
        current_node
            .label
            .as_ref()
            .and_then(|leaf| leaf.value.as_ref())
    }

    pub(super) fn children(&self) -> Children<'_, K, V> {
        self.children.iter().flatten()
    }
    pub fn leaflets(&self) -> Leaflets<'_, K, V> {
        Leaflets::new(self)
    }
    pub fn leaves(&self) -> Leaves<'_, K, V> {
        Leaves::new(self)
    }
}
