use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{resolve, Action, Fate, MarketDay, Phase, State};
use crate::deck::Rank;

/// How often the rollout plays a random legal action instead.
const WANDER: f64 = 0.1;
/// How much the others' points in the round count against one's own.
const SPITE: f64 = 0.5;

/// `seat`'s points in `trick` as it stands, less part of the others'.
fn standing(trick: &[(Seat, Rank)], seat: Seat) -> f64 {
    let cards: Vec<Rank> = trick.iter().map(|&(_, c)| c).collect();
    let (mut mine, mut theirs) = (0u32, 0u32);
    for (&(owner, card), fate) in trick.iter().zip(resolve(&cards)) {
        if fate == Fate::Scored {
            if owner == seat {
                mine += u32::from(card.value());
            } else {
                theirs += u32::from(card.value());
            }
        }
    }
    f64::from(mine) - SPITE * f64::from(theirs)
}

impl Heuristics for MarketDay {
    /// Play the card that leaves the round best for `seat` as it stands
    /// (later duplicates, a lone Rain, Knights); the Tinker trades for the
    /// card that helps most.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if rng.gen_bool(WANDER) {
            return legal.choose(rng).cloned().expect("a legal action");
        }
        let value = |a: &Action| -> f64 {
            let mut trick = s.trick.clone();
            match *a {
                Action::Play { card } => trick.push((seat, card)),
                Action::Swap { with } => {
                    let t = s.tinker().expect("a Tinker");
                    let (x, y) = (trick[t].0, trick[with].0);
                    trick[t].0 = y;
                    trick[with].0 = x;
                }
                Action::Keep => {}
            }
            standing(&trick, seat)
        };
        debug_assert!(s.phase != Phase::Over);
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

    use crate::market_day::testing::table;
    use crate::market_day::{Action, Config, MarketDay, Phase, State};
    use crate::testing::r;

    fn usual(s: &State, seat: u8) -> Action {
        let mut rng = SmallRng::seed_from_u64(9);
        let mut counts: Vec<(Action, u32)> = Vec::new();
        for _ in 0..50 {
            let a = MarketDay::rollout_action(s, Seat(seat), &mut rng);
            match counts.iter_mut().find(|(b, _)| *b == a) {
                Some((_, n)) => *n += 1,
                None => counts.push((a, 1)),
            }
        }
        counts.into_iter().max_by_key(|(_, n)| *n).unwrap().0
    }

    #[test]
    fn spoils_an_earlier_card_with_a_later_duplicate() {
        let mut s = table(&[&[4], &[5], &[8, 4], &[6]]);
        s.leader = Seat(1);
        s.trick = vec![(Seat(1), r(8))];
        s.hands[1].clear();
        s.turn = Seat(2);
        assert_eq!(usual(&s, 2), Action::Play { card: r(8) });
    }

    #[test]
    fn the_tinker_trades_for_the_best_card() {
        let mut s = table(&[&[1, 4], &[8, 4], &[3, 4], &[5, 4]]);
        for (seat, card) in [(0, 1), (1, 8), (2, 3), (3, 5)] {
            MarketDay::apply(&mut s, Seat(seat), &Action::Play { card: r(card) }).unwrap();
        }
        assert_eq!(s.phase, Phase::Tinker);
        assert_eq!(usual(&s, 0), Action::Swap { with: 1 });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=8u8 {
            let mut s = MarketDay::new(&Config::default(), players, rng.gen());
            while let Some(&seat) = MarketDay::pending(&s).0.first() {
                let action = MarketDay::rollout_action(&s, seat, &mut rng);
                assert!(
                    MarketDay::legal_actions(&s, seat).contains(&action),
                    "{action:?}"
                );
                MarketDay::apply(&mut s, seat, &action).unwrap();
            }
        }
    }
}
