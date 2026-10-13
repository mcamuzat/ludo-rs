//! Games played with the Pairs deck. Rules and their sources: RULES.md.

pub mod auberge;
pub mod continuous;
pub mod deck;
pub mod ghost_town;
pub mod testing;
pub mod zombie_life;

pub use auberge::Auberge;
pub use continuous::ContinuousPairs;
pub use ghost_town::GhostTown;
pub use zombie_life::ZombieLife;
