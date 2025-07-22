use crate::librarian::Result;
use regex_automata::{
    PatternID,
    dfa::{Automaton, dense::DFA},
    nfa::thompson::{Builder, Transition},
    util::{look::Look, primitives::StateID},
};
use regex_syntax::utf8::Utf8Sequences;
use std::collections::HashSet;

pub fn automata(
    pattern: &str,
    distances: impl IntoIterator<Item = u8>,
) -> Result<(DFA<Vec<u32>>, impl Fn(&DFA<Vec<u32>>, StateID) -> u8)> {
    let distances: HashSet<u8> = distances.into_iter().collect();

    let mut builder = Builder::new();
    // Map between the pattern ID and the distance it represents
    let mut patterns = Vec::with_capacity(distances.len());
    let state_start = builder.add_empty()?;

    let (mut layer_prev, pattern_id) = nfa_layer(
        &mut builder,
        pattern,
        distances.contains(&0).then_some(state_start),
    )?;
    builder.patch(state_start, layer_prev[0])?;
    if let Some(pattern_id) = pattern_id {
        // PatternIDs increment from 0, so the first pattern ID is always 0
        debug_assert_eq!(pattern_id.as_usize(), patterns.len());
        patterns.push(0u8);
    }

    for distance in 1..=(distances.iter().max().copied().unwrap_or(0)) {
        let (layer, pattern_id) = nfa_layer(
            &mut builder,
            pattern,
            distances.contains(&distance).then_some(state_start),
        )?;
        let mut it = layer_prev.iter().zip(layer.iter()).peekable();
        while let Some((&prev, &curr)) = it.next() {
            let (start, end) =
                build_utf8_sequences(&mut builder, Utf8Sequences::new(char::MIN, char::MAX))?;

            // Patch up the graph via any char (add)
            builder.patch(prev, start)?;
            builder.patch(end, curr)?;
            // Patch up & right
            if let Some(&(_, &next)) = it.peek() {
                // via epsilon (delete)
                builder.patch(end, next)?;
                // via any char (replace)
                builder.patch(prev, next)?;
            }
        }

        if let Some(pattern_id) = pattern_id {
            debug_assert_eq!(pattern_id.as_usize(), patterns.len());
            patterns.push(distance);
        }

        layer_prev = layer;
    }

    let nfa = builder.build(state_start, state_start)?;
    let dfa = regex_automata::dfa::dense::Builder::new().build_from_nfa(&nfa)?;

    Ok((dfa, move |dfa: &DFA<Vec<u32>>, state_id: StateID| {
        patterns[dfa.match_pattern(state_id, 0).as_usize()]
    }))
}

fn nfa_layer(
    builder: &mut Builder,
    pattern: &str,
    match_start_state: Option<StateID>,
) -> Result<(Vec<StateID>, Option<PatternID>)> {
    let state_end = builder.add_union(Vec::default())?;
    let pattern_id = if let Some(state_start) = match_start_state {
        let pattern_id = builder.start_pattern()?;
        let state_match = builder.add_match()?;
        builder.finish_pattern(state_start)?;
        let end = builder.add_look(state_match, Look::End)?;
        builder.patch(state_end, end)?;
        Some(pattern_id)
    } else {
        None
    };

    let mut states = Vec::with_capacity(pattern.len());

    let mut next = state_end;
    for c in pattern.chars().rev() {
        let (start, end) = build_utf8_sequences(builder, Utf8Sequences::new(c, c))?;
        builder.patch(end, next)?;
        states.push(next);
        next = builder.add_union(vec![start])?;
    }
    states.push(next);
    states.reverse();

    Ok((states, pattern_id))
}

fn build_utf8_sequences(
    builder: &mut Builder,
    sequences: Utf8Sequences,
) -> Result<(StateID, StateID)> {
    let state_end = builder.add_empty()?;

    let mut transitions = Vec::new();
    for sequence in sequences {
        let start = sequence
            .into_iter()
            .rev()
            .fold(Ok(state_end), |next, range| match next {
                Ok(next) => builder.add_range(Transition {
                    start: range.start,
                    end: range.end,
                    next,
                }),
                x => x,
            })?;
        transitions.push(start);
    }

    let state_start = builder.add_union(transitions)?;
    Ok((state_start, state_end))
}
