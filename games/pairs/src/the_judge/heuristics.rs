use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{recipe_points, Action, Phase, State, TheJudge, JUDGE_SIZE};
use crate::deck::Rank;

/// How often the rollout plays a random legal action instead.
const WANDER: f64 = 0.1;

impl State {
    /// Per rank, the copies `seat` cannot place: not in a recipe nor its
    /// own hand.
    fn unseen_for(&self, seat: Seat) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        for card in self
            .recipes
            .iter()
            .flatten()
            .chain(&self.hands[seat.index()])
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// The chance, from what `seat` sees, that the Judge holds none of
    /// `ranks` (each unseen card equally likely to be among its eight).
    /// For the AI: never in a view.
    pub fn spared_chance(&self, seat: Seat, ranks: &[Rank]) -> f64 {
        let unseen = self.unseen_for(seat);
        let total: u32 = unseen.iter().map(|&c| u32::from(c)).sum();
        let bad: u32 = ranks.iter().map(|r| u32::from(unseen[r.index()])).sum();
        (0..JUDGE_SIZE as u32).fold(1.0, |p, k| {
            if total < bad + k + 1 {
                0.0
            } else {
                p * f64::from(total - bad - k) / f64::from(total - k)
            }
        })
    }

    /// The recipe's points weighed by its chance to be spared.
    fn expected(&self, seat: Seat, recipe: &[Rank]) -> f64 {
        f64::from(recipe_points(recipe)) * self.spared_chance(seat, recipe)
    }
}

impl Heuristics for TheJudge {
    /// From what `seat` sees only: add the card that raises the recipe's
    /// expected score most, or pass when none raises it.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Cook || rng.gen_bool(WANDER) {
            return legal.choose(rng).cloned().unwrap_or(Action::Pass);
        }
        let recipe = &s.recipes[seat.index()];
        let now = s.expected(seat, recipe);
        let best = s
            .addable(seat)
            .into_iter()
            .map(|card| {
                let mut with = recipe.clone();
                with.push(card);
                (s.expected(seat, &with), card)
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        match best {
            Some((value, card)) if value > now => Action::Add { card },
            _ => Action::Pass,
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::testing::{r, ranks};
    use crate::the_judge::testing::table;
    use crate::the_judge::{Action, Config, TheJudge};

    #[test]
    fn a_rank_with_every_copy_in_sight_is_spared() {
        // Seat 0 holds the only 1.
        let s = table(&[&[1, 5], &[2, 3]], &[&[], &[]]);
        assert_eq!(s.spared_chance(Seat(0), &ranks(&[1])), 1.0);
        assert!(s.spared_chance(Seat(0), &ranks(&[10])) < 0.3);
    }

    #[test]
    fn adds_a_safe_card_and_stops_on_a_risky_long_recipe() {
        let s = table(&[&[1, 9], &[2, 3]], &[&[], &[]]);
        let mut rng = SmallRng::seed_from_u64(3);
        let mut adds = 0;
        for _ in 0..50 {
            if TheJudge::rollout_action(&s, Seat(0), &mut rng) == (Action::Add { card: r(9) }) {
                adds += 1;
            }
        }
        // The 9 is worth more than the safe 1, despite its risk.
        assert!(adds > 35, "{adds}");
        let s = table(&[&[8], &[2]], &[&[10, 9, 7, 6, 5, 4], &[]]);
        assert!(s.spared_chance(Seat(0), &s.recipes[0]) < 0.1);
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=6u8 {
            for _ in 0..5 {
                let mut s = TheJudge::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = TheJudge::pending(&s).0.first() {
                    let action = TheJudge::rollout_action(&s, seat, &mut rng);
                    assert!(
                        TheJudge::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    TheJudge::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
