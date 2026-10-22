use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Phase, Pickup, State};
use crate::deck::Rank;

/// How often the rollout plays a random legal action instead.
const WANDER: f64 = 0.1;
/// What spending a card from hand costs, per point of its rank.
const SPEND: f64 = 0.6;

impl State {
    /// Taking `rank`: its value, plus the value it knocks out of the other
    /// piles.
    fn take_value(&self, seat: Seat, rank: Rank) -> f64 {
        let knocked: u32 = self
            .piles
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != seat.index())
            .flat_map(|(_, p)| p.iter().filter(|&&c| c == rank))
            .map(|c| u32::from(c.value()))
            .sum();
        f64::from(rank.value()) + f64::from(knocked)
    }

    /// Playing `card`, as if it acted first: the best take, or the card
    /// itself when it can only be stacked; less what spending it costs.
    fn choose_value(&self, seat: Seat, card: Rank) -> f64 {
        let gain = self
            .takeable(card)
            .into_iter()
            .map(|rank| self.take_value(seat, rank))
            .max_by(f64::total_cmp)
            .unwrap_or(f64::from(card.value()));
        gain - SPEND * f64::from(card.value())
    }
}

impl Heuristics for Pickup {
    /// Choose the card whose best take is worth most; take the rank worth
    /// most, knockouts included.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if rng.gen_bool(WANDER) {
            return legal.choose(rng).cloned().expect("a legal action");
        }
        let value = |a: &Action| match (s.phase, a) {
            (Phase::Take, Action::Take { card }) => s.take_value(seat, *card),
            (_, Action::Choose { card }) => s.choose_value(seat, *card),
            _ => f64::NEG_INFINITY,
        };
        legal
            .iter()
            .max_by(|a, b| value(a).total_cmp(&value(b)))
            .cloned()
            .expect("a legal action")
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::pickup::testing::table;
    use crate::pickup::{Action, Config, Length, Phase, Pickup, State};
    use crate::testing::{r, ranks};

    fn usual(s: &State, seat: u8) -> Action {
        let mut rng = SmallRng::seed_from_u64(9);
        let mut counts: Vec<(Action, u32)> = Vec::new();
        for _ in 0..50 {
            let a = Pickup::rollout_action(s, Seat(seat), &mut rng);
            match counts.iter_mut().find(|(b, _)| *b == a) {
                Some((_, n)) => *n += 1,
                None => counts.push((a, 1)),
            }
        }
        counts.into_iter().max_by_key(|(_, n)| *n).unwrap().0
    }

    #[test]
    fn chooses_a_card_that_takes_something_good() {
        let s = table(&[&[9, 2], &[1, 1]], &[8, 1], &[]);
        assert_eq!(usual(&s, 0), Action::Choose { card: r(9) });
    }

    #[test]
    fn takes_the_rank_that_knocks_out_most() {
        let mut s = table(&[&[9, 1], &[1, 1]], &[9, 5], &[]);
        s.piles[1] = ranks(&[5, 5, 5]);
        Pickup::apply(&mut s, Seat(0), &Action::Choose { card: r(9) }).unwrap();
        Pickup::apply(&mut s, Seat(1), &Action::Choose { card: r(1) }).unwrap();
        assert_eq!(s.phase, Phase::Take);
        assert_eq!(usual(&s, 0), Action::Take { card: r(5) });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for length in [Length::OneDeal, Length::Target] {
            for players in 2..=6u8 {
                let mut s = Pickup::new(&Config { length }, players, rng.gen());
                while let Some(&seat) = Pickup::pending(&s).0.first() {
                    let action = Pickup::rollout_action(&s, seat, &mut rng);
                    assert!(
                        Pickup::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    Pickup::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
