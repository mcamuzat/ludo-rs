//! Black Hole (Wal Joris): two players lay their tiles 1 to 10 on a
//! triangle of 21 cells; the one cell left empty is the black hole, and the
//! lower total of tiles around it wins. Rules and their source: RULES.md,
//! where [B] is the rulebook.

pub mod board;
mod heuristics;
mod view;

pub use board::{neighbours, pos, Pos, CELLS};
pub use view::View;

use ludo_core::{
    Determinize, Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer,
};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub struct BlackHole;

/// [B] Tiles 1 to 10.
pub const TILES: u8 = 10;

/// The order tiles are laid in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Order {
    /// [B] The rule: 1 first, then 2, and so on.
    #[default]
    Ascending,
    /// [B] Variant 1: any order.
    Free,
    /// [B] Variant 2: drawn at random from a face-down pool.
    Random,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub order: Order,
}

/// A laid tile. Seat 0 is red, seat 1 blue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub owner: u8,
    pub value: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Place { cell: u8, tile: u8 },
}

/// The end of the game: the empty cell and each player's total around it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finish {
    pub hole: u8,
    pub sums: [u32; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub cells: Vec<Option<Tile>>,
    /// Each seat's tiles still to lay. Ascending: sorted; Random: in the
    /// order they will be drawn (hidden but for the first of the seat to
    /// move); Free: sorted.
    pub hands: Vec<Vec<u8>>,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    pub last: Option<u8>,
    pub finish: Option<Finish>,
}

impl State {
    /// The tile `seat` must lay now, unless the order is free.
    pub fn next_tile(&self, seat: Seat) -> Option<u8> {
        match self.config.order {
            Order::Free => None,
            Order::Ascending | Order::Random => self.hands[seat.index()].first().copied(),
        }
    }

    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.turn != Some(seat) {
            return Vec::new();
        }
        let tiles: Vec<u8> = match self.next_tile(seat) {
            Some(t) => vec![t],
            None => self.hands[seat.index()].clone(),
        };
        let empty: Vec<u8> = (0..CELLS as u8)
            .filter(|&c| self.cells[usize::from(c)].is_none())
            .collect();
        tiles
            .iter()
            .flat_map(|&tile| empty.iter().map(move |&cell| Action::Place { cell, tile }))
            .collect()
    }

    /// Each seat's total of tiles around `hole`.
    pub fn sums_around(&self, hole: u8) -> [u32; 2] {
        let mut sums = [0; 2];
        for n in neighbours(hole) {
            if let Some(t) = self.cells[usize::from(n)] {
                sums[usize::from(t.owner)] += u32::from(t.value);
            }
        }
        sums
    }

    fn place(&mut self, seat: Seat, cell: u8, tile: u8) {
        let hand = &mut self.hands[seat.index()];
        let at = hand
            .iter()
            .position(|&t| t == tile)
            .expect("a tile in hand");
        hand.remove(at);
        self.cells[usize::from(cell)] = Some(Tile {
            owner: seat.0,
            value: tile,
        });
        self.last = Some(cell);
        // [B] Once all 20 tiles are laid, the empty cell is the hole.
        if self.hands.iter().all(Vec::is_empty) {
            let hole = self
                .cells
                .iter()
                .position(Option::is_none)
                .expect("one empty cell") as u8;
            self.finish = Some(Finish {
                hole,
                sums: self.sums_around(hole),
            });
            self.turn = None;
        } else {
            self.turn = Some(Seat(1 - seat.0));
        }
    }
}

impl Game for BlackHole {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "black-hole",
            name: "Black Hole",
            min_players: 2,
            max_players: 2,
            // The Random order hides the pools (from both players alike).
            hidden_info: true,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "order",
            kind: OptionKind::Choice {
                values: &["Ascending", "Free", "Random"],
                default: "Ascending",
            },
        }]
    }

    /// Hot seat: The Random order hides the pools from both players alike.
    fn shared_screen() -> bool {
        true
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if players != 2 {
            return Err(Illegal::new("Black Hole se joue à 2 joueurs."));
        }
        Ok(())
    }

    fn new(config: &Config, _players: u8, seed: u64) -> State {
        let mut hands: Vec<Vec<u8>> = vec![(1..=TILES).collect(); 2];
        if config.order == Order::Random {
            // ChaCha, not SmallRng: a browser log must replay natively.
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            hands.iter_mut().for_each(|h| h.shuffle(&mut rng));
        }
        State {
            config: config.clone(),
            cells: vec![None; CELLS],
            hands,
            // [B] Red starts.
            turn: Some(Seat(0)),
            last: None,
            finish: None,
        }
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        let Action::Place { cell, tile } = *action;
        if s.turn.is_none() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if usize::from(cell) >= CELLS || s.cells[usize::from(cell)].is_some() {
            return Err(Illegal::new("Cette case n'est pas libre."));
        }
        if !s.hands[seat.index()].contains(&tile) {
            return Err(Illegal::new("Tu n'as plus cette tuile."));
        }
        if s.next_tile(seat).is_some_and(|t| t != tile) {
            return Err(Illegal::new("Ce n'est pas la tuile à poser."));
        }
        s.place(seat, cell, tile);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// [B] The lower total wins. ❓ Equal totals: a draw.
    fn outcome(s: &State) -> Option<Outcome> {
        let f = s.finish.as_ref()?;
        let ranks = match f.sums[0].cmp(&f.sums[1]) {
            std::cmp::Ordering::Less => vec![0, 1],
            std::cmp::Ordering::Greater => vec![1, 0],
            std::cmp::Ordering::Equal => vec![0, 0],
        };
        Some(Outcome::Ranked(ranks))
    }

    /// The first free cell, with the smallest tile allowed.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        s.moves(seat)
            .into_iter()
            .min_by_key(|&Action::Place { cell, tile }| (tile, cell))
    }
}

impl Determinize for BlackHole {
    /// Random order: the pools are reshuffled, but for the tile the seat to
    /// move has already drawn. Identity otherwise.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        if d.config.order == Order::Random {
            for (i, hand) in d.hands.iter_mut().enumerate() {
                let drawn = usize::from(d.turn.is_some_and(|t| t.index() == i)).min(hand.len());
                hand[drawn..].shuffle(rng);
            }
        }
        d
    }
}
