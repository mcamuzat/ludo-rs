use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{best_hand, matches, Action, Lamarckian, Phase, State};
use crate::deck::Card;

/// How often the rollout plays a random legal action instead.
const WANDER: f64 = 0.1;

/// A hand's worth for the rollout: its poker category first, then its size
/// (more cards, more ways to improve).
fn worth(hand: &[Card]) -> f64 {
    let category = best_hand(hand).map_or(0.0, |h| f64::from(h.category as u8));
    category * 10.0 + hand.len() as f64
}

impl State {
    /// The viewer's hand if it played `mark` now, before anyone else: the
    /// pool cards it would capture, or the best single one.
    fn after_mark(&self, seat: Seat, mark: Card) -> f64 {
        let mut hand: Vec<Card> = self.hands[seat.index()]
            .iter()
            .copied()
            .filter(|&c| c != mark)
            .collect();
        let captured: Vec<Card> = self
            .pool
            .iter()
            .copied()
            .filter(|&c| matches(mark, c))
            .collect();
        if captured.is_empty() {
            return self
                .pool
                .iter()
                .map(|&c| self.after_pick(&hand, c))
                .max_by(f64::total_cmp)
                .unwrap_or_else(|| worth(&hand));
        }
        hand.extend(captured);
        worth(&hand)
    }

    fn after_pick(&self, hand: &[Card], card: Card) -> f64 {
        let mut hand = hand.to_vec();
        hand.push(card);
        worth(&hand)
    }
}

impl Heuristics for Lamarckian {
    /// Mark the card that leaves the best hand; pick the pool card that
    /// improves the hand most.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if rng.gen_bool(WANDER) {
            return legal.choose(rng).cloned().expect("a legal action");
        }
        let value = |a: &Action| match (s.phase, a) {
            (Phase::Pick, Action::Pick { card }) => s.after_pick(&s.hands[seat.index()], *card),
            (_, Action::Mark { card }) => s.after_mark(seat, *card),
            _ => f64::NEG_INFINITY,
        };
        legal
            .iter()
            .max_by(|a, b| value(a).total_cmp(&value(b)))
            .cloned()
            .expect("a legal action")
    }

    /// Once the competition searched has ended, the scores decide: later
    /// competitions are dealt afresh and the search cannot see them.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.competition > root.competition || sim.phase == Phase::Over)
            .then(|| Outcome::Ranked(crate::standings(&sim.scores)).rewards())
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::lamarckian::testing::{c, table};
    use crate::lamarckian::{Action, Config, Lamarckian, Phase, State};

    fn usual(s: &State, seat: u8) -> Action {
        let mut rng = SmallRng::seed_from_u64(9);
        let mut counts: Vec<(Action, u32)> = Vec::new();
        for _ in 0..50 {
            let a = Lamarckian::rollout_action(s, Seat(seat), &mut rng);
            match counts.iter_mut().find(|(b, _)| *b == a) {
                Some((_, n)) => *n += 1,
                None => counts.push((a, 1)),
            }
        }
        counts.into_iter().max_by_key(|(_, n)| *n).unwrap().0
    }

    #[test]
    fn marks_the_card_that_makes_a_better_hand() {
        // The jack of clubs captures the third king; the kings would
        // capture more cards, but break the pair.
        let s = table(&["Kh Kd Jc 3s", "5c 6c 7c 8c"], "Kc Qh Qd 9s", "");
        assert_eq!(usual(&s, 0), Action::Mark { card: c("Jc") });
    }

    #[test]
    fn picks_the_card_that_pairs() {
        let mut s = table(&["Kh Qd 2d 3s", "5c 6c 7c 8c"], "Ks 9h", "");
        s.phase = Phase::Pick;
        s.picking = Some((Seat(0), c("Jc")));
        assert_eq!(usual(&s, 0), Action::Pick { card: c("Ks") });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=6u8 {
            let mut s = Lamarckian::new(&Config::default(), players, rng.gen());
            while let Some(&seat) = Lamarckian::pending(&s).0.first() {
                let action = Lamarckian::rollout_action(&s, seat, &mut rng);
                assert!(
                    Lamarckian::legal_actions(&s, seat).contains(&action),
                    "{action:?}"
                );
                Lamarckian::apply(&mut s, seat, &action).unwrap();
            }
        }
    }
}
