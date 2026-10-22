//! Riddles for practice [U 2026-10-22]: a table of AIs plays until one of
//! them can name the habitat from the tokens and its own hint; the game
//! stops there and the player, given that hint, must find the space.

use ludo_core::ai::Level;
use ludo_core::{Game, Heuristics, Seat};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use crate::clues::{hints, Hint, Mode};
use crate::knowledge::{query, Inference, CAP};
use crate::map::{Cell, Square};
use crate::rules::{Config, Cryptide, Event, Phase, State, Tokens};

/// Games tried before giving up (one is nearly always enough: a game ends
/// on a search, and an AI searches once it knows).
const ATTEMPTS: u64 = 32;
/// Actions in one game at most.
const MAX_MOVES: usize = 2_000;

/// A table stopped as soon as `solver` knows the habitat.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Riddle {
    pub players: u8,
    pub mode: Mode,
    pub squares: Vec<Square>,
    pub tokens: Vec<Tokens>,
    pub log: Vec<Event>,
    /// The seat whose hint the player is given.
    pub solver: Seat,
    pub hint: Hint,
    /// The answer.
    pub habitat: Cell,
    /// Every seat's hint, shown with the answer.
    pub hints: Vec<Hint>,
    /// What the solver can tell of each other seat's hint (empty for the
    /// solver): the hints left in some combination naming one space.
    pub deduction: Vec<Vec<Hint>>,
    /// For the player aid: every hint of the mode.
    pub mode_hints: Vec<Hint>,
}

/// The seat that can name the habitat now, if any, with what it can tell
/// of the others. "Can name" is [B]'s reasoning: every combination of
/// distinct hints agreeing with the tokens points to the same single
/// space. It is what a player works out at the table, without the
/// generator's own guarantees (`Inference::Puzzle`).
fn solver(s: &State) -> Option<(Seat, Vec<Vec<usize>>)> {
    (0..s.players)
        .map(|p| Seat((s.turn.0 + p) % s.players))
        .find_map(|seat| {
            let k = query(s, seat, Inference::Bot, CAP)?.combos();
            (k.complete && k.cells.len() == 1).then_some((seat, k.per_seat))
        })
}

fn riddle_of(s: &State, seat: Seat, per_seat: &[Vec<usize>]) -> Riddle {
    let table = s.table();
    Riddle {
        players: s.players,
        mode: s.mode,
        squares: s.puzzle.board().squares,
        tokens: s.tokens.clone(),
        log: s.log.clone(),
        solver: seat,
        hint: s.puzzle.hints[seat.index()],
        habitat: s.puzzle.habitat,
        hints: s.puzzle.hints.clone(),
        deduction: per_seat
            .iter()
            .map(|list| list.iter().map(|&i| table.hints[i]).collect())
            .collect(),
        mode_hints: hints(s.mode),
    }
}

/// One game of Hard AIs, stopped once a seat knows: the table, the seat
/// and what it can tell of the others; `None` if the game ended first.
fn play(mode: Mode, players: u8, seed: u64) -> Option<(State, Seat, Vec<Vec<usize>>)> {
    let mut s = Cryptide::new(&Config { mode }, players, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    for _ in 0..MAX_MOVES {
        // During the setup nobody knows anything yet, and the walk is slow.
        if !matches!(s.phase, Phase::Setup | Phase::Over) {
            if let Some((seat, per_seat)) = solver(&s) {
                return Some((s, seat, per_seat));
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

fn seeds(seed: u64) -> impl Iterator<Item = u64> {
    (0..ATTEMPTS).map(move |k| seed.wrapping_add(k.wrapping_mul(0x9E37_79B9)))
}

/// A riddle for `players` seats (3 to 5), from `seed`.
pub fn riddle(mode: Mode, players: u8, seed: u64) -> Option<Riddle> {
    seeds(seed).find_map(|seed| {
        play(mode, players, seed).map(|(s, seat, per_seat)| riddle_of(&s, seat, &per_seat))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_solver_names_the_habitat_alone() {
        for mode in [Mode::Normal, Mode::Advanced] {
            for players in 3u8..=5 {
                for seed in 0..3u64 {
                    let (s, seat, _) = seeds(seed)
                        .find_map(|seed| play(mode, players, seed))
                        .expect("a riddle");
                    let k = query(&s, seat, Inference::Bot, CAP).unwrap().combos();
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
