//! The map [R p. 4]: six sections of 6 × 3 spaces, two side by side in
//! each of three bands, 12 columns × 9 rows in all. Even columns sit half a
//! space higher than odd ones ([J] `VALID_HEXES`, [B] `dcounter.py`).

use std::ops::{BitAnd, BitOr, Not};
use std::sync::OnceLock;

use rand::Rng;
use serde::{Deserialize, Serialize};

pub const ROWS: u8 = 9;
pub const COLS: u8 = 12;
pub const CELLS: u8 = ROWS * COLS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Terrain {
    Lake,
    Mountain,
    Forest,
    Swamp,
    Desert,
}

impl Terrain {
    /// [R p. 4], in the order of [J] `CLUE_DATA`: forest, desert, water,
    /// swamp, mountain.
    pub const ALL: [Terrain; 5] = [
        Terrain::Forest,
        Terrain::Desert,
        Terrain::Lake,
        Terrain::Swamp,
        Terrain::Mountain,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Animal {
    Bear,
    Cougar,
}

impl Animal {
    pub const ALL: [Animal; 2] = [Animal::Bear, Animal::Cougar];
}

/// [R p. 4] Standing stone or abandoned shack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Stone,
    Shack,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Stone, Kind::Shack];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
    White,
    Green,
    Blue,
    Black,
}

impl Color {
    pub const ALL: [Color; 4] = [Color::White, Color::Green, Color::Blue, Color::Black];
}

/// A space: `row * 12 + col`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cell(pub u8);

impl Cell {
    pub fn new(row: u8, col: u8) -> Option<Cell> {
        (row < ROWS && col < COLS).then_some(Cell(row * COLS + col))
    }

    pub fn row(self) -> u8 {
        self.0 / COLS
    }

    pub fn col(self) -> u8 {
        self.0 % COLS
    }

    pub fn index(self) -> usize {
        usize::from(self.0)
    }

    pub fn is_valid(self) -> bool {
        self.0 < CELLS
    }

    /// The spaces one step away.
    pub fn neighbours(self) -> impl Iterator<Item = Cell> {
        let (r, c) = (i16::from(self.row()), i16::from(self.col()));
        // An odd column sits half a space lower: its side neighbours are on
        // its row and the next one; an even column's, on the previous one.
        let side = if c % 2 == 1 { [0, 1] } else { [-1, 0] };
        let steps = [
            (-1, 0),
            (1, 0),
            (side[0], -1),
            (side[1], -1),
            (side[0], 1),
            (side[1], 1),
        ];
        steps.into_iter().filter_map(move |(dr, dc)| {
            let (nr, nc) = (r + dr, c + dc);
            let row = u8::try_from(nr).ok()?;
            let col = u8::try_from(nc).ok()?;
            Cell::new(row, col)
        })
    }
}

/// A set of spaces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mask(pub u128);

const FULL: u128 = (1u128 << CELLS) - 1;

impl Mask {
    pub fn empty() -> Mask {
        Mask(0)
    }

    pub fn full() -> Mask {
        Mask(FULL)
    }

    pub fn contains(self, c: Cell) -> bool {
        c.is_valid() && self.0 & (1u128 << c.0) != 0
    }

    pub fn insert(&mut self, c: Cell) {
        if c.is_valid() {
            self.0 |= 1u128 << c.0;
        }
    }

    pub fn len(self) -> u32 {
        self.0.count_ones()
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The spaces, in increasing order.
    pub fn iter(self) -> impl Iterator<Item = Cell> {
        (0..CELLS).map(Cell).filter(move |&c| self.contains(c))
    }

    /// The only space, if there is exactly one.
    pub fn single(self) -> Option<Cell> {
        (self.len() == 1).then(|| Cell(self.0.trailing_zeros() as u8))
    }
}

impl BitAnd for Mask {
    type Output = Mask;
    fn bitand(self, other: Mask) -> Mask {
        Mask(self.0 & other.0)
    }
}

impl BitOr for Mask {
    type Output = Mask;
    fn bitor(self, other: Mask) -> Mask {
        Mask(self.0 | other.0)
    }
}

impl Not for Mask {
    type Output = Mask;
    fn not(self) -> Mask {
        Mask(!self.0 & FULL)
    }
}

impl FromIterator<Cell> for Mask {
    fn from_iter<I: IntoIterator<Item = Cell>>(cells: I) -> Mask {
        let mut m = Mask::empty();
        for c in cells {
            m.insert(c);
        }
        m
    }
}

fn distances() -> &'static [[u8; CELLS as usize]; CELLS as usize] {
    static TABLE: OnceLock<Box<[[u8; CELLS as usize]; CELLS as usize]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = Box::new([[u8::MAX; CELLS as usize]; CELLS as usize]);
        for from in 0..CELLS {
            let row = &mut table[usize::from(from)];
            row[usize::from(from)] = 0;
            let mut queue = std::collections::VecDeque::from([Cell(from)]);
            while let Some(c) = queue.pop_front() {
                let d = row[c.index()];
                for n in c.neighbours() {
                    if row[n.index()] == u8::MAX {
                        row[n.index()] = d + 1;
                        queue.push_back(n);
                    }
                }
            }
        }
        table
    })
}

