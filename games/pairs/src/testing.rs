//! Shorthands for tests (of this crate and of its users).

use crate::deck::Rank;

pub fn r(value: u8) -> Rank {
    Rank::new(value).expect("a rank from 1 to 10")
}

pub fn ranks(values: &[u8]) -> Vec<Rank> {
    values.iter().map(|&v| r(v)).collect()
}
