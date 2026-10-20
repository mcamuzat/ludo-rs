//! Cryptide (Hal Duncan and Ruth Veevers), sources and rules in RULES.md:
//! each player holds one clue about the creature's habitat; together the
//! clues name a single space of the map. Ask the others, then search.

pub mod map;

pub use map::{
    distance, random_layout, within, Animal, Board, Cell, Color, Kind, Layout, Mask, Square,
    Structure, Terrain, CELLS,
};
