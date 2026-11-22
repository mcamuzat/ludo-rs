//! Riddles for practice [U 2026-10-22]: a table of AIs plays until one of
//! them can name the habitat from the tokens and its own hint; the game
//! stops there and the player, given that hint, must find the space.

use ludo_core::ai::Level;
use ludo_core::{Game, Heuristics, Seat};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::clues::{hints, Hint, Mode};
use crate::knowledge::{marks, possible, query, Inference, CAP};
use crate::map::{Cell, Square};
use crate::rules::{Config, Cryptide, Event, Phase, State, Tokens};

/// Games tried before giving up (one is nearly always enough: a game ends
/// on a search, and an AI searches once it knows).
const ATTEMPTS: u64 = 32;
/// Games tried at level Hint: at three seats, a seat's own tokens pin its
/// hint in about one game in forty (a game costs ~3 ms natively).
const HINT_ATTEMPTS: u64 = 400;
/// Actions in one game at most.
const MAX_MOVES: usize = 2_000;

/// What the player is asked to find [U 2026-11-22].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiddleLevel {
    /// Every hint given, no game played: find the one space.
    Warmup,
    /// One seat's tokens leave it a single hint: name it.
    Hint,
    /// The solver knows, and each other seat's hint is (nearly) pinned.
    Easy,
    /// The solver knows: find the space in its place.
    Full,
}

/// The token that rules a hint out: a disc outside it, or a cube inside.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Mark {
    Disc,
    Cube,
}

/// A hint of the mode that the target's tokens rule out (level Hint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Elimination {
    pub hint: Hint,
    pub cell: Cell,
    pub mark: Mark,
}

/// A riddle: a map, the tokens of a table of AIs stopped at the right
/// moment (none at the warm-up), what the player is given and the answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Riddle {
    pub level: RiddleLevel,
    pub players: u8,
    pub mode: Mode,
    pub squares: Vec<Square>,
    pub tokens: Vec<Tokens>,
    pub log: Vec<Event>,
    /// The seat in focus: whose hint is given (Easy, Full), whose hint is
    /// sought (Hint), seat 0 at the warm-up.
    pub solver: Seat,
    /// `hints[solver]`.
    pub hint: Hint,
    /// The answer of the "find the space" levels.
    pub habitat: Cell,
    /// Every seat's hint, shown with the answer.
    pub hints: Vec<Hint>,
    /// The hints shown to the player, by seat.
    pub given: Vec<Option<Hint>>,
    /// By seat, the spaces its hint keeps.
    pub hint_cells: Vec<Vec<Cell>>,
    /// What the solver can tell of each other seat's hint (empty for the
    /// solver, and for every seat outside Easy and Full): the hints left in
    /// some combination naming one space.
    pub deduction: Vec<Vec<Hint>>,
    /// Level Hint: for every other hint of the mode, a token of the target's
    /// that rules it out. Empty at the other levels.
    pub eliminations: Vec<Elimination>,
    /// For the player aid: every hint of the mode.
    pub mode_hints: Vec<Hint>,
}

/// Where a game stops: the seat in focus and, for the solver levels, what
/// it can tell of the others.
struct Stop {
    seat: Seat,
    per_seat: Vec<Vec<usize>>,
}

/// The seat that can name the habitat now, if any, with what it can tell
/// of the others. "Can name" is [B]'s reasoning: every combination of
/// distinct hints agreeing with the tokens points to the same single
/// space. It is what a player works out at the table, without the
/// generator's own guarantees (`Inference::Puzzle`).
fn solver(s: &State) -> Option<Stop> {
    (0..s.players)
        .map(|p| Seat((s.turn.0 + p) % s.players))
        .find_map(|seat| {
            let k = query(s, seat, Inference::Bot, CAP)?.combos();
            (k.complete && k.cells.len() == 1).then_some(Stop {
                seat,
                per_seat: k.per_seat,
            })
        })
}

/// The first seat, from the one to play, whose own tokens leave it a
/// single hint of the mode (the others' tokens are not used).
fn pinned_seat(s: &State) -> Option<Stop> {
    let table = s.table();
    (0..s.players)
        .map(|p| Seat((s.turn.0 + p) % s.players))
        .find(|&seat| possible(table, &s.tokens, seat).len() == 1)
        .map(|seat| Stop {
            seat,
            per_seat: vec![Vec::new(); usize::from(s.players)],
        })
}

