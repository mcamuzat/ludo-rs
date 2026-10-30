//! Siberian Dice (Borovkov, Panferov): roll a die, place it, make dice perfect.
//! Rules and their sources: RULES.md.

pub mod board;
mod heuristics;
pub mod testing;
mod view;

pub use heuristics::gain;
pub use view::{CellView, DieView, View};

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use board::{cells, neighbors, CellId};

pub struct SiberianDice;

/// [R] Dice of each player.
pub const DICE: u8 = 12;
/// [R] Perfect dice needed to win.
pub const TARGET: u8 = 6;
/// [U] ludo-rs addition, not in [R]: a draw after this many rounds of the
/// table without a new record of perfect dice.
pub const QUIET_ROUNDS: u32 = 30;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Die {
    /// The seat owning it: 0 blue, 1 red.
    pub owner: u8,
    pub value: u8,
}

/// Where the die to roll comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    Reserve,
    Cell(CellId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// [R] Choose one of one's dice: the engine rolls it.
    Pick(Source),
    /// [R] Put the rolled die on a vacant cell.
    Place(CellId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Choose,
    /// The die from `from` rolled `value` and waits to be placed.
    Place {
        from: Source,
        value: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastMove {
    pub seat: u8,
    pub from: Source,
    pub to: CellId,
    pub value: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    /// [R] Only this seat has six perfect dice or more.
    Won { seat: u8 },
    /// [R] Both reached six with the same move: a draw.
    BothSix,
    /// [U] `QUIET_ROUNDS` rounds without a new record: a draw.
    QuietLimit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub board: Vec<Option<Die>>,
    pub reserve: [u8; 2],
    pub turn: Seat,
    pub step: Step,
    /// Most perfect dice each colour had at once since the start.
    pub records: [u8; 2],
    /// Moves in a row without a new record.
    pub quiet: u32,
    pub last: Option<LastMove>,
    /// Seed of the rolls: never in a view.
    pub seed: u64,
    /// Rolls made so far.
    pub rolls: u32,
    pub end: Option<End>,
}

/// Roll number `k` of a game seeded with `seed`.
fn roll(seed: u64, k: u32) -> u8 {
    let mut rng =
        ChaCha8Rng::seed_from_u64(seed ^ u64::from(k).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    (rng.gen::<u32>() % 6 + 1) as u8
}

/// [R] A die is perfect when its pips equal the number of dice on the
/// cells around it, whoever owns them.
pub fn is_perfect(board: &[Option<Die>], id: CellId) -> bool {
    board[usize::from(id)].is_some_and(|die| {
        let around = neighbors(id)
            .iter()
            .filter(|&&n| board[usize::from(n)].is_some())
            .count();
        usize::from(die.value) == around
    })
}

pub fn perfect_count(board: &[Option<Die>], seat: u8) -> u8 {
    (0..board.len() as CellId)
        .filter(|&id| board[usize::from(id)].is_some_and(|d| d.owner == seat))
        .filter(|&id| is_perfect(board, id))
        .count() as u8
}

impl State {
    fn other(&self) -> Seat {
        Seat(1 - self.turn.0)
    }

    /// [R] The move ends with the die placed: count, then pass the turn.
    fn settle(&mut self) {
        let p = [perfect_count(&self.board, 0), perfect_count(&self.board, 1)];
        self.end = match (p[0] >= TARGET, p[1] >= TARGET) {
            (true, true) => Some(End::BothSix),
            (true, false) => Some(End::Won { seat: 0 }),
            (false, true) => Some(End::Won { seat: 1 }),
            (false, false) => None,
        };
        if p[0] > self.records[0] || p[1] > self.records[1] {
            self.records = [p[0].max(self.records[0]), p[1].max(self.records[1])];
            self.quiet = 0;
        } else {
            self.quiet += 1;
            if self.end.is_none() && self.quiet >= 2 * QUIET_ROUNDS {
                self.end = Some(End::QuietLimit);
            }
        }
        self.turn = self.other();
        self.step = Step::Choose;
    }

    /// Takes the die from `from` and rolls `value` for it.
    pub(crate) fn take(&mut self, from: Source, value: u8) {
        match from {
            Source::Reserve => self.reserve[self.turn.index()] -= 1,
            Source::Cell(c) => self.board[usize::from(c)] = None,
        }
        self.step = Step::Place { from, value };
    }

    fn can_pick(&self, seat: Seat, from: Source) -> bool {
        match from {
            Source::Reserve => self.reserve[seat.index()] > 0,
            Source::Cell(c) => self
                .board
                .get(usize::from(c))
                .copied()
                .flatten()
                .is_some_and(|d| d.owner == seat.0),
        }
    }
}

impl Game for SiberianDice {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "siberian-dice",
            name: "Siberian Dice",
            min_players: 2,
            max_players: 2,
            hidden_info: true,
            clock: true,
        }
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if players != 2 {
            return Err(Illegal::new("Siberian Dice se joue à 2."));
        }
        Ok(())
    }

    fn new(_config: &Config, _players: u8, seed: u64) -> State {
        State {
            board: vec![None; cells().len()],
            reserve: [DICE; 2],
            turn: Seat(0),
            step: Step::Choose,
            records: [0; 2],
            quiet: 0,
            last: None,
            seed,
            rolls: 0,
            end: None,
        }
    }

    fn pending(s: &State) -> Pending {
        if s.end.is_some() {
            Pending::none()
        } else {
            Pending::one(s.turn)
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.end.is_some() || seat != s.turn {
            return Vec::new();
        }
        match s.step {
            Step::Choose => std::iter::once(Source::Reserve)
                .chain((0..cells().len() as CellId).map(Source::Cell))
                .filter(|&from| s.can_pick(seat, from))
                .map(Action::Pick)
                .collect(),
            Step::Place { .. } => (0..cells().len() as CellId)
                .filter(|&c| s.board[usize::from(c)].is_none())
                .map(Action::Place)
                .collect(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.end.is_some() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if seat != s.turn {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match (*action, s.step) {
            (Action::Pick(from), Step::Choose) => {
                if !s.can_pick(seat, from) {
                    return Err(Illegal::new(match from {
                        Source::Reserve => "Ta réserve est vide.",
                        Source::Cell(_) => "Choisis un de tes dés.",
                    }));
                }
                let value = roll(s.seed, s.rolls);
                s.rolls += 1;
                s.take(from, value);
            }
            (Action::Place(to), Step::Place { from, value }) => {
                if s.board.get(usize::from(to)) != Some(&None) {
                    return Err(Illegal::new("Pose ton dé sur une case vide."));
                }
                s.board[usize::from(to)] = Some(Die {
                    owner: seat.0,
                    value,
                });
                s.last = Some(LastMove {
                    seat: seat.0,
                    from,
                    to,
                    value,
                });
                s.settle();
            }
            (Action::Pick(_), Step::Place { .. }) => {
                return Err(Illegal::new("Pose d'abord le dé lancé."));
            }
            (Action::Place(_), Step::Choose) => {
                return Err(Illegal::new("Choisis d'abord le dé à lancer."));
            }
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.end.map(|end| {
            Outcome::Ranked(match end {
                End::Won { seat } => (0..2).map(|i| u8::from(i != seat)).collect(),
                End::BothSix | End::QuietLimit => vec![0, 0],
            })
        })
    }

    /// The reserve first, else the first die on the board; then the cell
    /// that best improves the lead in perfect dice (the first on a tie).
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        match s.step {
            Step::Choose => legal.into_iter().next(),
            Step::Place { .. } => legal.into_iter().rev().max_by_key(|a| match *a {
                Action::Place(to) => gain(s, seat, to),
                Action::Pick(_) => i32::MIN,
            }),
        }
    }
}

impl Determinize for SiberianDice {
    /// Only the rolls to come are hidden: their seed is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut s = s.clone();
        s.seed = rng.gen();
        s
    }
}
