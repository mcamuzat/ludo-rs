//! The city: six sections around the Factory, and which neighborhoods
//! touch. Same assembly and same test as `scripts/chocolatier-board.py`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::board_data::{HoodData, FACTORY_GRID, GRID_H, GRID_W, MAX_GAP, MIN_FACE, SECTIONS};

/// Size of a section, in pixels at 100 dpi (8.25 x 5.5 in).
pub const SECTION_W: u32 = 825;
pub const SECTION_H: u32 = 550;
/// The whole city: two upright sections on each side of a column of three.
pub const CITY_W: u32 = 2 * SECTION_H + SECTION_W;
pub const CITY_H: u32 = 2 * SECTION_W;
/// Where the Factory lies (never turned).
pub const FACTORY_AT: (u32, u32) = (SECTION_H, SECTION_H);

/// [R] "Arrange the 7 board sections in a square, with the Factory piece in
/// the middle": top-left, bottom-left, top and bottom of the middle column,
/// top-right, bottom-right. Side slots hold upright sections.
pub const SLOTS: [(u32, u32); 6] = [
    (0, 0),
    (0, SECTION_W),
    (SECTION_H, 0),
    (SECTION_H, 2 * SECTION_H),
    (SECTION_H + SECTION_W, 0),
    (SECTION_H + SECTION_W, SECTION_W),
];

pub fn is_side(slot: usize) -> bool {
    !matches!(slot, 2 | 3)
}

/// The section in a slot, turned a number of quarters clockwise (odd in
/// the side slots, even in the middle).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub section: u8,
    pub quarters: u8,
}

/// A neighborhood of the six sections, numbered section after section.
pub struct Hood {
    pub section: u8,
    /// Its rank within its section (the order of `board.json`).
    pub index: u8,
    pub data: &'static HoodData,
    /// Its houses: `first..first + data.prices.len()`.
    pub first: u8,
}

pub struct Tables {
    pub hoods: Vec<Hood>,
    /// Each house's neighborhood.
    pub house_hood: Vec<u8>,
    /// Each section's grid, rows of cells ('.' street, else hood id + 1).
    grids: Vec<Vec<u8>>,
}

pub fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut hoods = Vec::new();
        let mut house_hood = Vec::new();
        let mut grids = Vec::new();
        for (s, section) in SECTIONS.iter().enumerate() {
            let base = hoods.len() as u8;
            for (i, data) in section.hoods.iter().enumerate() {
                let id = hoods.len() as u8;
                hoods.push(Hood {
                    section: s as u8,
                    index: i as u8,
                    data,
                    first: house_hood.len() as u8,
                });
                house_hood.extend(std::iter::repeat_n(id, data.prices.len()));
            }
            grids.push(parse_grid(section.grid, |c| match c {
                b'.' => 0,
                c => base + (c - b'a') + 1,
            }));
        }
        Tables {
            hoods,
            house_hood,
            grids,
        }
    })
}

fn parse_grid(text: &str, code: impl Fn(u8) -> u8) -> Vec<u8> {
    let cells: Vec<u8> = text
        .lines()
        .flat_map(|row| row.bytes().map(&code))
        .collect();
    assert_eq!(cells.len(), GRID_W * GRID_H, "a section grid is complete");
    cells
}

/// Code of the Factory on the assembled grid.
pub const FACTORY: u8 = u8::MAX;

/// The city on one grid (cells of 5 px): 0 street, hood id + 1, `FACTORY`.
fn assemble(layout: &[Placement; 6]) -> (Vec<u8>, usize, usize) {
    let t = tables();
    let (w, h) = (GRID_H * 2 + GRID_W, GRID_W * 2);
    let mut city = vec![0u8; w * h];
    let cell = |px: u32| (px / 5) as usize;
    for (slot, p) in layout.iter().enumerate() {
        let grid = &t.grids[usize::from(p.section)];
        let (ox, oy) = (cell(SLOTS[slot].0), cell(SLOTS[slot].1));
        let (pw, ph) = if p.quarters % 2 == 1 {
            (GRID_H, GRID_W)
        } else {
            (GRID_W, GRID_H)
        };
        for py in 0..ph {
            for px in 0..pw {
                // The local cell shown at (px, py) once turned clockwise.
                let (lx, ly) = match p.quarters % 4 {
                    0 => (px, py),
                    1 => (py, GRID_H - 1 - px),
                    2 => (GRID_W - 1 - px, GRID_H - 1 - py),
                    _ => (GRID_W - 1 - py, px),
                };
                city[(oy + py) * w + ox + px] = grid[ly * GRID_W + lx];
            }
        }
    }
    let factory = parse_grid(FACTORY_GRID, |c| if c == b'.' { 0 } else { FACTORY });
    let (fx, fy) = (cell(FACTORY_AT.0), cell(FACTORY_AT.1));
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = factory[y * GRID_W + x];
            if c != 0 {
                city[(fy + y) * w + fx + x] = c;
            }
        }
    }
    (city, w, h)
}

