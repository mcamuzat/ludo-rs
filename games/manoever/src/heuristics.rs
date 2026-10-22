use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::board::{disc, Square, GRAYS};
use crate::{scores, Action, Dest, Manoever, State};

/// Simulated actions after which a simulation is scored as it stands.
const HORIZON: u32 = 200;

/// Height × bonus a soldier would have on landing at `sq`: what its next
/// moves get multiplied by.
fn lift(s: &State, sq: Square) -> u8 {
    let below = s.stacks[usize::from(sq)].len();
    let height = below as u8 + 1 + u8::from(disc(sq).is_some());
    let bonus = if below == 0 { crate::bonus(sq, 0) } else { 1 };
    height * bonus
}

fn wins_gray(s: &State, sq: Square) -> bool {
    GRAYS
        .iter()
        .zip(s.grays)
        .any(|(&(g, _), owner)| g == sq && owner.is_none())
}

/// Points so far plus, for each soldier still on the track, a share of an
/// average place (6) in proportion to the way it has come.
fn estimate(s: &State) -> Vec<f64> {
    let mut out: Vec<f64> = scores(s).into_iter().map(f64::from).collect();
    for (sq, stack) in s.stacks.iter().enumerate() {
        for x in stack {
            out[x.seat.index()] += 6.0 * sq as f64 / 84.0;
        }
    }
    out
}

impl Heuristics for Manoever {
    /// Go home if possible, else win a gray piece, else (three times in four)
    /// land where the next move is multiplied most, farthest first.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let best = legal
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Move {
                        to: Dest::Barracks,
                        ..
                    }
                )
            })
            .or_else(|| {
                legal.iter().find(
                    |a| matches!(a, Action::Move { to: Dest::Square(sq), .. } if wins_gray(s, *sq)),
                )
            });
        if let Some(a) = best {
            return *a;
        }
        if rng.gen_bool(0.25) {
            return *legal.choose(rng).expect("a pending seat can act");
        }
        *legal
            .iter()
            .max_by_key(|a| match a {
                Action::Move {
                    to: Dest::Square(sq),
                    ..
                } => (lift(s, *sq), *sq),
                _ => (0, 0),
            })
            .expect("a pending seat can act")
    }

    /// Long simulations are scored on points and way come.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        if sim.moves < root.moves + HORIZON {
            return None;
        }
        let e = estimate(sim);
        let ranks = e
            .iter()
            .map(|&mine| e.iter().filter(|&&t| t > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks).rewards())
    }

    /// For each soldier: home, gray pieces, landings that lift it, and its
    /// nearest and farthest squares.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        let mut keep: Vec<Action> = Vec::new();
        let mut numbers: Vec<u8> = legal
            .iter()
            .filter_map(|a| match a {
                Action::Move { soldier, .. } => Some(*soldier),
                Action::Roll => None,
            })
            .collect();
        numbers.dedup();
        if numbers.is_empty() {
            return legal;
        }
        for n in numbers {
            let mine: Vec<&Action> = legal
                .iter()
                .filter(|a| matches!(a, Action::Move { soldier, .. } if *soldier == n))
                .collect();
            let squares: Vec<Square> = mine
                .iter()
                .filter_map(|a| match a {
                    Action::Move {
                        to: Dest::Square(sq),
                        ..
                    } => Some(*sq),
                    _ => None,
                })
                .collect();
            for a in &mine {
                let wanted = match a {
                    Action::Move {
                        to: Dest::Square(sq),
                        ..
                    } => {
                        wins_gray(s, *sq)
                            || lift(s, *sq) > 1
                            || squares.first() == Some(sq)
                            || squares.last() == Some(sq)
                    }
                    _ => true,
                };
                if wanted {
                    keep.push(**a);
                }
            }
        }
        keep
    }
}
