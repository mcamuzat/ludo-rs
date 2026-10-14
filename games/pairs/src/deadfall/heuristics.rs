use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Deadfall, Phase, State};
use crate::deck::{has_rank, lowest, Rank};

/// A call this likely to be right is worth making mid-hand.
const SURE_CALL: f64 = 0.7;

impl State {
    /// Per rank, the cards `seat` cannot place: not in a stack nor its hand.
    fn unseen_for(&self, seat: Seat) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        for card in self
            .stacks
            .iter()
            .flatten()
            .chain(&self.hands[seat.index()])
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// The chance, from what `seat` sees, that no hand holds `rank`: every
    /// unseen copy is among the undealt cards. For the AI: never in a view.
    pub fn dead_chance(&self, seat: Seat, rank: Rank) -> f64 {
        if has_rank(&self.hands[seat.index()], rank) {
            return 0.0;
        }
        let unseen = self.unseen_for(seat);
        let total: u32 = unseen.iter().map(|&c| u32::from(c)).sum();
        let held: u32 = self
            .hands
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != seat.index())
            .map(|(_, h)| h.len() as u32)
            .sum();
        let undealt = total.saturating_sub(held);
        (0..u32::from(unseen[rank.index()])).fold(1.0, |p, k| {
            if k >= undealt {
                0.0
            } else {
                p * f64::from(undealt - k) / f64::from(total - k)
            }
        })
    }
}

impl Heuristics for Deadfall {
    /// From what `seat` sees only. Call the likeliest dead top card when it
    /// is nearly sure or there is nothing left to play; otherwise play the
    /// card most likely to stay alive (one of a pair, then the rank with
    /// the most unseen copies, the highest first).
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Play {
            return legal.choose(rng).cloned().unwrap_or(Action::Call { seat });
        }
        let best_call = legal
            .iter()
            .filter_map(|a| match *a {
                Action::Call { seat: target } => {
                    let rank = *s.stacks[target.index()].last()?;
                    Some((s.dead_chance(seat, rank), rank, target))
                }
                _ => None,
            })
            .max_by(|a, b| {
                (a.0, a.1)
                    .partial_cmp(&(b.0, b.1))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        let hand = &s.hands[seat.index()];
        if let Some((chance, _, target)) = best_call {
            if hand.is_empty() || chance >= SURE_CALL {
                return Action::Call { seat: target };
            }
        }
        let unseen = s.unseen_for(seat);
        let safety = |card: Rank| {
            let pair = hand.iter().filter(|&&c| c == card).count() >= 2;
            (pair, unseen[card.index()], card)
        };
        hand.iter()
            .copied()
            .max_by_key(|&card| safety(card))
            .map_or(Action::Call { seat }, |card| Action::Play { card })
    }

    /// The door card is not worth a search (and a table of AIs would
    /// think one after the other): the lowest, hard to get rid of later.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Door || legal.is_empty() {
            return legal;
        }
        match lowest(&s.hands[seat.index()]) {
            Some(card) => vec![Action::Door { card }],
            None => legal,
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::deadfall::testing::table;
    use crate::deadfall::{Action, Config, Deadfall, State};
    use crate::testing::r;

    fn rollout(s: &State, seat: u8) -> Action {
        Deadfall::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(1))
    }

    #[test]
    fn a_rank_in_my_hand_is_never_dead() {
        let s = table(&[&[5], &[1]], &[&[2], &[5]], &[]);
        assert_eq!(s.dead_chance(Seat(0), r(5)), 0.0);
    }

    #[test]
    fn a_rank_with_every_copy_in_sight_is_dead() {
        let s = table(&[&[5], &[3]], &[&[2, 1], &[2]], &[9]);
        assert_eq!(s.dead_chance(Seat(0), r(2)), 1.0);
        assert_eq!(rollout(&s, 0), Action::Call { seat: Seat(1) });
    }

    #[test]
    fn the_chance_counts_the_undealt_cards() {
        // Seat 0 cannot place 55 - 3 = 52 cards, one of them in seat 1's
        // hand: the last 2 is dead with chance 51/52.
        let s = table(&[&[5], &[3]], &[&[2], &[9]], &[]);
        assert!((s.dead_chance(Seat(0), r(2)) - 51.0 / 52.0).abs() < 1e-9);
    }

    #[test]
    fn plays_one_of_a_pair_rather_than_call_a_live_rank() {
        let s = table(&[&[4, 7, 7], &[1, 2, 3]], &[&[6], &[10]], &[]);
        assert_eq!(rollout(&s, 0), Action::Play { card: r(7) });
    }

    #[test]
    fn calls_with_an_empty_hand() {
        let s = table(&[&[], &[1, 2, 3]], &[&[6], &[10]], &[]);
        assert_eq!(rollout(&s, 0), Action::Call { seat: Seat(1) });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=7u8 {
            for _ in 0..5 {
                let mut s = Deadfall::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = Deadfall::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = Deadfall::rollout_action(&s, seat, &mut rng);
                    assert!(
                        Deadfall::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    Deadfall::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
