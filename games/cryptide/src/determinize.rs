//! The others' hints and the habitat, redrawn among the puzzles the map's
//! tokens still allow.

use ludo_core::{Determinize, Seat};
use rand::Rng;

use crate::clues::habitat;
use crate::knowledge::{possible, sample};
use crate::map::{Cell, Mask};
use crate::rules::{Cryptide, Phase, State};

/// Nodes walked per redraw: `determinize` runs at every step of the
/// conformance, and only for search, so a smaller budget than the AI's.
const DETERMINIZE_CAP: u64 = 20_000;

/// Random draws from the possible lists when the walk found nothing.
const REJECTION_TRIES: usize = 2_000;

fn draw<R: Rng + ?Sized>(rng: &mut R, n: usize) -> usize {
    rng.gen_range(0..n as u32) as usize
}

impl Determinize for Cryptide {
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        // Over: everything is public.
        if s.phase == Phase::Over {
            return s.clone();
        }
        let table = s.table();
        let Some(mine) = s
            .puzzle
            .hints
            .get(observer.index())
            .and_then(|h| table.index_of(h))
        else {
            return s.clone();
        };
        let chosen = sample(
            table,
            &s.tokens,
            observer,
            mine,
            s.players,
            DETERMINIZE_CAP,
            rng,
        )
        .or_else(|| {
            let lists: Vec<Vec<usize>> = (0..s.players)
                .map(|p| {
                    if p == observer.0 {
                        vec![mine]
                    } else {
                        possible(table, &s.tokens, Seat(p))
                    }
                })
                .collect();
            let pick = |rng: &mut R| -> Vec<usize> {
                lists.iter().map(|l| l[draw(rng, l.len())]).collect()
            };
            (0..REJECTION_TRIES)
                .find_map(|_| {
                    let hints = pick(rng);
                    habitat(table, &hints).map(|home| (hints, home))
                })
                .or_else(|| {
                    // Each hint still agrees with its seat's tokens; the
                    // habitat is any space they share, else the observer's.
                    let hints = pick(rng);
                    let all = hints.iter().fold(Mask::full(), |m, &i| m & table.masks[i]);
                    let home = all
                        .iter()
                        .next()
                        .or_else(|| table.masks[mine].iter().next())
                        .unwrap_or(Cell(0));
                    Some((hints, home))
                })
        })
        .expect("a fallback always exists");
        let mut d = s.clone();
        d.puzzle.hints = chosen.0.iter().map(|&i| table.hints[i]).collect();
        d.puzzle.habitat = chosen.1;
        d
    }
}
