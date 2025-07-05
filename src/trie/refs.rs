use super::{Key, Label, Leaf, Leaflet, Trie};
use std::{any::type_name_of_val, ops::Deref};

impl<'a, K: Key, V> Deref for Leaflet<'a, K, V> {
    type Target = Label<K, V>;
    fn deref(&self) -> &Self::Target {
        self.trie
            .label
            .as_ref()
            .expect("Leaflet must point to labelled trie node")
    }
}

impl<'a, K: Key, V> core::fmt::Debug for Leaflet<'a, K, V>
where
    K: core::fmt::Debug,
    K::Notch: core::fmt::Debug,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name_of_val(self))
            .field("trie", &self.trie)
            .finish()
    }
}

impl<'a, K: Key, V> TryFrom<&'a Trie<K, V>> for Leaflet<'a, K, V> {
    type Error = ();
    fn try_from(trie: &'a Trie<K, V>) -> Result<Self, Self::Error> {
        if trie.label.is_some() {
            Ok(Leaflet { trie })
        } else {
            Err(())
        }
    }
}

impl<'a, K: Key, V> Deref for Leaf<'a, K, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<'a, K: Key, V> core::fmt::Debug for Leaf<'a, K, V>
where
    K: core::fmt::Debug,
    K::Notch: core::fmt::Debug,
    V: core::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name_of_val(self))
            .field("trie", &self.leaflet.trie)
            .finish()
    }
}

impl<'a, K: Key, V> TryFrom<&'a Trie<K, V>> for Leaf<'a, K, V> {
    type Error = ();
    fn try_from(trie: &'a Trie<K, V>) -> Result<Self, Self::Error> {
        if trie
            .label
            .as_ref()
            .is_some_and(|label| label.value.is_some())
        {
            Ok(Leaf {
                leaflet: Leaflet { trie: trie },
            })
        } else {
            Err(())
        }
    }
}

impl<'a, K: Key, V> TryFrom<Leaflet<'a, K, V>> for Leaf<'a, K, V> {
    type Error = ();
    fn try_from(leaflet: Leaflet<'a, K, V>) -> Result<Self, Self::Error> {
        if leaflet.value.is_some() {
            Ok(Leaf { leaflet })
        } else {
            Err(())
        }
    }
}

impl<'a, K: Key, V> Leaf<'a, K, V> {
    pub fn value(&self) -> &V {
        self.leaflet
            .value
            .as_ref()
            .expect("Leaf must reference a node with a value")
    }
}
