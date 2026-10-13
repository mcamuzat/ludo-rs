use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Auberge, Phase, State};
use crate::deck::pair_in;

/// Accuse above this estimated chance of a pair.
const ACCUSE: f64 = 0.5;
/// Dare only below this chance of pairing the target's face-up cards.
const DARE: f64 = 0.2;
/// How often a hand card is played on someone rather than kept.
const ATTACK: f64 = 0.6;

/// The opponent `seat` thinks most likely to hold a pair, among those it
/// may accuse.
fn prime_suspect(s: &State, seat: Seat, legal: &[Action]) -> Option<(Seat, f64)> {
    legal
        .iter()
        .filter_map(|action| match action {
            Action::Accuse { target } => Some((*target, s.pair_chance(Some(seat), *target))),
            _ => None,
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

impl Heuristics for Auberge {
    /// From what `seat` sees only. Setup: place a card of its own pair.
    /// Play: accuse a likely pair; else break its own pair by attacking with
    /// it; else mostly attack to empty its hand (no hand, no accusation);
    /// else dare a safe target, or draw.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let any = |rng: &mut R| legal.choose(rng).cloned().unwrap_or(Action::Draw);
        if s.phase == Phase::Setup {
            return match pair_in(&s.hands[seat.index()]) {
                Some(card) => Action::Place { card },
                None => any(rng),
            };
        }
        let suspect = prime_suspect(s, seat, &legal);
        if let Some((target, chance)) = suspect {
            if chance >= ACCUSE {
                return Action::Accuse { target };
            }
        }
        let attacks: Vec<&Action> = legal
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        if let Some(pair) = s.own_pair(seat) {
            let with_pair: Vec<&&Action> = attacks
                .iter()
                .filter(|a| matches!(a, Action::Attack { card, .. } if *card == pair))
                .collect();
            if let Some(&&attack) = with_pair.choose(rng) {
                return attack.clone();
            }
        }
        if !attacks.is_empty() && rng.gen_bool(ATTACK) {
            if let Some(&attack) = attacks.choose(rng) {
                return attack.clone();
            }
        }
        let safest = legal
            .iter()
            .filter_map(|action| match action {
                Action::Dare { target } => Some((*target, s.dare_risk(Some(seat), *target))),
                _ => None,
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((target, risk)) = safest {
            if risk <= DARE && (risk <= s.draw_risk(seat) || rng.gen_bool(0.5)) {
                return Action::Dare { target };
            }
        }
        if legal.contains(&Action::Draw) {
            return Action::Draw;
        }
        match suspect {
            Some((target, _)) if attacks.is_empty() => Action::Accuse { target },
            _ => any(rng),
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::auberge::testing::{setup, table};
    use crate::auberge::{Action, Auberge, Config, State};
    use crate::testing::r;

    fn rollout(s: &State, seat: u8, seed: u64) -> Action {
        Auberge::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(seed))
    }

    #[test]
    fn accuses_a_player_who_very_likely_holds_a_pair() {
        // Eight hidden cards behind three face-up ones.
        let s = table(
            &[&[5], &[1, 2, 2, 3, 3, 4, 4, 6]],
            &[&[7], &[10, 9, 8]],
            &[5, 5, 6],
        );
        assert!(s.pair_chance(Some(Seat(0)), Seat(1)) > 0.9);
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Accuse { target: Seat(1) });
        }
    }

    #[test]
    fn gets_rid_of_its_own_pair_by_attacking_with_it() {
        let s = table(&[&[7, 7, 3], &[4]], &[&[2], &[9]], &[5, 5, 6, 8, 8, 10]);
        for seed in 0..30 {
            assert_eq!(
                rollout(&s, 0, seed),
                Action::Attack {
                    target: Seat(1),
                    card: r(7)
                }
            );
        }
    }

    #[test]
    fn does_not_accuse_on_a_slim_chance() {
        let s = table(&[&[], &[4]], &[&[2], &[9]], &[5, 5, 6, 8, 8, 10]);
        for seed in 0..30 {
            assert_ne!(rollout(&s, 0, seed), Action::Accuse { target: Seat(1) });
        }
    }

    #[test]
    fn places_a_card_of_its_pair_during_setup() {
        let s = setup(&[&[9, 4, 9], &[2, 8, 3]], &[1]);
        for seed in 0..20 {
            assert_eq!(rollout(&s, 0, seed), Action::Place { card: r(9) });
        }
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(21);
        for players in 2..=8u8 {
            for _ in 0..5 {
                let mut s = Auberge::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = Auberge::pending(&s).0.first() {
                    let action = Auberge::rollout_action(&s, seat, &mut rng);
                    assert!(
                        Auberge::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    Auberge::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