/// Steps from `a` to `b` on the map.
pub fn distance(a: Cell, b: Cell) -> u8 {
    distances()[a.index()][b.index()]
}

/// [R p. 8] The spaces at most `n` steps from `c`, `c` included.
pub fn within(c: Cell, n: u8) -> Mask {
    (0..CELLS)
        .map(Cell)
        .filter(|&o| distance(c, o) <= n)
        .collect()
}

/// [J] `MAP`, `BEAR`, `COUGAR` (sections 1 to 6), checked space by space
/// against [B] `gameboard.py`: terrain rows (W lake, M mountain, F forest,
/// S swamp, D desert) and territories (B bear, C cougar).
const SECTIONS: [([&str; 3], [&str; 3]); 6] = [
    (
        ["WWWWFF", "SSWDFF", "SSDDDF"],
        ["......", "......", "...BBB"],
    ),
    (
        ["SFFFFF", "SSFDDD", "SMMMMD"],
        ["CCC...", "......", "......"],
    ),
    (
        ["SSFFFW", "SSFMWW", "MMMMWW"],
        ["......", "CC....", "C....."],
    ),
    (
        ["DDMMMM", "DDMWWW", "DDDFFF"],
        ["......", ".....C", ".....C"],
    ),
    (
        ["SSSMMM", "SDDWMM", "DDWWWW"],
        ["......", ".....B", "....BB"],
    ),
    (
        ["DDSSSF", "MMSSFF", "MWWWWF"],
        ["B.....", "B.....", "......"],
    ),
];

fn section_square(section: usize, row: usize, col: usize) -> (Terrain, Option<Animal>) {
    let (terrains, animals) = SECTIONS[section];
    let terrain = match terrains[row].as_bytes()[col] {
        b'W' => Terrain::Lake,
        b'M' => Terrain::Mountain,
        b'F' => Terrain::Forest,
        b'S' => Terrain::Swamp,
        _ => Terrain::Desert,
    };
    let animal = match animals[row].as_bytes()[col] {
        b'B' => Some(Animal::Bear),
        b'C' => Some(Animal::Cougar),
        _ => None,
    };
    (terrain, animal)
}

/// Which section lies at each position (reading order: top band left,
/// top band right, …), and whether it is turned upside down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// Section index (0 = section 1) at each position.
    pub order: [u8; 6],
    pub flipped: [bool; 6],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Structure {
    pub kind: Kind,
    pub color: Color,
    pub cell: Cell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Square {
    pub terrain: Terrain,
    pub animal: Option<Animal>,
    pub structure: Option<(Kind, Color)>,
}

/// The assembled map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub squares: Vec<Square>,
}

impl Board {
    pub fn new(layout: &Layout, structures: &[Structure]) -> Board {
        let mut squares = Vec::with_capacity(usize::from(CELLS));
        for c in (0..CELLS).map(Cell) {
            let (row, col) = (usize::from(c.row()), usize::from(c.col()));
            let position = (row / 3) * 2 + col / 6;
            let (mut r, mut k) = (row % 3, col % 6);
            // [B] flip_piece: turned upside down, by half a turn.
            if layout.flipped[position] {
                (r, k) = (2 - r, 5 - k);
            }
            let (terrain, animal) = section_square(usize::from(layout.order[position]), r, k);
            let structure = structures
                .iter()
                .find(|s| s.cell == c)
                .map(|s| (s.kind, s.color));
            squares.push(Square {
                terrain,
                animal,
                structure,
            });
        }
        Board { squares }
    }

    pub fn square(&self, c: Cell) -> &Square {
        &self.squares[c.index()]
    }
}

/// [J] `generateRandomMap`: the sections in a random order, each turned
/// or not.
pub fn random_layout<R: Rng + ?Sized>(rng: &mut R) -> Layout {
    let mut order = [0, 1, 2, 3, 4, 5];
    for i in (1..order.len()).rev() {
        let j = rng.gen_range(0..=i as u32) as usize;
        order.swap(i, j);
    }
    let flipped = std::array::from_fn(|_| rng.gen_bool(0.5));
    Layout { order, flipped }
}
