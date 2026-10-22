//! Das glorreiche Manöver (Roland Siegers, Schmidt 1992): a race of 84
//! squares where a soldier goes the farther the further back it is and the
//! higher it stands. Rules and their sources: RULES.md.

pub mod board;
pub mod testing;
mod view;

pub use view::View;

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use board::{Square, LAST};

pub struct Manoever;

/// No options yet; the variants of the rules would go here.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {}

/// A soldier: its owner and the number it bears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Soldier {
    pub seat: Seat,
    pub number: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dest {
    Square(Square),
    /// Past square 84, into the barracks.
    Barracks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// [R] Setup: throw the die and bring a soldier out on that square.
    Roll,
    /// Soldier `soldier` (its number) of the player to move goes to `to`.
    Move { soldier: u8, to: Dest },
}

/// One move of a turn, for the site to show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRecord {
    pub number: u8,
    pub from: Square,
    pub to: Dest,
    /// Value of the gray piece won on landing.
    pub gray: Option<u8>,
    /// Barracks place taken (1 to 12).
    pub place: Option<u8>,
}

/// [R] Soldiers per player: 12 in all.
pub fn soldiers_per_seat(players: u8) -> u8 {
    match players {
        2 => 6,
        3 => 4,
        _ => 3,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    /// Draws the setup rolls; never shown.
    pub seed: u64,
    /// Dice thrown so far, rerolls included.
    pub rolls: u32,
    /// Soldiers of each seat still waiting to come out at the setup.
    pub waiting: Vec<u8>,
    /// `stacks[square]`, bottom to top; index 0 unused.
    pub stacks: Vec<Vec<Soldier>>,
    /// `barracks[place - 1]`.
    pub barracks: [Option<Soldier>; 12],
    /// Who won each gray piece of `board::GRAYS`.
    pub grays: [Option<Seat>; 3],
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// Numbers of the soldiers already moved this turn.
    pub moved: Vec<u8>,
    pub last_roll: Option<(Seat, u8)>,
    /// Moves of the turn being played.
    pub turn_log: Vec<MoveRecord>,
    /// The last complete turn of each seat.
    pub last_turns: Vec<Vec<MoveRecord>>,
    /// Actions played since the start.
    pub moves: u32,
    pub over: bool,
}

/// Die number `k` of a game seeded with `seed`.
fn die(seed: u64, k: u32) -> u8 {
    let mut rng =
        ChaCha8Rng::seed_from_u64(seed ^ u64::from(k).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    (rng.gen::<u32>() % 6 + 1) as u8
}

impl State {
    fn in_setup(&self) -> bool {
        self.waiting.iter().any(|&w| w > 0)
    }

    /// [R] Setup: roll, reroll onto a full square, and bring out the
    /// lowest-numbered waiting soldier (❓ its number does not matter).
    fn roll(&mut self, seat: Seat) {
        let square = loop {
            let d = die(self.seed, self.rolls);
            self.rolls += 1;
            if self.stacks[usize::from(d)].len() < 3 {
                break d;
            }
        };
        let total = soldiers_per_seat(self.players);
        let waiting = &mut self.waiting[seat.index()];
        let number = total - *waiting + 1;
        *waiting -= 1;
        self.stacks[usize::from(square)].push(Soldier { seat, number });
        self.last_roll = Some((seat, square));
        // The next seat with a soldier waiting; seat 0 opens the march.
        self.turn = Some(
            (1..=self.players)
                .map(|k| Seat((seat.0 + k) % self.players))
                .find(|s| self.waiting[s.index()] > 0)
                .unwrap_or(Seat(0)),
        );
    }
}

impl Game for Manoever {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "manoever",
            name: "Das glorreiche Manöver",
            min_players: 2,
            max_players: 4,
            hidden_info: true,
            clock: true,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        State {
            players,
            seed,
            rolls: 0,
            waiting: vec![soldiers_per_seat(players); usize::from(players)],
            stacks: vec![Vec::new(); usize::from(LAST) + 1],
            barracks: [None; 12],
            grays: [None; 3],
            turn: Some(Seat(0)),
            moved: Vec::new(),
            last_roll: None,
            turn_log: Vec::new(),
            last_turns: vec![Vec::new(); usize::from(players)],
            moves: 0,
            over: false,
        }
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != Some(seat) {
            return Vec::new();
        }
        if s.in_setup() {
            return vec![Action::Roll];
        }
        Vec::new()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.turn.is_none() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match *action {
            Action::Roll if s.in_setup() => s.roll(seat),
            Action::Roll => return Err(Illegal::new("Le dé ne sert plus.")),
            Action::Move { .. } => return Err(Illegal::new("Ce soldat ne peut pas aller là.")),
        }
        s.moves += 1;
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(_s: &State) -> Option<Outcome> {
        None
    }
}

impl Determinize for Manoever {
    /// Only the setup rolls to come are hidden: their seed is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut s = s.clone();
        s.seed = rng.gen();
        s
    }
}
