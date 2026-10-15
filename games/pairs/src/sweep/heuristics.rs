use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Phase, State, Sweep};

impl Heuristics for Sweep {
    /// From what `seat` sees only. Shoot the highest card when it tops
    /// every target (it sweeps them up), else the lowest (to be the lowest
    /// shooter, who takes the highest targets). Capture the highest card.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let hand = &s.hands[seat.index()];
        let chosen = match s.phase {
            Phase::Play => {
                let top = s.targets.iter().copied().max();
                let high = hand.iter().copied().max();
                let low = hand.iter().copied().min();
                match (high, top) {
                    (Some(h), Some(t)) if h > t => Some(Action::Shoot { card: h }),
                    _ => low.map(|card| Action::Shoot { card }),
                }
            }
            Phase::Capture => legal
                .iter()
                .max_by_key(|a| match a {
                    Action::Capture { card } => card.value(),
                    Action::Shoot { .. } => 0,
                })
                .cloned(),
            Phase::Over => None,
        };
        chosen
            .filter(|a| legal.contains(a))
            .or_else(|| legal.choose(rng).cloned())
            .expect("a pending seat has a legal action")
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::sweep::{Config, Sweep};

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 3..=6u8 {
            for _ in 0..10 {
                let mut s = Sweep::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = Sweep::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = Sweep::rollout_action(&s, seat, &mut rng);
                    assert!(Sweep::legal_actions(&s, seat).contains(&action));
                    Sweep::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
