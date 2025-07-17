use itertools::Itertools;
use regex::Regex;
use regex_automata::nfa::thompson::Transition;

use crate::librarian::Result;

pub fn dfa_exact(string: &str) -> Result<regex_automata::dfa::dense::DFA<Vec<u32>>> {
    let mut builder = regex_automata::nfa::thompson::Builder::new();
    builder.start_pattern()?;

    let state_start = builder.add_union(Vec::with_capacity(string.len()))?;
    let state_match = builder.add_match()?;

    for perm in string.chars().permutations(string.len()) {
        let perm = perm.into_iter().collect::<String>();
        let mut next = state_match;
        for c in perm.bytes().rev() {
            let state = builder.add_range(Transition {
                start: c,
                end: c,
                next,
            })?;
            next = state;
        }
        builder.patch(state_start, next)?;
    }

    builder.finish_pattern(state_start)?;

    let nfa = builder.build(state_start, state_start)?;
    let dfa = regex_automata::dfa::dense::Builder::new().build_from_nfa(&nfa)?;

    Ok(dfa)
}

pub fn dfa_partial(
    string: &str,
    _wildcards: usize,
) -> Result<regex_automata::dfa::dense::DFA<Vec<u32>>> {
    let mut pattern = String::with_capacity(string.as_bytes().len() + 16);

    pattern.push_str(r"^[");
    pattern.push_str(string);
    pattern.push(']');
    pattern.push_str(&format!("{{{}}}", string.len()));
    pattern.push('$');

    dbg!(&pattern);
    dbg!(Regex::new(&pattern).unwrap());

    let nfa = regex_automata::nfa::thompson::NFA::new(&pattern)?;
    let dfa = regex_automata::dfa::dense::Builder::new().build_from_nfa(&nfa)?;
    Ok(dfa)
}
