//! Clues [R p. 9–10] and the puzzle generator ([J] `generateClues`).

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::map::{
    random_layout, within, Animal, Board, Cell, Color, Kind, Layout, Mask, Structure, Terrain,
    CELLS,
};

/// [R p. 5] Normal: positive clues, three structure colours. Advanced:
/// negative clues too, and the black structures.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Normal,
    Advanced,
}

/// [R p. 9–10] What a clue is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Clue {
    /// On one of two terrains.
    Terrains(Terrain, Terrain),
    /// On, or 1 space from, a terrain.
    NearTerrain(Terrain),
    /// On, or 1 space from, either animal's territory.
    NearAnimals,
    /// On, or up to 2 spaces from, a kind of structure.
    NearKind(Kind),
    /// On, or up to 2 spaces from, one animal's territory.
    NearAnimal(Animal),
    /// On, or up to 3 spaces from, a structure colour.
    NearColor(Color),
}

impl Clue {
    /// [R p. 8] The distance, the element's own space included.
    pub fn distance(self) -> u8 {
        match self {
            Clue::Terrains(..) => 0,
            Clue::NearTerrain(_) | Clue::NearAnimals => 1,
            Clue::NearKind(_) | Clue::NearAnimal(_) => 2,
            Clue::NearColor(_) => 3,
        }
    }

    /// The spaces the clue's element is on.
    fn sources(self, board: &Board) -> Mask {
        (0..CELLS)
            .map(Cell)
            .filter(|&c| {
                let sq = board.square(c);
                match self {
                    Clue::Terrains(a, b) => sq.terrain == a || sq.terrain == b,
                    Clue::NearTerrain(t) => sq.terrain == t,
                    Clue::NearAnimals => sq.animal.is_some(),
                    Clue::NearAnimal(a) => sq.animal == Some(a),
                    Clue::NearKind(k) => sq.structure.is_some_and(|(kind, _)| kind == k),
                    Clue::NearColor(col) => sq.structure.is_some_and(|(_, color)| color == col),
                }
            })
            .collect()
    }
}

/// A player's clue: [R p. 5] in advanced mode it may be negative ("the
/// habitat is not …").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hint {
    pub clue: Clue,
    pub negated: bool,
}

impl Hint {
    /// The spaces the habitat may be on, according to this hint.
    pub fn cells(&self, board: &Board) -> Mask {
        let sources = self.clue.sources(board);
        let n = self.clue.distance();
        let near = if n == 0 {
            sources
        } else {
            sources.iter().fold(Mask::empty(), |m, c| m | within(c, n))
        };
        if self.negated {
            !near
        } else {
            near
        }
    }
}

/// The clues of a mode, in the order of [J] `CLUE_DATA`.
pub fn hints(mode: Mode) -> Vec<Hint> {
    let mut clues = Vec::new();
    let t = Terrain::ALL;
    for i in 0..t.len() {
        for j in i + 1..t.len() {
            clues.push(Clue::Terrains(t[i], t[j]));
        }
    }
    for terrain in [
        Terrain::Forest,
        Terrain::Desert,
        Terrain::Swamp,
        Terrain::Mountain,
        Terrain::Lake,
    ] {
        clues.push(Clue::NearTerrain(terrain));
    }
    clues.push(Clue::NearAnimals);
    clues.extend(Kind::ALL.map(Clue::NearKind));
    clues.extend(Animal::ALL.map(Clue::NearAnimal));
    let colors: &[Color] = match mode {
        Mode::Normal => &[Color::White, Color::Blue, Color::Green],
        Mode::Advanced => &[Color::White, Color::Blue, Color::Green, Color::Black],
    };
    clues.extend(colors.iter().map(|&c| Clue::NearColor(c)));
    let positive = clues.iter().map(|&clue| Hint {
        clue,
        negated: false,
    });
    match mode {
        Mode::Normal => positive.collect(),
        Mode::Advanced => positive
            .clone()
            .chain(clues.iter().map(|&clue| Hint {
                clue,
                negated: true,
            }))
            .collect(),
    }
}

/// The hints of a mode with their spaces on one map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HintTable {
    pub hints: Vec<Hint>,
    pub masks: Vec<Mask>,
}

impl HintTable {
    pub fn new(board: &Board, mode: Mode) -> HintTable {
        let hints = hints(mode);
        let masks = hints.iter().map(|h| h.cells(board)).collect();
        HintTable { hints, masks }
    }

    pub fn index_of(&self, hint: &Hint) -> Option<usize> {
        self.hints.iter().position(|h| h == hint)
    }
}

/// [J] At most two clues "on one of two terrains" (negative ones too).
pub const MAX_TERRAIN_CLUES: usize = 2;

fn is_terrains(table: &HintTable, i: usize) -> bool {
    matches!(table.hints[i].clue, Clue::Terrains(..))
}

