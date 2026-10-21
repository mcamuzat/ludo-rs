//! Black-Pot (Cogitas): three dice, four operations, and lines of five.
//! Rules and their sources: RULES.md.

pub mod board;
pub mod calc;
pub mod lines;
pub mod testing;
mod view;

pub use view::{Pawn, TeamView, View};

use std::collections::HashSet;

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use board::{cells, CellId};
use calc::{announce, dice_used, eval, values, Expr};
use lines::{new_alignments, Alignment};

pub struct BlackPot;

/// [R] Pawns of each side (a team shares its 40 at four players).
pub const PAWNS: u8 = 40;
/// ludo-rs addition, not in [R]: the game stops after this many rounds of
/// the table in a row without a pawn placed.
pub const QUIET_ROUNDS: u32 = 10;
/// [R] What each pawn not played is worth at the end.
pub const UNPLAYED_POINTS: u32 = 10;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {}

/// [R] Some of the dice combined into one number, and the cell it takes
/// (`None` exactly when the number is 0: nothing is placed).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Group {
    pub expr: Expr,
    pub cell: Option<CellId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// The three dice split into groups, each used once.
    Play { groups: Vec<Group> },
    /// [U] « Je ne trouve pas » : a move exists but the player does not see
    /// it, and passes without placing a pawn.
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayedGroup {
    /// [R] The calculation announced aloud.
    pub text: String,
    pub value: u32,
    pub cell: Option<CellId>,
}

/// What a seat did with its last roll.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnRecord {
    Played {
        dice: [u8; 3],
        groups: Vec<PlayedGroup>,
        /// Points of the lines this move completed.
        points: u32,
    },
    NotFound {
        dice: [u8; 3],
    },
    /// No move existed: the turn passed by itself.
    Passed {
        dice: [u8; 3],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    /// [R] This team placed its last pawn.
    AllPlaced { team: u8 },
    /// No pawn placed for `QUIET_ROUNDS` rounds of the table.
    QuietLimit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    /// The team of the pawn on each cell.
    pub owner: Vec<Option<u8>>,
    /// `counted[cell][dir]`: the pawn already counts in a line of `dir`.
    pub counted: Vec<[bool; 4]>,
    pub pawns_left: [u8; 2],
    pub alignments: Vec<Alignment>,
    /// The current roll, public.
    pub dice: [u8; 3],
    pub roller: Seat,
    /// Who still plays this roll, the one to play first.
    pub queue: Vec<Seat>,
    /// Each seat's last turn.
    pub last: Vec<Option<TurnRecord>>,
    /// Turns in a row without a pawn placed.
    pub quiet: u32,
    /// Turns (moves and passes) since the start.
    pub turns: u32,
    /// Seed of the rolls: never in a view.
    pub seed: u64,
    /// Rolls made so far.
    pub rolls: u32,
    pub end: Option<End>,
}

/// The team of `seat`: each player alone at two, [R] partners facing each
/// other at four (seats 0 and 2 against 1 and 3).
pub fn team_of(players: u8, seat: Seat) -> u8 {
    if players == 4 {
        seat.0 % 2
    } else {
        seat.0
    }
}

