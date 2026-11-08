//! The map of the 1997 board and the regions formed on it.

use crate::Piece;

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 11;
pub const CELLS: usize = WIDTH * HEIGHT;

/// The board, row by row from the top: `~` river, `T` a temple with a
/// treasure at the start, `C` the same on one of the four corner squares
/// whose treasure must be taken first, `.` land. Not given by [R]: read off
/// the 1997 board ❓ (to check against the box).
const MAP: [&str; HEIGHT] = [
    "....~~~~~.T.~...",
    ".C..~.......~..C",
    "...~~T......~~..",
    "~~~~.........~~~",
    ".............T~~",
    "..............~.",
    "~~~~....T...~~~.",
    ".C.~~~~.....~...",
    "......~~~~~~~.C.",
    ".....T..........",
    "..........T.....",
];

fn square(cell: u8) -> u8 {
    let cell = usize::from(cell);
    MAP[cell / WIDTH].as_bytes()[cell % WIDTH]
}

pub fn is_river(cell: u8) -> bool {
    square(cell) == b'~'
}

/// [R] The ten squares marked with a winged sphinx: a temple and a
/// treasure at the start.
pub fn starts_with_temple(cell: u8) -> bool {
    matches!(square(cell), b'T' | b'C')
}

/// [R] The four treasures near the corners, taken first.
pub fn is_corner(cell: u8) -> bool {
    square(cell) == b'C'
}

/// The squares sharing a side with `cell` ([R] "adjacent").
pub fn neighbors(cell: u8) -> impl Iterator<Item = u8> {
    let c = usize::from(cell);
    let (x, y) = (c % WIDTH, c / WIDTH);
    [
        (y > 0).then(|| c - WIDTH),
        (y + 1 < HEIGHT).then(|| c + WIDTH),
        (x > 0).then(|| c - 1),
        (x + 1 < WIDTH).then(|| c + 1),
    ]
    .into_iter()
    .flatten()
    .map(|n| n as u8)
}

/// The top-left squares of the 2 × 2 squares that contain `cell`.
pub fn squares_with(cell: u8) -> impl Iterator<Item = u8> {
    let c = usize::from(cell);
    let (x, y) = (c % WIDTH, c / WIDTH);
    [(0, 0), (1, 0), (0, 1), (1, 1)]
        .into_iter()
        .filter(move |&(dx, dy)| x >= dx && y >= dy && x - dx + 1 < WIDTH && y - dy + 1 < HEIGHT)
        .map(move |(dx, dy)| (c - dx - dy * WIDTH) as u8)
}

/// The four cells of the 2 × 2 square whose top-left cell is `top_left`.
pub fn square_cells(top_left: u8) -> [u8; 4] {
    let w = WIDTH as u8;
    [top_left, top_left + 1, top_left + w, top_left + w + 1]
}

/// No region.
pub const NONE: u8 = u8::MAX;

/// [R] Tiles and leaders sharing a side form a region; a catastrophe
/// breaks the link. Each cell's region number, `NONE` for empty squares,
/// catastrophes and `skip` (a leader being moved, the junction tile).
pub fn regions(cells: &[Option<Piece>], skip: Option<u8>) -> Vec<u8> {
    let links = |c: usize| {
        Some(c as u8) != skip && matches!(cells[c], Some(Piece::Tile { .. } | Piece::Leader { .. }))
    };
    let mut label = vec![NONE; CELLS];
    let mut next = 0u8;
    let mut stack = Vec::new();
    for start in 0..CELLS {
        if label[start] != NONE || !links(start) {
            continue;
        }
        label[start] = next;
        stack.push(start as u8);
        while let Some(c) = stack.pop() {
            for n in neighbors(c) {
                let n = usize::from(n);
                if label[n] == NONE && links(n) {
                    label[n] = next;
                    stack.push(n as u8);
                }
            }
        }
        next += 1;
    }
    label
}

/// Which regions are kingdoms ([R] a region with at least one leader),
/// indexed by region number.
pub fn kingdoms(cells: &[Option<Piece>], label: &[u8]) -> Vec<bool> {
    let mut is = vec![false; CELLS];
    for (c, piece) in cells.iter().enumerate() {
        if matches!(piece, Some(Piece::Leader { .. })) && label[c] != NONE {
            is[usize::from(label[c])] = true;
        }
    }
    is
}

/// How many distinct kingdoms are next to `cell` (no allocation).
pub fn kingdom_count_next_to(cell: u8, label: &[u8], kingdom: &[bool]) -> usize {
    let mut found = [NONE; 4];
    let mut n = 0;
    for c in neighbors(cell) {
        let l = label[usize::from(c)];
        if l != NONE && kingdom[usize::from(l)] && !found[..n].contains(&l) {
            found[n] = l;
            n += 1;
        }
    }
    n
}

/// The distinct kingdoms next to `cell`.
pub fn kingdoms_next_to(cell: u8, label: &[u8], kingdom: &[bool]) -> Vec<u8> {
    let mut found = Vec::with_capacity(4);
    for n in neighbors(cell) {
        let l = label[usize::from(n)];
        if l != NONE && kingdom[usize::from(l)] && !found.contains(&l) {
            found.push(l);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_has_ten_temples_four_of_them_corners() {
        let cells = 0..CELLS as u8;
        assert_eq!(cells.clone().filter(|&c| starts_with_temple(c)).count(), 10);
        assert_eq!(cells.clone().filter(|&c| is_corner(c)).count(), 4);
        assert!(cells
            .filter(|&c| starts_with_temple(c))
            .all(|c| !is_river(c)));
        assert!(MAP.iter().all(|row| row.len() == WIDTH));
    }

    #[test]
    fn squares_stay_on_the_board() {
        assert_eq!(squares_with(0).collect::<Vec<_>>(), vec![0]);
        assert_eq!(squares_with(17).count(), 4);
        assert_eq!(
            squares_with((CELLS - 1) as u8).collect::<Vec<_>>(),
            vec![(CELLS - 18) as u8]
        );
    }
}
