use std::rc::Rc;

use crate::librarian::search::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedNode<N: Node> {
    root: Rc<N>,
    curr: N,
    parent: Option<Rc<NestedNode<N>>>,
    depth: usize,
}

impl<N: Node> NestedNode<N> {
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
        }
    }

    /// Returns an iterator of the nodes from the current node to the root.
    pub fn chain(&self) -> impl Iterator<Item = &N> {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedNodeIter<N: Node> {
    node: NestedNode<N>,
    children: N::Children,
}

impl<N: Node> Iterator for NestedNodeIter<N> {
    type Item = (u8, NestedNode<N>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.children.next() {
            Some((byte, child)) => {
                let node_new = NestedNode {
                    root: self.node.root.clone(),
                    curr: child,
                    parent: self.node.parent.clone(),
                    depth: self.node.depth,
                };
                Some((byte, node_new))
            }
            None if self.node.depth > 0 && self.node.is_leaf() => {
                self.node = NestedNode {
                    root: self.node.root.clone(),
                    curr: self.node.root.as_ref().clone(),
                    parent: Some(Rc::new(self.node.clone())),
                    depth: self.node.depth - 1,
                };
                self.children = self.node.curr.children();
                self.next()
            }
            _ => None,
        }
    }
}

impl<N: Node> Node for NestedNode<N> {
    type Children = NestedNodeIter<N>;

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