/// Roll number `k` of a game seeded with `seed`.
fn roll(seed: u64, k: u32) -> [u8; 3] {
    let mut rng =
        ChaCha8Rng::seed_from_u64(seed ^ u64::from(k).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    [0; 3].map(|_| (rng.gen::<u32>() % 6 + 1) as u8)
}

/// [R] What a team has: its lines, plus 10 points per pawn not played.
pub fn score(s: &State, team: u8) -> u32 {
    s.alignments
        .iter()
        .filter(|a| a.team == team)
        .map(|a| a.points)
        .sum::<u32>()
        + UNPLAYED_POINTS * u32::from(s.pawns_left[usize::from(team)])
}

/// [R] The ways to split the three dice into groups.
const PARTITIONS: [&[&[u8]]; 5] = [
    &[&[0, 1, 2]],
    &[&[0, 1], &[2]],
    &[&[0, 2], &[1]],
    &[&[1, 2], &[0]],
    &[&[0], &[1], &[2]],
];

/// The moves of `seat`, one per set of cells taken (the move placing no
/// pawn included), each with one calculation. Empty when `seat` is not to
/// play.
pub fn plays(s: &State, seat: Seat) -> Vec<Action> {
    if s.end.is_some() || s.queue.first() != Some(&seat) {
        return Vec::new();
    }
    let left = usize::from(s.pawns_left[usize::from(team_of(s.players, seat))]);
    let mut free: Vec<Vec<CellId>> = vec![Vec::new(); 25];
    for (id, cell) in cells().iter().enumerate() {
        if s.owner[id].is_none() {
            free[usize::from(cell.value)].push(id as CellId);
        }
    }
    let mut seen: HashSet<Vec<CellId>> = HashSet::new();
    let mut found = Vec::new();
    for partition in PARTITIONS {
        let options: Vec<Vec<(u32, Expr)>> = partition
            .iter()
            .map(|group| values(&s.dice, group).into_iter().collect())
            .collect();
        let mut chosen = Vec::new();
        assign(&options, &free, left, &mut chosen, &mut |groups| {
            let mut key: Vec<CellId> = groups.iter().filter_map(|g| g.cell).collect();
            key.sort_unstable();
            if seen.insert(key) {
                found.push(Action::Play {
                    groups: groups.to_vec(),
                });
            }
        });
    }
    found
}

/// Every way to give the groups still in `options` a value and a free cell
/// of that value, distinct from those in `chosen`, placing at most `left`
/// pawns in all.
fn assign(
    options: &[Vec<(u32, Expr)>],
    free: &[Vec<CellId>],
    left: usize,
    chosen: &mut Vec<Group>,
    found: &mut dyn FnMut(&[Group]),
) {
    let Some((first, rest)) = options.split_first() else {
        found(chosen);
        return;
    };
    for (value, expr) in first {
        if *value == 0 {
            chosen.push(Group {
                expr: expr.clone(),
                cell: None,
            });
            assign(rest, free, left, chosen, found);
            chosen.pop();
            continue;
        }
        if left == 0 {
            continue;
        }
        let Some(cells) = free.get(*value as usize) else {
            continue;
        };
        for &cell in cells {
            if chosen.iter().any(|g| g.cell == Some(cell)) {
                continue;
            }
            chosen.push(Group {
                expr: expr.clone(),
                cell: Some(cell),
            });
            assign(rest, free, left - 1, chosen, found);
            chosen.pop();
        }
    }
}

impl State {
    fn team(&self, seat: Seat) -> u8 {
        team_of(self.players, seat)
    }

    /// [R] `roller` throws the dice. At two players they play their own
    /// roll [U]; at four, the next three play it first and the roller last.
    fn start_roll(&mut self, roller: Seat) {
        self.roller = roller;
        self.dice = roll(self.seed, self.rolls);
        self.rolls += 1;
        self.queue = if self.players == 4 {
            (1..=4).map(|k| Seat((roller.0 + k) % 4)).collect()
        } else {
            vec![roller]
        };
    }

    /// Passes the turn of each player who has no move, rolls again when
    /// everyone has played the roll, and stops the game on the quiet limit.
    fn settle(&mut self) {
        while self.end.is_none() {
            if self.quiet >= QUIET_ROUNDS * u32::from(self.players) {
                self.end = Some(End::QuietLimit);
                self.queue.clear();
                return;
            }
            let Some(&seat) = self.queue.first() else {
                self.start_roll(Seat((self.roller.0 + 1) % self.players));
                continue;
            };
            if !plays(self, seat).is_empty() {
                return;
            }
            // [R] A player who finds nothing passes.
            self.last[seat.index()] = Some(TurnRecord::Passed { dice: self.dice });
            self.quiet += 1;
            self.queue.remove(0);
            self.turns += 1;
        }
    }

    fn finish_turn(&mut self, seat: Seat) {
        self.queue.remove(0);
        self.turns += 1;
        let team = self.team(seat);
        if self.pawns_left[usize::from(team)] == 0 {
            // [R] The game stops as soon as a player places its last pawn.
            self.end = Some(End::AllPlaced { team });
            self.queue.clear();
        } else {
            self.settle();
        }
    }

    /// Checks a move without changing anything; returns each group's value.
    fn check_play(&self, seat: Seat, groups: &[Group]) -> Result<Vec<u32>, Illegal> {
        let mut used: Vec<u8> = groups.iter().flat_map(|g| dice_used(&g.expr)).collect();
        used.sort_unstable();
        // [R] The three numbers rolled must all be used.
        if used != [0, 1, 2] {
            return Err(Illegal::new(
                "Il faut employer chacun des 3 dés, une seule fois.",
            ));
        }
        let mut taken: Vec<CellId> = Vec::new();
        let mut values = Vec::new();
        for group in groups {
            let value = eval(&group.expr, &self.dice)?;
            match group.cell {
                None if value != 0 => {
                    return Err(Illegal::new(format!(
                        "Ce calcul donne {value} : choisis une case."
                    )))
                }
                None => {}
                Some(_) if value == 0 => {
                    return Err(Illegal::new("Un calcul qui donne 0 ne pose pas de pion."))
                }
                Some(cell) => {
                    let Some(info) = cells().get(usize::from(cell)) else {
                        return Err(Illegal::new("Cette case n'existe pas."));
                    };
                    if self.owner[usize::from(cell)].is_some() {
                        return Err(Illegal::new("Cette case est déjà occupée."));
                    }
                    if u32::from(info.value) != value {
                        return Err(Illegal::new(format!(
                            "{}, pas {}",
                            announce(&group.expr, &self.dice)?,
                            info.value
                        )));
                    }
                    if taken.contains(&cell) {
                        return Err(Illegal::new("Deux calculs visent la même case."));
                    }
                    taken.push(cell);
                }
            }
            values.push(value);
        }
        let left = self.pawns_left[usize::from(self.team(seat))];
        if taken.len() > usize::from(left) {
            return Err(Illegal::new(format!("Il ne te reste que {left} pions.")));
        }
        Ok(values)
    }

    fn play(&mut self, seat: Seat, groups: &[Group], values: Vec<u32>) -> Result<(), Illegal> {
        let team = self.team(seat);
        let mut played = Vec::new();
        for (group, value) in groups.iter().zip(values) {
            played.push(PlayedGroup {
                text: announce(&group.expr, &self.dice)?,
                value,
                cell: group.cell,
            });
        }
        let placed: Vec<CellId> = groups.iter().filter_map(|g| g.cell).collect();
        for &cell in &placed {
            self.owner[usize::from(cell)] = Some(team);
        }
        self.pawns_left[usize::from(team)] -= placed.len() as u8;
        // [R] A line of five is turned over and paid at once.
        let found = new_alignments(&self.owner, &self.counted, team);
        for line in &found {
            for &cell in &line.cells {
                self.counted[usize::from(cell)][line.dir.index()] = true;
            }
        }
        let points = found.iter().map(|a| a.points).sum();
        self.alignments.extend(found);
        self.last[seat.index()] = Some(TurnRecord::Played {
            dice: self.dice,
            groups: played,
            points,
        });
        self.quiet = if placed.is_empty() { self.quiet + 1 } else { 0 };
        self.finish_turn(seat);
        Ok(())
    }
}

/// Ranks: the higher score first; partners share their rank, ties share.
fn ranks(s: &State) -> Vec<u8> {
    let scores = [score(s, 0), score(s, 1)];
    (0..s.players)
        .map(|seat| {
            let mine = scores[usize::from(team_of(s.players, Seat(seat)))];
            scores.iter().filter(|&&other| other > mine).count() as u8
        })
        .collect()
}

impl Game for BlackPot {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "black-pot",
            name: "Black-Pot",
            min_players: 2,
            max_players: 4,
            hidden_info: true,
            clock: true,
        }
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if players == 3 {
            return Err(Illegal::new("Black-Pot se joue à 2 ou à 4."));
        }
        Ok(())
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        // [R] The first to throw is drawn (on the table, the highest die).
        let first = ChaCha8Rng::seed_from_u64(seed).gen::<u32>() % u32::from(players);
        let mut s = State {
            players,
            owner: vec![None; cells().len()],
            counted: vec![[false; 4]; cells().len()],
            pawns_left: [PAWNS; 2],
            alignments: Vec::new(),
            dice: [1; 3],
            roller: Seat(0),
            queue: Vec::new(),
            last: vec![None; usize::from(players)],
            quiet: 0,
            turns: 0,
            seed,
            rolls: 0,
            end: None,
        };
        s.start_roll(Seat(first as u8));
        s.settle();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.queue.first() {
            Some(&seat) if s.end.is_none() => Pending::one(seat),
            _ => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        let mut legal = plays(s, seat);
        if !legal.is_empty() {
            legal.push(Action::NotFound);
        }
        legal
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.end.is_some() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.queue.first() != Some(&seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match action {
            Action::Play { groups } => {
                let values = s.check_play(seat, groups)?;
                let mut next = s.clone();
                next.play(seat, groups, values)?;
                *s = next;
            }
            Action::NotFound => {
                if plays(s, seat).is_empty() {
                    return Err(Illegal::new(
                        "Aucun coup n'existe : le tour passe tout seul.",
                    ));
                }
                s.last[seat.index()] = Some(TurnRecord::NotFound { dice: s.dice });
                s.quiet += 1;
                s.finish_turn(seat);
            }
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.end.map(|_| Outcome::Ranked(ranks(s)))
    }

    /// [U] Time out: « Je ne trouve pas », which shows no move.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        (!plays(s, seat).is_empty()).then_some(Action::NotFound)
    }
}

impl Determinize for BlackPot {
    /// Only the rolls to come are hidden: their seed is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut s = s.clone();
        s.seed = rng.gen();
        s
    }
}
