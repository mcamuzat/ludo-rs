//! The contract every game implements, and what the platform builds on it.

mod game;

pub use game::{Determinize, Game, GameMeta, Heuristics, Illegal, Outcome, Pending, Seat, Viewer};

#[cfg(test)]
mod testgames;
