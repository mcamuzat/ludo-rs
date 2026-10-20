//! Cryptide (Hal Duncan and Ruth Veevers), sources and rules in RULES.md:
//! each player holds one clue about the creature's habitat; together the
//! clues name a single space of the map. Ask the others, then search.

pub mod clues;
mod determinize;
mod heuristics;
pub mod knowledge;
pub mod map;
mod rules;
pub mod testing;
mod view;

pub use clues::{generate, habitat, hints, Clue, Hint, HintTable, Mode, Puzzle};
pub use heuristics::ideal_cube_share;
pub use map::{
    distance, random_layout, within, Animal, Board, Cell, Color, Kind, Layout, Mask, Square,
    Structure, Terrain, CELLS,
};
pub use rules::{Action, Answer, Config, Cryptide, Event, Phase, State, TableCache, Tokens, Why};
pub use view::{Reveal, View};
