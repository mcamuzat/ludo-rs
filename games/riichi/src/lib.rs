//! Japanese Riichi Mahjong, Mahjong Soul rules, on top of riichienv-core.
//! Rules and what ludo adds: RULES.md.

// Used by the Game impl (next commit).
#[allow(dead_code)]
mod action;
mod tile;

pub use action::Action;
pub use tile::Tile;
