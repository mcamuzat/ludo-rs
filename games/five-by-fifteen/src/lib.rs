//! 5×15 (Friedemann Friese, 2F-Spiele, 2020), a solo game. Rules and their
//! sources: RULES.md.

mod heuristics;
pub mod testing;
mod view;

pub use view::{Locomotive, View};

use std::collections::BTreeSet;

use ludo_core::{
    Determinize, Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub struct FiveByFifteen;

pub const ROWS: usize = 5;
/// Columns 1 to 15 of the rule; the zeroth column (the "1"s) is `anchors`.
pub const COLS: usize = 15;

/// The five colours, in the order of their names in the interface.
pub const COLORS: [Color; 5] = [
    Color::Emerald,
    Color::Crimson,
    Color::Azure,
    Color::Amber,
    Color::Violet,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
    Emerald,
    Crimson,
    Azure,
    Amber,
    Violet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tile {
    pub color: Color,
    pub number: u8,
}

/// [R] "Adjusting the difficulty": the shortest run action b may move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    #[default]
    Short,
    Medium,
    Long,
}

impl Route {
    pub fn min_run(self) -> usize {
        match self {
            Route::Short => 3,
            Route::Medium => 4,
            Route::Long => 5,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub route: Route,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pos {
    pub row: u8,
    pub col: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// [R] Action a: the tile at `from` goes into the gap `to`.
    Place { from: Pos, to: Pos },
    /// [R] Action b: the run ending with the 15 at `to - 1` moves one
    /// column right, into the gap `to`.
    Shift { row: u8, to: u8 },
    /// [U] Ends the game, lost.
    GiveUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Playing,
    Won,
    /// [R] No action left.
    Stuck,
    GaveUp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    /// The colour of each row's "1".
    pub anchors: [Color; ROWS],
    pub cells: [[Option<Tile>; COLS]; ROWS],
    pub status: Status,
    pub moves: u32,
    /// [U] The position after the last move was seen before (a loop).
    pub repeated: bool,
    /// Hashes of every position reached.
    pub seen: BTreeSet<u64>,
}

impl State {
    fn at(&self, row: usize, col: usize) -> Option<Tile> {
        self.cells[row][col]
    }

    /// The tile left of a cell; left of the first column, the row's "1".
    fn left_of(&self, row: usize, col: usize) -> Option<Tile> {
        if col == 0 {
            Some(Tile {
                color: self.anchors[row],
                number: 1,
            })
        } else {
            self.at(row, col - 1)
        }
    }

    fn right_of(&self, row: usize, col: usize) -> Option<Tile> {
        (col + 1 < COLS).then(|| self.at(row, col + 1)).flatten()
    }

    /// [R] a) Same colour as a neighbour, 1 higher than the left one or 1
    /// lower than the right one.
    fn fits(&self, tile: Tile, row: usize, col: usize) -> bool {
        let after = self
            .left_of(row, col)
            .is_some_and(|l| l.color == tile.color && l.number + 1 == tile.number);
        let before = self
            .right_of(row, col)
            .is_some_and(|r| r.color == tile.color && r.number == tile.number + 1);
        after || before
    }

    /// How many tiles of a row are in place from the left (0 to 14).
    pub fn sorted(&self, row: usize) -> usize {
        (0..COLS - 1)
            .take_while(|&c| {
                self.at(row, c)
                    == Some(Tile {
                        color: self.anchors[row],
                        number: c as u8 + 2,
                    })
            })
            .count()
    }

    /// The length of the run of consecutive tiles of one colour ending at
    /// `(row, col)` (0 if the cell is empty).
    fn run_ending_at(&self, row: usize, col: usize) -> usize {
        let Some(last) = self.at(row, col) else {
            return 0;
        };
        let mut len = 1;
        while len <= col {
            match self.at(row, col - len) {
                Some(t)
                    if t.color == last.color
                        && usize::from(t.number) + len == usize::from(last.number) =>
                {
                    len += 1
                }
                _ => break,
            }
        }
        len
    }

    /// [R] b) The run the 15 left of `to` ends, if action b may move it.
    fn shift_run(&self, row: usize, to: usize) -> Option<usize> {
        if to == 0 || to >= COLS || self.at(row, to).is_some() {
            return None;
        }
        // [R] A sorted row is gapless: its last column is no gap.
        if self.sorted(row) == COLS - 1 {
            return None;
        }
        if self.at(row, to - 1)?.number != 15 {
            return None;
        }
        let len = self.run_ending_at(row, to - 1);
        (len >= self.config.route.min_run()).then_some(len)
    }

    fn gaps(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..ROWS).flat_map(move |r| {
            (0..COLS)
                .filter(move |&c| self.at(r, c).is_none())
                .map(move |c| (r, c))
        })
    }

    /// Actions a and b, in a fixed order.
    fn moves(&self) -> Vec<Action> {
        let mut out = Vec::new();
        for (r, c) in self.gaps() {
            for fr in 0..ROWS {
                for fc in 0..COLS {
                    if let Some(tile) = self.at(fr, fc) {
                        if self.fits(tile, r, c) {
                            out.push(Action::Place {
                                from: pos(fr, fc),
                                to: pos(r, c),
                            });
                        }
                    }
                }
            }
        }
        for (r, c) in self.gaps() {
            if self.shift_run(r, c).is_some() {
                out.push(Action::Shift {
                    row: r as u8,
                    to: c as u8,
                });
            }
        }
        out
    }

    fn won(&self) -> bool {
        (0..ROWS).all(|r| self.sorted(r) == COLS - 1)
    }

    /// FNV-1a over the cells, written out so that it is the same in wasm32
    /// and natively.
    fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for cell in self.cells.iter().flatten() {
            let byte = cell.map_or(0, |t| t.color as u8 * 16 + t.number);
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    /// Ends the game when it is won or stuck.
    fn settle(&mut self) {
        if self.won() {
            self.status = Status::Won;
        } else if self.moves().is_empty() {
            self.status = Status::Stuck;
        }
    }

    /// A position just reached: remembered, flagged if seen before.
    fn reached(&mut self) {
        self.repeated = !self.seen.insert(self.hash());
        self.settle();
    }

    fn fresh(config: Config, anchors: [Color; ROWS], cells: [[Option<Tile>; COLS]; ROWS]) -> State {
        let mut s = State {
            config,
            anchors,
            cells,
            status: Status::Playing,
            moves: 0,
            repeated: false,
            seen: BTreeSet::new(),
        };
        s.reached();
        s
    }
}

fn pos(row: usize, col: usize) -> Pos {
    Pos {
        row: row as u8,
        col: col as u8,
    }
}

fn cell(p: Pos) -> Option<(usize, usize)> {
    let (r, c) = (usize::from(p.row), usize::from(p.col));
    (r < ROWS && c < COLS).then_some((r, c))
}

fn shuffle<T>(items: &mut [T], rng: &mut ChaCha8Rng) {
    for i in (1..items.len()).rev() {
        let j = rng.gen_range(0..=i as u32) as usize;
        items.swap(i, j);
    }
}

impl Game for FiveByFifteen {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "five-by-fifteen",
            name: "5×15",
            min_players: 1,
            max_players: 1,
            hidden_info: false,
            clock: false,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "route",
            kind: OptionKind::Choice {
                values: &["Short", "Medium", "Long"],
                default: "Short",
            },
        }]
    }

    /// [R] "Game preparations".
    fn new(config: &Config, _players: u8, seed: u64) -> State {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut tiles: Vec<Tile> = COLORS
            .into_iter()
            .flat_map(|color| (1..=15).map(move |number| Tile { color, number }))
            .collect();
        shuffle(&mut tiles, &mut rng);
        let mut cells = [[None; COLS]; ROWS];
        let mut ones = Vec::new();
        for (i, tile) in tiles.into_iter().enumerate() {
            if tile.number == 1 {
                ones.push(tile.color);
            } else {
                cells[i / COLS][i % COLS] = Some(tile);
            }
        }
        shuffle(&mut ones, &mut rng);
        let anchors = [ones[0], ones[1], ones[2], ones[3], ones[4]];
        State::fresh(config.clone(), anchors, cells)
    }

    fn pending(s: &State) -> Pending {
        if s.status == Status::Playing {
            Pending::one(Seat(0))
        } else {
            Pending::none()
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat != Seat(0) || s.status != Status::Playing {
            return Vec::new();
        }
        let mut out = s.moves();
        out.push(Action::GiveUp);
        out
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if seat != Seat(0) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if s.status != Status::Playing {
            return Err(Illegal::new("La partie est finie."));
        }
        match *action {
            Action::GiveUp => {
                s.status = Status::GaveUp;
                return Ok(());
            }
            Action::Place { from, to } => {
                let ((fr, fc), (tr, tc)) = cell(from)
                    .zip(cell(to))
                    .ok_or_else(|| Illegal::new("Case hors du plateau."))?;
                let tile = s
                    .at(fr, fc)
                    .ok_or_else(|| Illegal::new("Il n'y a pas de wagon sur cette case."))?;
                if s.at(tr, tc).is_some() {
                    return Err(Illegal::new("Ce n'est pas un trou."));
                }
                if !s.fits(tile, tr, tc) {
                    return Err(Illegal::new(
                        "Ce wagon doit suivre son prédécesseur ou précéder son successeur, de la même couleur.",
                    ));
                }
                s.cells[fr][fc] = None;
                s.cells[tr][tc] = Some(tile);
            }
            Action::Shift { row, to } => {
                let (r, c) = (usize::from(row), usize::from(to));
                if r >= ROWS {
                    return Err(Illegal::new("Voie hors du plateau."));
                }
                let len = s.shift_run(r, c).ok_or_else(|| {
                    Illegal::new("Seul un train assez long, mené par le 15, avance dans le trou qui le suit.")
                })?;
                for k in (c - len..c).rev() {
                    s.cells[r][k + 1] = s.cells[r][k];
                }
                s.cells[r][c - len] = None;
            }
        }
        s.moves += 1;
        s.reached();
        Ok(())
    }

    fn view(s: &State, _viewer: Viewer) -> View {
        View::of(s)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        (s.status != Status::Playing).then(|| Outcome::Ranked(vec![0]))
    }
}

impl Determinize for FiveByFifteen {
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, _rng: &mut R) -> State {
        s.clone()
    }
}
