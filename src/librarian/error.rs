use regex_automata::{
    dfa::{StartError, dense},
    nfa::thompson,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DFA(Box<dense::BuildError>),
    #[error(transparent)]
    NFA(Box<thompson::BuildError>),
    #[error("Failed to start DFA: {0}")]
    DFASearch(#[from] StartError),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error("Failed to find any grams up to {0} differences from the pattern")]
    NoNearest(u8),
}

impl From<dense::BuildError> for Error {
    fn from(value: dense::BuildError) -> Self {
        Error::DFA(Box::new(value))
    }
}

impl From<thompson::BuildError> for Error {
    fn from(value: thompson::BuildError) -> Self {
        Error::NFA(Box::new(value))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
