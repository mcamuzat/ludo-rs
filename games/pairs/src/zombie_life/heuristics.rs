use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Phase, State, ZombieLife, Zone};

impl Heuristics for ZombieLife {
    /// A free card from the horde first. Otherwise draw while the cards a
    /// pass would bank are worth less than the chance of one more; with
    /// nothing to bank, always draw. A drawn card goes home when it can.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        match s.phase {
            Phase::Placing => {
                let home = Action::Place { zone: Zone::Refuge };
                if legal.contains(&home) {
                    return home;
                }
                return Action::Place { zone: Zone::Horde };
            }
            Phase::Play => {}
            Phase::Dealing | Phase::Over => {
                return legal.choose(rng).cloned().unwrap_or(Action::Pass);
            }
        }
        // A card of the horde costs nothing: it leaves the horde, so its
        // rank cannot overrun its new owner until another one comes. The
        // rank with the fewest copies left is the least likely to.
        let pool = s.draw_pool();
        let take = legal
            .iter()
            .filter_map(|a| match a {
                Action::Take { card } => Some(*card),
                _ => None,
            })
            .min_by_key(|card| pool[card.index()]);
        if let Some(card) = take {
            return Action::Take { card };
        }
        if !legal.contains(&Action::Draw) {
            return Action::Pass;
        }
        // What a pass banks now: the refuge less a card per other player in.
        let others = s.active().len().saturating_sub(1);
        let banked = s.refuges[seat.index()].len().saturating_sub(others);
        if banked == 0 || s.overrun_chance(seat) * (banked as f64 + 1.0) < 1.0 {
            Action::Draw
        } else {
            Action::Pass
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::testing::r;
    use crate::zombie_life::testing::table;
    use crate::zombie_life::{Action, Config, State, ZombieLife, Zone};

    fn rollout(s: &State, seat: u8, seed: u64) -> Action {
        ZombieLife::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(seed))
    }

    /// Every card not on the table, as the draw pile.
    fn rest(taken: &[u8]) -> Vec<u8> {
        let mut deck: Vec<u8> = (1..=10u8).flat_map(|v| vec![v; usize::from(v)]).collect();
        for card in taken {
            let at = deck.iter().position(|c| c == card).expect("card left");
            deck.remove(at);
        }
        deck
    }

    #[test]
    fn the_chance_of_an_overrun_counts_the_cards_nobody_can_place() {
        // Three 5s left among 51 cards; only a 5 overruns seat 0.
        let s = table(&[&[5], &[6], &[7]], &[5], &rest(&[5, 6, 7, 5]));
        assert!((s.overrun_chance(Seat(0)) - 3.0 / 51.0).abs() < 1e-9);
        assert_eq!(s.overrun_chance(Seat(1)), 0.0);
    }

    #[test]
    fn never_passes_when_passing_scores_nothing() {
        // One card and two players still in: a pass would hide it.
        let s = table(&[&[10], &[6], &[7]], &[10], &rest(&[10, 6, 7, 10]));
        assert!(s.overrun_chance(Seat(0)) > 0.1);
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Draw);
        }
    }

    #[test]
    fn passes_when_the_risk_is_high_and_points_are_banked() {
        let mine = [5, 6, 7, 8, 9, 10];
        let s = table(
            &[&mine, &[1], &[2]],
            &mine,
            &rest(&[5, 6, 7, 8, 9, 10, 5, 6, 7, 8, 9, 10, 1, 2]),
        );
        assert!(s.overrun_chance(Seat(0)) > 0.7);
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Pass);
        }
    }

    #[test]
    fn draws_when_nothing_can_overrun() {
        let s = table(&[&[5, 6, 7], &[1], &[2]], &[], &rest(&[5, 6, 7, 1, 2]));
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Draw);
        }
    }

    #[test]
    fn takes_the_horde_card_with_the_fewest_copies_left() {
        let s = table(&[&[5], &[6], &[7]], &[9, 8], &rest(&[5, 6, 7, 9, 8]));
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Take { card: r(8) });
        }
    }

    #[test]
    fn a_drawn_card_goes_home_when_it_can() {
        let mut s = table(&[&[5], &[6], &[7]], &[], &[9, 5]);
        ZombieLife::apply(&mut s, Seat(0), &Action::Draw).unwrap();
        assert_eq!(rollout(&s, 0, 1), Action::Place { zone: Zone::Refuge });
        ZombieLife::apply(&mut s, Seat(0), &Action::Pass).unwrap_err();
        ZombieLife::apply(&mut s, Seat(0), &Action::Place { zone: Zone::Horde }).unwrap();
        ZombieLife::apply(&mut s, Seat(1), &Action::Pass).unwrap();
        ZombieLife::apply(&mut s, Seat(2), &Action::Pass).unwrap();
        // Seat 0 draws a second 5: only the horde takes it.
        ZombieLife::apply(&mut s, Seat(0), &Action::Draw).unwrap();
        assert_eq!(rollout(&s, 0, 1), Action::Place { zone: Zone::Horde });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 3..=6u8 {
            for _ in 0..10 {
                let mut s = ZombieLife::new(&Config::default(), players, rng.gen());
                let mut steps = 0;
                while let Some(&seat) = ZombieLife::pending(&s).0.first() {
                    let action = ZombieLife::rollout_action(&s, seat, &mut rng);
                    assert!(
                        ZombieLife::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    ZombieLife::apply(&mut s, seat, &action).unwrap();
                    steps += 1;
                    assert!(steps < 50_000, "the rollout never ends");
                }
            }
        }
    }
}
