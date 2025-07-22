use super::Node;
use regex_automata::{
    dfa::Automaton,
    util::{primitives::StateID, start::Config},
};
use smallvec::{SmallVec, smallvec};
use std::{any::type_name_of_val, fmt::Debug};

#[derive(Debug)]
enum HeadPos<N: Node<u8>> {
    This(N),
    Children(N::Children),
}

struct Head<N: Node<u8>> {
    accepting: bool,
    state: StateID,
    pos: HeadPos<N>,
}

impl<N: Node<u8> + Debug> Debug for Head<N>
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

impl<N: Node<u8>> Head<N> {
    fn new(node: N, state: StateID) -> Self {
        Self {
            state,
            accepting: false,
            pos: HeadPos::This(node),
        }
    }
    fn accepting(node: N, state: StateID) -> Self {
        Self {
            state,
            accepting: true,
            pos: HeadPos::This(node),
        }
    }
}

pub struct MultiHeadDFA<'d, DFA: Automaton, N: Node<u8>> {
    dfa: &'d DFA,
    heads: SmallVec<[Head<N>; 32]>,
}

impl<DFA: Automaton, N: Node<u8> + Debug> Debug for MultiHeadDFA<'_, DFA, N>
where
    N::Children: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name_of_val(self))
            .field("heads", &self.heads)
            .finish()
    }
}

impl<'d, DFA: Automaton, N: Node<u8>> MultiHeadDFA<'d, DFA, N> {
    pub fn new(dfa: &'d DFA, node: N) -> Result<Self, regex_automata::dfa::StartError> {
        let first = Head::new(node, dfa.start_state(&Config::new())?);
        Ok(Self {
            dfa,
            heads: smallvec![first],
        })
    }
}

impl<DFA: Automaton, N: Node<u8>> Iterator for MultiHeadDFA<'_, DFA, N>
where
    Self: Debug,
    N: Debug,
{
    type Item = (N, StateID);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(head) = self.heads.last_mut() {
            if !head.accepting {
                match head.pos {
                    HeadPos::This(ref node) => {
                        let node = node.clone();
                        head.pos = HeadPos::Children(node.children());
                        if node.is_leaf() {
                            let state = self.dfa.next_eoi_state(head.state);
                            if self.dfa.is_match_state(state) {
                                return Some((node, state));
                            }
                        }
                    }
                    HeadPos::Children(ref mut children) => {
                        if let Some((byte, child)) = children.next() {
                            let state = self.dfa.next_state(head.state, byte);
                            if self.dfa.is_dead_state(state) {
                                continue;
                            }
                            if self.dfa.is_match_state(state) {
                                self.heads.push(Head::accepting(child, state));
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
                let state = head.state;
                match head.pos {
                    HeadPos::This(ref node) => {
                        let node = node.clone();
                        head.pos = HeadPos::Children(node.children());
                        if node.is_leaf() {
                            return Some((node.clone(), head.state));
                        }
                    }
                    HeadPos::Children(ref mut children) => {
                        if let Some((_, child)) = children.next() {
                            self.heads.push(Head::accepting(child, state));
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
