use ludo_core::{Heuristics, Seat};
use rand::Rng;

use super::{strength, Action, Monster, State};

impl Heuristics for Monster {
    /// Discard as little as possible, keeping the highest hand (fewer
    /// cards for the Monster); fold when no discard can clear the pairs.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, _rng: &mut R) -> Action {
        let hand = &s.hands[seat.index()];
        State::discard_options(hand)
            .into_iter()
            .max_by_key(|out| {
                let mut kept = hand.clone();
                for card in out {
                    if let Some(at) = kept.iter().position(|c| c == card) {
                        kept.remove(at);
                    }
                }
                (std::cmp::Reverse(out.len()), strength(&kept))
            })
            .map_or(Action::Fold, |cards| Action::Discard { cards })
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::monster::testing::table;
    use crate::monster::{Action, Config, Monster};
    use crate::testing::ranks;

    #[test]
    fn clears_the_pairs_with_the_lowest_cards() {
        let s = table(
            &[
                &[1, 2, 3, 4, 5, 6],
                &[9, 9, 4, 4, 7, 10],
                &[1, 2, 3, 4, 5, 6],
                &[1, 2, 3, 4, 5, 6],
            ],
            3,
        );
        let mut rng = SmallRng::seed_from_u64(1);
        assert_eq!(
            Monster::rollout_action(&s, Seat(1), &mut rng),
            Action::Discard {
                cards: ranks(&[4, 9])
            }
        );
        let s = table(&[&[1], &[9, 9, 8, 8, 7, 7], &[1], &[1]], 3);
        // Three pairs need three discards: still possible.
        assert_eq!(
            Monster::rollout_action(&s, Seat(1), &mut rng),
            Action::Discard {
                cards: ranks(&[7, 8, 9])
            }
        );
        let s = table(&[&[1], &[9, 9, 9, 9, 8, 8], &[1], &[1]], 3);
        assert_eq!(Monster::rollout_action(&s, Seat(1), &mut rng), Action::Fold);
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 4..=8u8 {
            for _ in 0..5 {
                let mut s = Monster::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = Monster::pending(&s).0.first() {
                    let action = Monster::rollout_action(&s, seat, &mut rng);
                    assert!(Monster::legal_actions(&s, seat).contains(&action));
                    Monster::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
