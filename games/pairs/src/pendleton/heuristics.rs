use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Pendleton, Phase, State};

impl State {
    /// What `seat` gains from `column` if the drawn card goes there.
    fn gain(&self, seat: Seat, column: usize) -> i64 {
        let Some(card) = self.drawn else {
            return 0;
        };
        let before = self.column_score(column);
        let mut after = self.clone();
        after.played[seat.index()][column].push(card);
        let after = after.column_score(column);
        let net = |c: &super::ColumnScore| {
            i64::from(c.scored[seat.index()]) - i64::from(c.scored[1 - seat.index()])
        };
        net(&after) - net(&before)
    }
}

impl Heuristics for Pendleton {
    /// The column where the card swings the most points; among equals, a
    /// random one.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let gain = |a: &Action| {
            let Action::Play { column } = *a;
            s.gain(seat, usize::from(column))
        };
        let best = legal.iter().map(gain).max().unwrap_or(0);
        let top: Vec<&Action> = legal.iter().filter(|a| gain(a) == best).collect();
        top.choose(rng)
            .map(|&a| a.clone())
            .unwrap_or(Action::Play { column: 0 })
    }

    /// Once the round the search started in is over, its points decide.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        if sim.round_no == root.round_no || sim.phase == Phase::Over {
            return None;
        }
        let gained: Vec<f64> = (0..2)
            .map(|i| f64::from(sim.scores[i]) - f64::from(root.scores[i]))
            .collect();
        let edge = (gained[0] - gained[1]) / 40.0;
        let p0 = (0.5 + edge).clamp(0.0, 1.0);
        Some(vec![p0, 1.0 - p0])
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::pendleton::testing::table;
    use crate::pendleton::{Action, Config, Pendleton};

    #[test]
    fn takes_the_column_the_card_wins() {
        // Seat 0 holds a 6: it overtakes the 5 on the 9, and ties nowhere.
        let s = table(
            &[1, 3, 5, 7, 9],
            [&[&[], &[], &[], &[], &[2]], &[&[], &[], &[], &[], &[5]]],
            6,
            0,
            &[],
        );
        let action = Pendleton::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(action, Action::Play { column: 4 });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for _ in 0..10 {
            let mut s = Pendleton::new(&Config { target: Some(40) }, 2, rng.gen());
            while let Some(&seat) = Pendleton::pending(&s).0.first() {
                let action = Pendleton::rollout_action(&s, seat, &mut rng);
                assert!(Pendleton::legal_actions(&s, seat).contains(&action));
                Pendleton::apply(&mut s, seat, &action).unwrap();
            }
        }
    }
}
