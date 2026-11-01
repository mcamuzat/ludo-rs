//! Corporation (Eric Solomon): employees of four colours climb a building,
//! one office at a time, only when a pair of colours on the directors'
//! table promotes them. The first player whose three employees reach the
//! boardroom wins. Rules and their source: RULES.md, where [C] is the
//! rulebook.

mod heuristics;
mod view;

pub use view::{LastTurn, View};

use ludo_core::{
    Determinize, Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer,
};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub struct Corporation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    Red,
    Blue,
    Green,
    Yellow,
}

/// The order of the reception desks, of the offices' cycle, and of the
/// seats: seat 0 is red, 1 blue, 2 green, 3 yellow.
pub const COLORS: [Color; 4] = [Color::Red, Color::Blue, Color::Green, Color::Yellow];

impl Color {
    pub fn index(self) -> usize {
        self as usize
    }
}

/// [C] Three employees and three table counters of each colour.
pub const PER_COLOR: u8 = 3;
/// Positions 0..4 are the reception desks, 4..12 the offices from the
/// ground floor up, 12 the boardroom.
pub const DESKS: u8 = 4;
pub const BOARDROOM: u8 = 12;
pub const POSITIONS: usize = 13;
/// [C] Six promotions of two circles each: circle `2k` is the employee's
/// colour, `2k + 1` the colour beneath him. Promotion 1 (k = 0) is the
/// bottom one, as numbered in [C].
pub const CIRCLES: usize = 12;

/// ludo-rs addition, not in [C]: the game is drawn after this many rounds
/// of the table with no employee entered or promoted.
pub const QUIET_ROUNDS: u32 = 20;

/// The colour of a position: each desk its own, the offices cycling red,
/// blue, green, yellow from the ground floor (read off [C]'s figures); the
/// boardroom has none.
pub fn room_color(pos: u8) -> Option<Color> {
    match pos {
        0..DESKS => Some(COLORS[usize::from(pos)]),
        DESKS..BOARDROOM => Some(COLORS[usize::from(pos - DESKS) % 4]),
        _ => None,
    }
}

/// [C] Where a promotion leads: every desk to the ground floor, every
/// office to the one above, the top office to the boardroom.
pub fn above(pos: u8) -> u8 {
    if pos < DESKS {
        DESKS
    } else {
        pos + 1
    }
}

/// How the directors' table is set up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableSetup {
    /// [C] The players lay the counters in turn, each choosing colour and
    /// circle.
    #[default]
    Draft,
    /// The counters laid at random, as in [C]'s solo puzzle.
    Random,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub table: TableSetup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Setting up: lay a counter on a free circle.
    Place { circle: u8, color: Color },
    /// [C] Exchange two counters of the table (`a < b`, different colours).
    Swap { a: u8, b: u8 },
    /// [C] Enter an employee from outside onto a reception desk.
    Enter { color: Color, desk: u8 },
    /// Leave out the optional step at hand (exchange or entry).
    Skip,
    /// [C] Apply a promotion to the top employee at `from`.
    Promote { from: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Draft,
    Swap,
    Entry,
    Promote,
    Over,
}

/// What happened during a turn, for the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Swap { a: u8, b: u8 },
    Enter { color: Color, desk: u8 },
    Promote { color: Color, from: u8, to: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    Won {
        seat: Seat,
    },
    /// [`QUIET_ROUNDS`] rounds without an entry or a promotion.
    Quiet,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub players: u8,
    /// The counters on the table's circles.
    pub table: Vec<Option<Color>>,
    /// Each position's stack, bottom first.
    pub stacks: Vec<Vec<Color>>,
    /// Employees not yet entered, by colour.
    pub outside: [u8; 4],
    pub phase: Phase,
    pub turn: Seat,
    /// Turns in a row without an entry or a promotion.
    pub quiet: u32,
    /// The steps of the turn being played.
    pub steps: Vec<Step>,
    pub last_turn: Option<LastTurn>,
    pub end: Option<End>,
}

impl State {
    /// The seat playing `color`, if any (the others are neutral).
    pub fn owner(&self, color: Color) -> Option<Seat> {
        (color.index() < usize::from(self.players)).then_some(Seat(color.index() as u8))
    }

    /// Counters of `color` not yet on the table.
    pub fn supply(&self, color: Color) -> u8 {
        PER_COLOR - self.table.iter().filter(|&&c| c == Some(color)).count() as u8
    }

