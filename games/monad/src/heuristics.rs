use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::rules::{leap_rank, trade_target};
use crate::{Action, Monad, Rank, State, Target};

/// Simulated moves after which a simulation is scored as it stands.
const HORIZON: u32 = 120;

/// What a transaction brings, by value; `None` for the other moves.
fn gain(s: &State, seat: Seat, action: &Action) -> Option<u32> {
    match action {
        Action::Trade { cards } => trade_target(s, seat, *cards).map(|(t, _)| t.value()),
        Action::Buy { target, .. } => Some(target.value()),
        Action::Leap { colors } => leap_rank(colors.len()).map(Rank::value),
        _ => None,
    }
}

/// Each seat's progress: its Monads, then the points in its hand.
fn progress(s: &State) -> Vec<u32> {
    s.hands
        .iter()
        .zip(&s.monads)
        .map(|(hand, &m)| u32::from(m) * 80 + hand.iter().map(|c| c.rank.value()).sum::<u32>())
        .collect()
}

impl Heuristics for Monad {
    /// Take every bonus; else the transaction that brings the most, with a
    /// random move one time in four; else end the turn, draw, flip or pass.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if legal.contains(&Action::TakeBonus) {
            return Action::TakeBonus;
        }
        if rng.gen_bool(0.25) {
            return legal
                .choose(rng)
                .expect("a pending seat has a move")
                .clone();
        }
        legal
            .iter()
            .filter_map(|a| gain(s, seat, a).map(|g| (g, a)))
            .max_by_key(|&(g, _)| g)
            .map(|(_, a)| a)
            .or_else(|| legal.iter().find(|a| gain(s, seat, a).is_none()))
            .expect("a pending seat has a move")
            .clone()
    }

    /// Long simulations are ranked on progress as it stands.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.moves >= root.moves + HORIZON).then(|| {
            let p = progress(sim);
            let ranks = p
                .iter()
                .map(|&mine| {
                    u8::try_from(p.iter().filter(|&&x| x > mine).count()).expect("4 seats")
                })
                .collect();
            Outcome::Ranked(ranks).rewards()
        })
    }

    /// Leaves out wild cards spent on a lower card, and keeps for each
    /// target the three buys that overpay least.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        let mut buys: Vec<(Target, u32, Rank, &Action)> = Vec::new();
        let mut kept: Vec<Action> = Vec::new();
        for action in &legal {
            match action {
                Action::Trade { cards: [a, b] } if a.rank != b.rank => {
                    let identity = s.identities[seat.index()];
                    let (wild, other) = if a.color == identity { (a, b) } else { (b, a) };
                    if wild.rank < other.rank {
                        kept.push(action.clone());
                    }
                }
                Action::Buy { target, cards } => {
                    let total: u32 = cards.iter().map(|c| c.rank.value()).sum();
                    let top = cards
                        .iter()
                        .map(|c| c.rank)
                        .max()
                        .expect("two cards or more");
                    buys.push((*target, total - target.value(), top, action));
                }
                _ => kept.push(action.clone()),
            }
        }
        buys.sort_by_key(|&(t, over, top, _)| (t.value(), over, top));
        let mut per_target: Vec<(Target, usize)> = Vec::new();
        for (target, _, _, action) in buys {
            let n = match per_target.iter_mut().find(|(t, _)| *t == target) {
                Some((_, n)) => n,
                None => {
                    per_target.push((target, 0));
                    &mut per_target.last_mut().expect("just pushed").1
                }
            };
            if *n < 3 {
                *n += 1;
                kept.push(action.clone());
            }
        }
        if kept.is_empty() {
            legal
        } else {
            kept
        }
    }
}
