//! Wu Hsing (Sid Sackson, « Domino Bead Game ») : 2 à 4 joueurs posent des
//! dominos de valeurs 1 à 5 en prolongeant des cycles de couleurs. Règles et
//! source : RULES.md, où [R] est la règle française.

pub mod deck;

pub use deck::{deal, is_double, values, Deal, DOMINOES};
