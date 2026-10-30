//! Card games, most of them played with a French-suited pack (`deck.rs`);
//! Hattrick and David & Goliath have their own. Rules and their sources: RULES.md.

pub mod collusion;
pub mod david_goliath;
pub mod deck;
pub mod hattrick;
pub mod lamarckian;
pub mod president;

pub use collusion::Collusion;
pub use david_goliath::DavidGoliath;
pub use hattrick::Hattrick;
pub use lamarckian::Lamarckian;
pub use president::President;

/// Ranks from scores (0: first); equal scores share a rank.
pub fn standings<T: PartialOrd + Copy>(scores: &[T]) -> Vec<u8> {
    scores
        .iter()
        .map(|&mine| scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
        .collect()
}
