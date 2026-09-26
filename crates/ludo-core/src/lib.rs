//! The contract every game implements, and what the platform builds on it.

pub mod ai;
mod any;
pub mod conformance;
mod game;
mod log;

pub use any::{find, AnyGame, AnyMatch, GameEntry, LudoError};
pub use game::{Determinize, Game, GameMeta, Heuristics, Illegal, Outcome, Pending, Seat, Viewer};
pub use log::{replay, Match, MatchLog};

#[cfg(test)]
mod testgames;
