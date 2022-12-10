use crate::grammar::consts::{
    Terminal,
    NonTerminal,
};

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum Letter {
    NonTerminal(NonTerminal),
    Terminal(Terminal),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Production {
    pub lhs: NonTerminal,
    pub rhs: Vec<Letter>,
}

impl Production {
    pub fn get_nth_if_non_terminal(letters: &Vec<Letter>, n: usize) -> Option<&NonTerminal> {
        if n >= letters.len() {
            return None;
        }

        match &letters[n] {
            Letter::NonTerminal(idx) => Some(idx),
            _ => None,
        }
    }

    pub fn get_nth(letters: &Vec<Letter>, n: usize) -> Option<&Letter> {
        if n >= letters.len() {
            return None;
        }

        Some(&letters[n])
    }
}