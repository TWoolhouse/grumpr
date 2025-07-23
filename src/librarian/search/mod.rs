pub mod automata;
mod multi_dfa;
pub use multi_dfa::MultiHeadDFA;
mod node;
pub use node::NestedNode as Nest;
pub mod query;

pub trait Node<T>: Clone {
    type Children: Iterator<Item = (T, Self)>;
    fn children(&self) -> Self::Children;
    fn is_leaf(&self) -> bool;
}
