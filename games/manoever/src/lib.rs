//! Das glorreiche Manöver (Roland Siegers, Schmidt 1992): a race of 84
//! squares where a soldier goes the farther the further back it is and the
//! higher it stands. Rules and their sources: RULES.md.

pub mod board;
mod heuristics;
mod moves;
pub mod testing;
mod view;

pub use moves::{bonus, destinations, find, height, max_move, position, room, scores, to_move};
pub use view::View;

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use board::{Square, GRAYS, LAST};

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
        // The next seat with a soldier waiting, else the march opens.
        match (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| self.waiting[s.index()] > 0)
        {
            Some(next) => self.turn = Some(next),
            // Seat 0 opens the march, or the first after it that can move.
            None => self.pass_from(Seat(self.players - 1)),
        }
    }

    /// Gives the turn to the first seat after `from` (`from` itself last)
    /// that has a soldier to move; [K] the others pass.
    fn pass_from(&mut self, from: Seat) {
        self.moved.clear();
        let next = (1..=self.players)
            .map(|k| Seat((from.0 + k) % self.players))
            .find(|&s| !to_move(self, s).is_empty());
        match next {
            Some(seat) => self.turn = Some(seat),
            // Cannot happen (see RULES.md, « Fin »): kept as a guard.
            None => self.finish(),
        }
    }

    fn finish(&mut self) {
        self.over = true;
        self.turn = None;
    }

    /// Plays a legal move of `soldier` of `seat` to `to`.
    fn advance(&mut self, seat: Seat, number: u8, to: Dest) {
        let soldier = Soldier { seat, number };
        let (from, _) = find(self, soldier).expect("a legal move has its soldier");
        self.stacks[usize::from(from)].pop();
        let mut record = MoveRecord {
            number,
            from,
            to,
            gray: None,
            place: None,
        };
        match to {
            Dest::Square(sq) => {
                self.stacks[usize::from(sq)].push(soldier);
                // [R] The first to land on a gray piece wins it.
                if let Some(k) = GRAYS.iter().position(|&(g, _)| g == sq) {
                    if self.grays[k].is_none() {
                        self.grays[k] = Some(seat);
                        record.gray = Some(GRAYS[k].1);
                    }
                }
            }
            Dest::Barracks => record.place = Some(self.house(soldier)),
        }
        self.turn_log.push(record);
        self.moved.push(number);
        // [R] Once place II is taken, the last soldier goes to place I.
        if self.barracks.iter().filter(|p| p.is_some()).count() == 11 {
            if let Some(last) = (1..=LAST).find_map(|sq| self.stacks[usize::from(sq)].pop()) {
                self.house(last);
            }
            self.last_turns[seat.index()] = std::mem::take(&mut self.turn_log);
            self.finish();
        } else if to_move(self, seat).is_empty() {
            self.last_turns[seat.index()] = std::mem::take(&mut self.turn_log);
            self.pass_from(seat);
        }
    }

    /// [R] A soldier home takes the highest free place; returns it (1..=12).
    fn house(&mut self, soldier: Soldier) -> u8 {
        let i = self
            .barracks
            .iter()
            .rposition(Option::is_none)
            .expect("a soldier on the track has a free place");
        self.barracks[i] = Some(soldier);
        i as u8 + 1
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
        to_move(s, seat)
            .into_iter()
            .flat_map(|number| {
                destinations(s, Soldier { seat, number })
                    .into_iter()
                    .map(move |to| Action::Move {
                        soldier: number,
                        to,
                    })
            })
            .collect()
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
            Action::Move { soldier, to } => {
                if s.in_setup() || !Self::legal_actions(s, seat).contains(action) {
                    return Err(Illegal::new("Ce soldat ne peut pas aller là."));
                }
                s.advance(seat, soldier, to);
            }
        }
        s.moves += 1;
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// Rolls, or sends the first soldier to move as far as it can.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        if s.turn != Some(seat) {
            return None;
        }
        if s.in_setup() {
            return Some(Action::Roll);
        }
        let number = *to_move(s, seat).first()?;
        let to = *destinations(s, Soldier { seat, number }).last()?;
        Some(Action::Move {
            soldier: number,
            to,
        })
    }

    /// [R] Highest total wins; ❓ ties share their rank.
    fn outcome(s: &State) -> Option<Outcome> {
        if !s.over {
            return None;
        }
        let totals = scores(s);
        Some(Outcome::Ranked(
            totals
                .iter()
                .map(|&mine| totals.iter().filter(|&&t| t > mine).count() as u8)
                .collect(),
        ))
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
