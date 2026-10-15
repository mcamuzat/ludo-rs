//! Games played with the Pairs deck. Rules and their sources: RULES.md.

pub mod auberge;
pub mod captains_orders;
pub mod continuous;
pub mod deadfall;
pub mod deck;
pub mod ghost_town;
pub mod goblin_poker;
pub mod kitty;
pub mod octopus;
pub mod pendleton;
pub mod port;
pub mod rocket;
pub mod starboard;
pub mod sweep;
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
pub use octopus::Octopus;
pub use pendleton::Pendleton;
pub use port::Port;
pub use rocket::Rocket;
pub use starboard::Starboard;
pub use sweep::Sweep;
pub use to_the_moon::ToTheMoon;
pub use zombie_life::ZombieLife;