/// For every hint of the mode but `seat`'s, the first of its discs outside
/// the hint, else the first of its cubes inside (in space order).
fn eliminations(s: &State, seat: Seat) -> Vec<Elimination> {
    let table = s.table();
    let (discs, cubes) = marks(&s.tokens, seat);
    let own = s.puzzle.hints[seat.index()];
    table
        .hints
        .iter()
        .zip(&table.masks)
        .filter(|(h, _)| **h != own)
        .filter_map(|(&hint, &m)| {
            let disc = discs
                .iter()
                .find(|&c| !m.contains(c))
                .map(|cell| Elimination {
                    hint,
                    cell,
                    mark: Mark::Disc,
                });
            disc.or_else(|| {
                cubes
                    .iter()
                    .find(|&c| m.contains(c))
                    .map(|cell| Elimination {
                        hint,
                        cell,
                        mark: Mark::Cube,
                    })
            })
        })
        .collect()
}

fn riddle_of(level: RiddleLevel, s: &State, stop: &Stop) -> Riddle {
    let table = s.table();
    let board = s.puzzle.board();
    let seat = stop.seat;
    let mine = s.puzzle.hints.clone();
    let given = match level {
        RiddleLevel::Warmup => mine.iter().copied().map(Some).collect(),
        RiddleLevel::Hint => vec![None; mine.len()],
        RiddleLevel::Easy | RiddleLevel::Full => (0..mine.len())
            .map(|p| (p == seat.index()).then_some(mine[p]))
            .collect(),
    };
    Riddle {
        level,
        players: s.players,
        mode: s.mode,
        squares: board.squares.clone(),
        tokens: s.tokens.clone(),
        log: s.log.clone(),
        solver: seat,
        hint: mine[seat.index()],
        habitat: s.puzzle.habitat,
        given,
        hint_cells: mine
            .iter()
            .map(|h| h.cells(&board).iter().collect())
            .collect(),
        deduction: stop
            .per_seat
            .iter()
            .map(|list| list.iter().map(|&i| table.hints[i]).collect())
            .collect(),
        eliminations: if level == RiddleLevel::Hint {
            eliminations(s, seat)
        } else {
            Vec::new()
        },
        hints: mine,
        mode_hints: hints(s.mode),
    }
}

/// One game of Hard AIs, stopped once `stop` finds its moment; `None` if
/// the game ended first.
fn play(
    mode: Mode,
    players: u8,
    seed: u64,
    stop: &dyn Fn(&State) -> Option<Stop>,
) -> Option<(State, Stop)> {
    let mut s = Cryptide::new(&Config { mode }, players, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    for _ in 0..MAX_MOVES {
        // During the setup nobody knows anything yet, and the walk is slow.
        if !matches!(s.phase, Phase::Setup | Phase::Over) {
            if let Some(found) = stop(&s) {
                return Some((s, found));
            }
        }
        if s.phase == Phase::Over {
            return None;
        }
        let seat = s.turn;
        let action = Cryptide::policy_action(&s, seat, Level::Hard, &mut rng)?;
        Cryptide::apply(&mut s, seat, &action).ok()?;
    }
    None
}

fn seeds(seed: u64, attempts: u64) -> impl Iterator<Item = u64> {
    (0..attempts).map(move |k| seed.wrapping_add(k.wrapping_mul(0x9E37_79B9)))
}

/// The first game, from `seed` on, that `stop` cuts short.
fn found(
    attempts: u64,
    level: RiddleLevel,
    mode: Mode,
    players: u8,
    seed: u64,
    stop: &dyn Fn(&State) -> Option<Stop>,
) -> Option<Riddle> {
    seeds(seed, attempts).find_map(|seed| {
        play(mode, players, seed, stop).map(|(s, stop)| riddle_of(level, &s, &stop))
    })
}

/// A riddle of `level` for `players` seats (3 to 5), from `seed`.
pub fn riddle(level: RiddleLevel, mode: Mode, players: u8, seed: u64) -> Option<Riddle> {
    match level {
        RiddleLevel::Warmup => {
            // A fresh game holds the puzzle and no token yet.
            let s = Cryptide::new(&Config { mode }, players, seed);
            let stop = Stop {
                seat: Seat(0),
                per_seat: vec![Vec::new(); usize::from(players)],
            };
            Some(riddle_of(level, &s, &stop))
        }
        RiddleLevel::Hint => found(HINT_ATTEMPTS, level, mode, players, seed, &pinned_seat),
        RiddleLevel::Full => found(ATTEMPTS, level, mode, players, seed, &solver),
        RiddleLevel::Easy => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_solver_names_the_habitat_alone() {
        for mode in [Mode::Normal, Mode::Advanced] {
            for players in 3u8..=5 {
                for seed in 0..3u64 {
                    let (s, stop) = seeds(seed, ATTEMPTS)
                        .find_map(|seed| play(mode, players, seed, &solver))
                        .expect("a riddle");
                    let k = query(&s, stop.seat, Inference::Bot, CAP).unwrap().combos();
                    assert!(k.complete);
                    assert_eq!(
                        k.cells.iter().map(|c| c.0).collect::<Vec<_>>(),
                        [s.puzzle.habitat]
                    );
                    assert_ne!(s.phase, Phase::Over);
                }
            }
        }
    }
}
