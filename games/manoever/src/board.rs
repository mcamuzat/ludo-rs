//! The track: 84 squares and the 12 places of the barracks, read from the
//! rules (`scripts/manoever-board.py`) into `board_data.rs`, and the bonus
//! pieces at the squares the rules suggest.

use serde::{Deserialize, Serialize};

mod data {
    include!("board_data.rs");
}

pub use data::{BARRACKS, SQUARES};

/// A square of the track, 1 to 84.
pub type Square = u8;

/// The last square before the barracks.
pub const LAST: Square = 84;

/// Where a square or place is drawn: the two pages of the rules assembled,
/// page 6 above page 5 (842 × 1191, y downwards).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

pub const WIDTH: i16 = 842;
pub const HEIGHT: i16 = 1191;

/// A special bonus piece: white doubles, black triples [R].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disc {
    White,
    Black,
}

/// [R] Suggested setup, chosen by the user [U]: black on 12, 20, 47 and 65,
/// white on 8, 24, 31, 36, 60 and 74.
pub fn disc(sq: Square) -> Option<Disc> {
    match sq {
        12 | 20 | 47 | 65 => Some(Disc::Black),
        8 | 24 | 31 | 36 | 60 | 74 => Some(Disc::White),
        _ => None,
    }
}

/// [R] The gray pieces: square and value in points.
pub const GRAYS: [(Square, u8); 3] = [(25, 1), (40, 2), (55, 3)];

/// [R] The white squares at both ends of the track, where no bonus piece goes.
pub fn is_end_square(sq: Square) -> bool {
    matches!(sq, 1..=6 | 79..=84)
}

/// The board for the site to draw.
pub fn board_json() -> serde_json::Value {
    let discs: Vec<_> = (1..=LAST)
        .filter_map(|sq| disc(sq).map(|kind| serde_json::json!({ "square": sq, "kind": kind })))
        .collect();
    let grays: Vec<_> = GRAYS
        .iter()
        .map(|&(square, value)| serde_json::json!({ "square": square, "value": value }))
        .collect();
    let ends: Vec<Square> = (1..=LAST).filter(|&sq| is_end_square(sq)).collect();
    serde_json::json!({
        "width": WIDTH,
        "height": HEIGHT,
        "squares": &SQUARES[..],
        "barracks": &BARRACKS[..],
        "discs": discs,
        "grays": grays,
        "endSquares": ends,
    })
}
