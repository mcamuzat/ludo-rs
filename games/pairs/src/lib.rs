//! Games played with the Pairs deck. Rules and their sources: RULES.md.

pub mod auberge;
pub mod captains_orders;
pub mod continuous;
pub mod deadfall;
pub mod deck;
pub mod ghost_town;
pub mod goblin_poker;
pub mod kitty;
pub mod pendleton;
pub mod testing;
pub mod to_the_moon;
pub mod zombie_life;

pub use auberge::Auberge;
pub use captains_orders::CaptainsOrders;
pub use continuous::ContinuousPairs;
pub use deadfall::Deadfall;
pub use ghost_town::GhostTown;
pub use goblin_poker::GoblinPoker;
pub use kitty::Kitty;
pub use pendleton::Pendleton;
pub use to_the_moon::ToTheMoon;
pub use zombie_life::ZombieLife;
