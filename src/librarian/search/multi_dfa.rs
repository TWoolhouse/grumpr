use super::Node;
use regex_automata::{
    dfa::Automaton,
    util::{primitives::StateID, start::Config},
};
use smallvec::{smallvec, SmallVec};
use std::{any::type_name_of_val, fmt::Debug};

#[derive(Debug)]
enum HeadPos<N: Node> {
    This(N),
    Children(N::Children),
}

struct Head<N: Node> {
    state: Option<StateID>,
    pos: HeadPos<N>,
}

impl<N: Node + Debug> Debug for Head<N>
where
    N::Children: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name_of_val(self))
            .field("state", &self.state)
            .field("pos", &self.pos)
            .finish()
    }
}

impl<N: Node> Head<N> {
    fn new(node: N, state: StateID) -> Self {
        Self {
            state: Some(state),
            pos: HeadPos::This(node),
        }
    }
    fn accepting(node: N) -> Self {
        Self {
            state: None,
            pos: HeadPos::This(node),
        }
    }
}

pub struct MultiHeadDFA<'d, DFA: Automaton, N: Node> {
    dfa: &'d DFA,
    heads: SmallVec<[Head<N>; 32]>,
}

impl<'d, DFA: Automaton, N: Node + Debug> Debug for MultiHeadDFA<'d, DFA, N>
where
    N::Children: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name_of_val(self))
            .field("heads", &self.heads)
            .finish()
    }
}

impl<'d, DFA: Automaton, N: Node> MultiHeadDFA<'d, DFA, N> {
    pub fn new(dfa: &'d DFA, node: N) -> Result<Self, regex_automata::dfa::StartError> {
        let first = Head::new(node, dfa.start_state(&Config::new())?);
        Ok(Self {
            dfa,
            heads: smallvec![first],
        })
    }
}

impl<'d, DFA: Automaton, N: Node> Iterator for MultiHeadDFA<'d, DFA, N>
where
    Self: Debug,
    N: Debug,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(head) = self.heads.last_mut() {
            if let Some(state) = head.state {
                match head.pos {
                    HeadPos::This(ref node) => {
                        let node = node.clone();
                        head.pos = HeadPos::Children(node.children());
                        if node.is_leaf() && self.dfa.is_match_state(self.dfa.next_eoi_state(state))
                        {
                            return Some(node);
                        }
                    }
                    HeadPos::Children(ref mut children) => {
                        if let Some((byte, child)) = children.next() {
                            let state = self.dfa.next_state(state, byte);
                            if self.dfa.is_dead_state(state) {
                                continue;
                            }
                            if self.dfa.is_match_state(state) {
                                self.heads.push(Head::accepting(child));
                            } else {
                                self.heads.push(Head::new(child, state));
                            }
                        } else {
                            // No more children, pop the head.
                            self.heads.pop();
                        }
                    }
                }
            } else {
                match head.pos {
                    HeadPos::This(ref node) => {
                        let node = node.clone();
                        head.pos = HeadPos::Children(node.children());
                        if node.is_leaf() {
                            return Some(node.clone());
                        }
                    }
                    HeadPos::Children(ref mut children) => {
                        if let Some((_, child)) = children.next() {
                            self.heads.push(Head::accepting(child));
                        } else {
                            self.heads.pop();
                        }
                    }
                }
                {}
            }
        }
        None
    }
}
