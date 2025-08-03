use crate::{
    librarian::search::Node,
    trie::{Key, Trie, iter::Bytes},
};
use std::rc::Rc;

impl<'a, K: Key + 'a, V: 'a> Node<u8> for &'a Trie<K, V> {
    type Children = Bytes<'a, K, V>;

    fn children(&self) -> Self::Children {
        self.bytes()
    }
    fn is_leaf(&self) -> bool {
        self.value.is_some()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NestedNode<T, N: Node<T>> {
    root: Rc<N>,
    curr: N,
    parent: Option<Rc<NestedNode<T, N>>>,
    depth: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T, N: Node<T>> Clone for NestedNode<T, N> {
    fn clone(&self) -> Self {
        NestedNode {
            root: self.root.clone(),
            curr: self.curr.clone(),
            parent: self.parent.clone(),
            depth: self.depth,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, N: Node<T>> NestedNode<T, N> {
    /// Creates a new `NestedNode` with the given root and depth.
    ///
    /// The `depth` indicates how many times the tree can be traversed and nested.
    /// If the `depth` is 0, the, the tree will not be nested.
    pub fn new(root: N, depth: usize) -> Self {
        NestedNode {
            root: Rc::new(root.clone()),
            curr: root,
            parent: None,
            depth,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns an iterator of the nodes from the current node to the root.
    pub fn chain_rev(&self) -> impl Iterator<Item = &N> {
        let mut current = Some(self);
        std::iter::from_fn(move || {
            if let Some(node) = current {
                current = node.parent.as_deref();
                Some(&node.curr)
            } else {
                None
            }
        })
    }

    /// Returns a vector of the nodes from the root node to the current.
    pub fn chain(&self) -> Vec<&N> {
        let mut chain = self.chain_rev().collect::<Vec<_>>();
        chain.reverse();
        chain
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedNodeIter<T, N: Node<T>> {
    node: NestedNode<T, N>,
    children: N::Children,
}

impl<T, N: Node<T>> Iterator for NestedNodeIter<T, N> {
    type Item = (T, NestedNode<T, N>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.children.next() {
            Some((byte, child)) => {
                let node_new = NestedNode {
                    root: self.node.root.clone(),
                    curr: child,
                    parent: self.node.parent.clone(),
                    depth: self.node.depth,
                    _marker: std::marker::PhantomData,
                };
                Some((byte, node_new))
            }
            None if self.node.depth > 0 && self.node.is_leaf() => {
                self.node = NestedNode {
                    root: self.node.root.clone(),
                    curr: self.node.root.as_ref().clone(),
                    parent: Some(Rc::new(self.node.clone())),
                    depth: self.node.depth - 1,
                    _marker: std::marker::PhantomData,
                };
                self.children = self.node.curr.children();
                self.next()
            }
            _ => None,
        }
    }
}

impl<T, N: Node<T>> Node<T> for NestedNode<T, N> {
    type Children = NestedNodeIter<T, N>;

    fn children(&self) -> Self::Children {
        NestedNodeIter {
            node: self.clone(),
            children: self.curr.children(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.curr.is_leaf()
    }
}
