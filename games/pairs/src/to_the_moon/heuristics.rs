use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use super::{Action, State, ToTheMoon};
use crate::deck::has_rank;

/// Below this chance of a match, a test beats spending a hand card.
const SAFE_TEST: f64 = 0.3;
/// Above it, withdrawing beats a test.
const RISKY_TEST: f64 = 0.6;

impl State {
    /// The chance that the top card matches the countdown, from what `seat`
    /// sees. For the AI: never in a view.
    pub fn test_risk(&self, seat: Seat) -> f64 {
        let pool = self.unlocated_for(Some(seat));
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 1.0;
        }
        let matching: u32 = self
            .trick
            .iter()
            .map(|card| u32::from(pool[card.index()]))
            .sum();
        f64::from(matching) / f64::from(total)
    }
}

impl Heuristics for ToTheMoon {
    /// From what `seat` sees only. Test while it is safe; else build the
    /// highest rank (the most common, the hardest to place later); else
    /// test unless it is too risky; else withdraw with the lowest card.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, _rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let can_test = legal.contains(&Action::Test);
        let risk = if can_test { s.test_risk(seat) } else { 1.0 };
        if risk < SAFE_TEST {
            return Action::Test;
        }
        let build = legal
            .iter()
            .filter_map(|a| match a {
                Action::Build { card } => Some(*card),
                _ => None,
            })
            .max();
        if let Some(card) = build {
            return Action::Build { card };
        }
        if risk < RISKY_TEST {
            return Action::Test;
        }
        let take = s
            .trick
            .iter()
            .copied()
            .filter(|_| s.trick.len() >= 2)
            .min_by_key(|&card| (has_rank(&s.hands[seat.index()], card), card));
        Action::Withdraw { take }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::testing::r;
    use crate::to_the_moon::testing::table;
    use crate::to_the_moon::{Action, Config, State, ToTheMoon};

    fn rollout(s: &State, seat: u8) -> Action {
        ToTheMoon::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(1))
    }

    #[test]
    fn the_risk_counts_the_cards_the_seat_cannot_place() {
        // Seat 1 cannot place 55 - 2 (trick) - 2 (hand) = 51 cards: nine 10s
        // and two 3s match.
        let s = table(&[&[], &[1, 2]], &[10, 3], &[4]);
        assert!((s.test_risk(Seat(1)) - 11.0 / 51.0).abs() < 1e-9);
    }

    #[test]
    fn tests_a_small_countdown() {
        let s = table(&[&[], &[1, 2]], &[3], &[4]);
        assert_eq!(rollout(&s, 1), Action::Test);
    }

    #[test]
    fn builds_the_highest_rank_when_testing_is_risky() {
        let s = table(&[&[], &[1, 2, 9]], &[10, 8, 7, 6], &[4]);
        assert_eq!(rollout(&s, 1), Action::Build { card: r(9) });
    }

    #[test]
    fn withdraws_with_a_rank_it_lacks_when_nothing_is_safe() {
        let s = table(&[&[], &[10]], &[10, 9, 8, 7, 6, 5], &[4]);
        assert_eq!(rollout(&s, 1), Action::Withdraw { take: Some(r(5)) });
        let alone = table(&[&[], &[10]], &[10], &[]);
        assert_eq!(rollout(&alone, 1), Action::Withdraw { take: None });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=6u8 {
            for _ in 0..5 {
                let mut s = ToTheMoon::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = ToTheMoon::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = ToTheMoon::rollout_action(&s, seat, &mut rng);
                    assert!(
                        ToTheMoon::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    ToTheMoon::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