    /// Employees of `color` in the boardroom.
    pub fn arrived(&self, color: Color) -> u8 {
        self.stacks[usize::from(BOARDROOM)]
            .iter()
            .filter(|&&c| c == color)
            .count() as u8
    }

    /// [C] The colour "beneath" the top employee at `pos`: the employee
    /// directly under him, or his office when he stands alone. ❓ The
    /// office does not count under a stack.
    pub fn beneath(&self, pos: u8) -> Option<Color> {
        let stack = &self.stacks[usize::from(pos)];
        match stack.len() {
            0 => None,
            1 => room_color(pos),
            n => Some(stack[n - 2]),
        }
    }

    /// [C] Whether some pair of the table promotes the top employee at `pos`.
    pub fn promotable(&self, pos: u8) -> bool {
        let (Some(&top), Some(under)) = (self.stacks[usize::from(pos)].last(), self.beneath(pos))
        else {
            return false;
        };
        pos != BOARDROOM
            && self
                .table
                .chunks(2)
                .any(|pair| pair[0] == Some(top) && pair[1] == Some(under))
    }

    /// The positions whose top employee a promotion applies to.
    pub fn eligible(&self) -> Vec<u8> {
        (0..BOARDROOM).filter(|&p| self.promotable(p)).collect()
    }

    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.end.is_some() || seat != self.turn {
            return Vec::new();
        }
        match self.phase {
            Phase::Draft => (0..CIRCLES as u8)
                .filter(|&c| self.table[usize::from(c)].is_none())
                .flat_map(|circle| {
                    COLORS
                        .into_iter()
                        .filter(|&color| self.supply(color) > 0)
                        .map(move |color| Action::Place { circle, color })
                })
                .collect(),
            Phase::Swap => {
                let mut v = vec![Action::Skip];
                for a in 0..CIRCLES as u8 {
                    for b in a + 1..CIRCLES as u8 {
                        if self.table[usize::from(a)] != self.table[usize::from(b)] {
                            v.push(Action::Swap { a, b });
                        }
                    }
                }
                v
            }
            Phase::Entry => {
                let mut v = vec![Action::Skip];
                for color in COLORS {
                    if self.outside[color.index()] > 0 {
                        v.extend((0..DESKS).map(|desk| Action::Enter { color, desk }));
                    }
                }
                v
            }
            Phase::Promote => self
                .eligible()
                .into_iter()
                .map(|from| Action::Promote { from })
                .collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn place(&mut self, circle: u8, color: Color) {
        self.table[usize::from(circle)] = Some(color);
        self.turn = self.next_seat();
        if self.table.iter().all(Option::is_some) {
            // ❓ Play goes on in turn order: 12 counters among 2, 3 or 4
            // players bring the turn back to seat 0.
            self.phase = Phase::Swap;
        }
    }

    fn swap(&mut self, a: u8, b: u8) {
        self.table.swap(usize::from(a), usize::from(b));
        self.steps.push(Step::Swap { a, b });
        self.open_entry();
    }

    /// [C] The entry is optional; it is skipped when nobody is left outside.
    fn open_entry(&mut self) {
        if self.outside.iter().all(|&n| n == 0) {
            self.settle();
        } else {
            self.phase = Phase::Entry;
        }
    }

    fn enter(&mut self, color: Color, desk: u8) {
        self.outside[color.index()] -= 1;
        // [C] On top of an employee already there.
        self.stacks[usize::from(desk)].push(color);
        self.steps.push(Step::Enter { color, desk });
        self.settle();
    }

    /// [C] Promotions apply until none is left. A lone one is applied
    /// here; the player chooses the order when there are several.
    fn settle(&mut self) {
        while self.end.is_none() {
            match self.eligible()[..] {
                [] => return self.end_turn(),
                [only] => self.promote(only),
                _ => {
                    self.phase = Phase::Promote;
                    return;
                }
            }
        }
    }

    fn promote(&mut self, from: u8) {
        let color = self.stacks[usize::from(from)]
            .pop()
            .expect("a promotable stack has a top employee");
        let to = above(from);
        // [C] On top of the stack in the new office.
        self.stacks[usize::from(to)].push(color);
        self.steps.push(Step::Promote { color, from, to });
        // [C] Whoever's turn it is, the player of that colour wins with his
        // three employees in the boardroom. ❓ The game stops right there.
        if to == BOARDROOM && self.arrived(color) == PER_COLOR {
            if let Some(seat) = self.owner(color) {
                self.finish(End::Won { seat });
            }
        }
    }

    fn end_turn(&mut self) {
        let progress = self
            .steps
            .iter()
            .any(|s| matches!(s, Step::Enter { .. } | Step::Promote { .. }));
        self.quiet = if progress { 0 } else { self.quiet + 1 };
        self.last_turn = Some(LastTurn {
            seat: self.turn,
            steps: std::mem::take(&mut self.steps),
        });
        if self.quiet >= QUIET_ROUNDS * u32::from(self.players) {
            self.finish(End::Quiet);
        } else {
            self.turn = self.next_seat();
            self.phase = Phase::Swap;
        }
    }

    fn finish(&mut self, end: End) {
        self.end = Some(end);
        self.phase = Phase::Over;
        if !self.steps.is_empty() {
            self.last_turn = Some(LastTurn {
                seat: self.turn,
                steps: std::mem::take(&mut self.steps),
            });
        }
    }

    fn next_seat(&self) -> Seat {
        Seat((self.turn.0 + 1) % self.players)
    }
}

impl Game for Corporation {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "corporation",
            name: "Corporation",
            min_players: 2,
            max_players: 4,
            hidden_info: false,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "table",
            kind: OptionKind::Choice {
                values: &["Draft", "Random"],
                default: "Draft",
            },
        }]
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if !(2..=4).contains(&players) {
            return Err(Illegal::new("Corporation se joue de 2 à 4 joueurs."));
        }
        Ok(())
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let mut s = State {
            config: config.clone(),
            players,
            table: vec![None; CIRCLES],
            stacks: vec![Vec::new(); POSITIONS],
            outside: [PER_COLOR; 4],
            phase: Phase::Draft,
            // ❓ Seat 0 starts, the setting up as the game.
            turn: Seat(0),
            quiet: 0,
            steps: Vec::new(),
            last_turn: None,
            end: None,
        };
        if config.table == TableSetup::Random {
            let mut counters: Vec<Color> = COLORS
                .iter()
                .flat_map(|&c| [c; PER_COLOR as usize])
                .collect();
            // ChaCha, not SmallRng: a browser log must replay natively.
            counters.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
            s.table = counters.into_iter().map(Some).collect();
            s.phase = Phase::Swap;
        }
        s
    }

    fn pending(s: &State) -> Pending {
        if s.end.is_some() {
            Pending::none()
        } else {
            Pending::one(s.turn)
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.end.is_some() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if seat != s.turn {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !s.moves(seat).contains(action) {
            return Err(Illegal::new(match s.phase {
                Phase::Draft => "Pose un jeton qui reste sur un cercle libre.",
                Phase::Swap => "Échange deux jetons de couleurs différentes, ou passe.",
                Phase::Entry => "Fais entrer un employé qui attend dehors, ou passe.",
                Phase::Promote | Phase::Over => "Cette promotion ne s'applique pas.",
            }));
        }
        match *action {
            Action::Place { circle, color } => s.place(circle, color),
            Action::Swap { a, b } => s.swap(a, b),
            Action::Skip if s.phase == Phase::Swap => s.open_entry(),
            Action::Skip => s.settle(),
            Action::Enter { color, desk } => s.enter(color, desk),
            Action::Promote { from } => {
                s.promote(from);
                s.settle();
            }
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// [C] The winner first; ❓ the others by employees in the boardroom.
    /// The quiet limit draws the game.
    fn outcome(s: &State) -> Option<Outcome> {
        let ranks = match s.end? {
            End::Quiet => vec![0; usize::from(s.players)],
            End::Won { seat } => {
                let arrived: Vec<u8> = COLORS[..usize::from(s.players)]
                    .iter()
                    .map(|&c| s.arrived(c))
                    .collect();
                (0..s.players)
                    .map(|i| {
                        if i == seat.0 {
                            0
                        } else {
                            1 + (0..s.players)
                                .filter(|&j| {
                                    j != seat.0 && arrived[usize::from(j)] > arrived[usize::from(i)]
                                })
                                .count() as u8
                        }
                    })
                    .collect()
            }
        };
        Some(Outcome::Ranked(ranks))
    }

    /// Leave out the optional steps; lay the first counter allowed; apply
    /// the lowest promotion.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        s.moves(seat).into_iter().next()
    }
}

impl Determinize for Corporation {
    /// Nothing is hidden.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, _rng: &mut R) -> State {
        s.clone()
    }
}
