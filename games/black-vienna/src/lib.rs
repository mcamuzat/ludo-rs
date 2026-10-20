//! Black Vienna (Gilbert Obermair), sources and rules in RULES.md: ask the
//! others about three suspects at a time, deduce who holds whom, and name
//! the three criminals set aside before anyone else.

pub mod material;

pub use material::{deal, hand_sizes, suspects, CardId, Deal, Suspect, INVESTIGATIONS};