/// The habitat that `chosen` (indices in `table`) names, when they form a
/// puzzle: all distinct [R p. 8], exactly one space in common, each of them
/// needed (without it the others name more than one space, [J]
/// `checkSolution`), at most two "two terrains" clues [J].
pub fn habitat(table: &HintTable, chosen: &[usize]) -> Option<Cell> {
    if chosen.iter().any(|&i| i >= table.hints.len()) {
        return None;
    }
    for (k, i) in chosen.iter().enumerate() {
        if chosen[..k].contains(i) {
            return None;
        }
    }
    if chosen.iter().filter(|&&i| is_terrains(table, i)).count() > MAX_TERRAIN_CLUES {
        return None;
    }
    let home = chosen
        .iter()
        .fold(Mask::full(), |m, &i| m & table.masks[i])
        .single()?;
    let needed = (0..chosen.len()).all(|skip| {
        let others = chosen
            .iter()
            .enumerate()
            .filter(|&(k, _)| k != skip)
            .fold(Mask::full(), |m, (_, &i)| m & table.masks[i]);
        others.len() > 1
    });
    needed.then_some(home)
}

/// A game's map and clues: one hint per seat, in seat order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Puzzle {
    pub layout: Layout,
    pub structures: Vec<Structure>,
    pub hints: Vec<Hint>,
    pub habitat: Cell,
}

impl Puzzle {
    pub fn board(&self) -> Board {
        Board::new(&self.layout, &self.structures)
    }
}

fn draw(rng: &mut ChaCha8Rng, n: usize) -> usize {
    rng.gen_range(0..n as u32) as usize
}

/// [R p. 4–5] Every kind in each colour of the mode, on distinct spaces.
fn random_structures(mode: Mode, rng: &mut ChaCha8Rng) -> Vec<Structure> {
    let colors: &[Color] = match mode {
        Mode::Normal => &Color::ALL[..3],
        Mode::Advanced => &Color::ALL,
    };
    let mut taken = Mask::empty();
    let mut out = Vec::new();
    for &color in colors {
        for kind in Kind::ALL {
            let cell = loop {
                let c = Cell(rng.gen_range(0..u32::from(CELLS)) as u8);
                if !taken.contains(c) {
                    break c;
                }
            };
            taken.insert(cell);
            out.push(Structure { kind, color, cell });
        }
    }
    out
}

/// Attempts at clues on one map before drawing another ([J]: 50).
const TRIES: usize = 50;

/// [J] `generateClues`, its defects fixed (see RULES.md): random clues
/// first, each keeping some space; then the clue that leaves the fewest
/// spaces (more than one); then one that leaves exactly one.
fn try_clues(table: &HintTable, players: usize, rng: &mut ChaCha8Rng) -> Option<Vec<usize>> {
    let n = table.hints.len();
    let mut chosen: Vec<usize> = Vec::new();
    let mut left = Mask::full();
    let allowed = |chosen: &[usize], i: usize| {
        !chosen.contains(&i)
            && !(is_terrains(table, i)
                && chosen.iter().filter(|&&j| is_terrains(table, j)).count() >= MAX_TERRAIN_CLUES)
    };
    while chosen.len() < players - 2 {
        let options: Vec<usize> = (0..n)
            .filter(|&i| allowed(&chosen, i) && (left & table.masks[i]).len() > 1)
            .collect();
        if options.is_empty() {
            return None;
        }
        let i = options[draw(rng, options.len())];
        left = left & table.masks[i];
        chosen.push(i);
    }
    let fewest = (0..n)
        .filter(|&i| allowed(&chosen, i) && (left & table.masks[i]).len() > 1)
        .min_by_key(|&i| (left & table.masks[i]).len())?;
    let narrow: Vec<usize> = (0..n)
        .filter(|&i| allowed(&chosen, i) && (left & table.masks[i]).len() > 1)
        .filter(|&i| (left & table.masks[i]).len() == (left & table.masks[fewest]).len())
        .collect();
    let i = narrow[draw(rng, narrow.len())];
    left = left & table.masks[i];
    chosen.push(i);
    let last: Vec<usize> = (0..n)
        .filter(|&i| allowed(&chosen, i) && (left & table.masks[i]).len() == 1)
        .collect();
    if last.is_empty() {
        return None;
    }
    chosen.push(last[draw(rng, last.len())]);
    habitat(table, &chosen).is_some().then_some(chosen)
}

/// A random map and puzzle for `players` seats (3 to 5).
pub fn generate(mode: Mode, players: u8, seed: u64) -> Puzzle {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let players = usize::from(players.clamp(3, 5));
    loop {
        let layout = random_layout(&mut rng);
        let structures = random_structures(mode, &mut rng);
        let board = Board::new(&layout, &structures);
        let table = HintTable::new(&board, mode);
        for _ in 0..TRIES {
            if let Some(chosen) = try_clues(&table, players, &mut rng) {
                // The order of the clues says nothing: shuffle them among seats.
                let mut chosen = chosen;
                for i in (1..chosen.len()).rev() {
                    chosen.swap(i, draw(&mut rng, i + 1));
                }
                let habitat = habitat(&table, &chosen).expect("checked");
                return Puzzle {
                    layout,
                    structures,
                    hints: chosen.iter().map(|&i| table.hints[i]).collect(),
                    habitat,
                };
            }
        }
    }
}
