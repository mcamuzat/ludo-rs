use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{capture, round_points, Action, Phase, State, TheCave};
use crate::deck::Rank;

/// How often the rollout plays a random legal action instead.
const WANDER: f64 = 0.1;
/// How much the cards a play leaves for the next player weigh.
const EXPOSURE: f64 = 0.5;

impl State {
    /// What playing `card` is worth to `seat` from what it sees: the change
    /// in its majority points, less what the Cave it leaves offers next.
    fn play_value(&self, seat: Seat, card: Rank) -> f64 {
        let i = seat.index();
        let before = round_points(&self.captured)[i];
        let taken = capture(&self.cave, card);
        let mut captured = self.captured.clone();
        captured[i].extend(&taken);
        let after = round_points(&captured)[i];
        let mut cave = self.cave.clone();
        if let Some(&rank) = taken.first() {
            cave.retain(|&c| c != rank);
        }
        cave.push(card);
        let offered = (1..=10)
            .filter_map(Rank::new)
            .map(|r| capture(&cave, r).len())
            .max()
            .unwrap_or(0);
        f64::from(after) - f64::from(before) - EXPOSURE * offered as f64
    }
}

impl Heuristics for TheCave {
    /// Play the card that wins the most majority points, leaving a poor
    /// Cave behind.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Play || rng.gen_bool(WANDER) {
            return legal.choose(rng).cloned().expect("a legal play");
        }
        s.playable(seat)
            .into_iter()
            .map(|card| (s.play_value(seat, card), card))
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, card)| Action::Play { card })
            .expect("a card in hand")
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::testing::{r, ranks};
    use crate::the_cave::testing::table;
    use crate::the_cave::{Action, Config, TheCave};

    #[test]
    fn prefers_a_capture_that_wins_a_majority() {
        let mut s = table(&[&[8, 4], &[1, 1]], &[7, 7, 3, 9, 10], &[]);
        s.captured[1] = ranks(&[7]);
        let mut rng = SmallRng::seed_from_u64(5);
        let eights = (0..50)
            .filter(|_| {
                TheCave::rollout_action(&s, Seat(0), &mut rng) == (Action::Play { card: r(8) })
            })
            .count();
        assert!(eights > 35, "{eights}");
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=6u8 {
            for _ in 0..5 {
                let mut s = TheCave::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = TheCave::pending(&s).0.first() {
                    let action = TheCave::rollout_action(&s, seat, &mut rng);
                    assert!(TheCave::legal_actions(&s, seat).contains(&action));
                    TheCave::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