/// [R] "To touch, two neighborhoods (or a neighborhood and the Factory)
/// must be directly across the street from each other, not just diagonally
/// across an intersection": facing each other across a street at most
/// `MAX_GAP` cells wide, along at least `MIN_FACE` rows or columns.
/// Returns the pairs that touch (codes as on the grid, smaller first).
pub fn touching_pairs(layout: &[Placement; 6]) -> Vec<(u8, u8)> {
    let (city, w, h) = assemble(layout);
    let mut counts: BTreeMap<(u8, u8), usize> = BTreeMap::new();
    let mut scan = |line: &mut dyn Iterator<Item = u8>| {
        let (mut last, mut gap) = (0u8, 0usize);
        for c in line {
            if c == 0 {
                gap += 1;
                continue;
            }
            if last != 0 && last != c && gap > 0 && gap <= MAX_GAP {
                *counts.entry((last.min(c), last.max(c))).or_default() += 1;
            }
            last = c;
            gap = 0;
        }
    };
    for y in 0..h {
        scan(&mut (0..w).map(|x| city[y * w + x]));
    }
    for x in 0..w {
        scan(&mut (0..h).map(|y| city[y * w + x]));
    }
    counts
        .into_iter()
        .filter(|&(_, n)| n >= MIN_FACE)
        .map(|(pair, _)| pair)
        .collect()
}

/// For each hood: whether it touches the Factory, and the hoods it touches.
pub fn neighbors(layout: &[Placement; 6]) -> (Vec<bool>, Vec<Vec<u8>>) {
    let n = tables().hoods.len();
    let mut factory = vec![false; n];
    let mut touching = vec![Vec::new(); n];
    for (a, b) in touching_pairs(layout) {
        if b == FACTORY {
            factory[usize::from(a - 1)] = true;
        } else {
            touching[usize::from(a - 1)].push(b - 1);
            touching[usize::from(b - 1)].push(a - 1);
        }
    }
    (factory, touching)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The layout of the script's control picture: sections in page order,
    /// middle ones unturned, side ones a quarter clockwise.
    fn control() -> [Placement; 6] {
        std::array::from_fn(|slot| Placement {
            section: slot as u8,
            quarters: if is_side(slot) { 1 } else { 0 },
        })
    }

    #[test]
    fn the_survey_has_forty_houses_of_each_ingredient() {
        let t = tables();
        assert_eq!(t.hoods.len(), 46);
        assert_eq!(t.house_hood.len(), 199);
        let mut by_color = BTreeMap::new();
        for hood in &t.hoods {
            *by_color.entry(hood.data.color).or_insert(0) += hood.data.prices.len();
        }
        assert_eq!(by_color.get(&None), Some(&39));
        assert!(by_color
            .iter()
            .filter(|(c, _)| c.is_some())
            .all(|(_, &n)| n == 40));
    }

    #[test]
    fn the_control_layout_has_the_links_of_the_script() {
        assert_eq!(touching_pairs(&control()).len(), 107);
    }

    #[test]
    fn neighbors_are_symmetric_and_some_touch_the_factory() {
        let (factory, touching) = neighbors(&control());
        assert!(factory.iter().filter(|&&f| f).count() >= 8);
        for (a, list) in touching.iter().enumerate() {
            for &b in list {
                assert!(touching[usize::from(b)].contains(&(a as u8)));
            }
        }
    }
}
