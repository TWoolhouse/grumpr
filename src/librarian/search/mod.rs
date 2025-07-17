mod nest;
pub use nest::NestedNode as Nest;
mod multi_dfa;
pub use multi_dfa::MultiHeadDFA;
mod nodes;
pub mod permutation;
pub mod query;

pub trait Node<T>: Clone {
    type Children: Iterator<Item = (T, Self)>;
    fn children(&self) -> Self::Children;
    fn is_leaf(&self) -> bool;
}
