use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Phase, State, TrollGuts};
use crate::deck::{count_by_rank, full_deck, has_rank, Rank};

impl State {
    /// The chance, from what `seat` sees, that an unknown card pairs its
    /// face-up cards. For the AI: never in a view.
    pub fn unknown_risk(&self, seat: Seat) -> f64 {
        let mut unknown = count_by_rank(&full_deck());
        for card in self
            .faceup
            .iter()
            .flatten()
            .chain(&self.stacks[seat.index()])
        {
            unknown[card.index()] = unknown[card.index()].saturating_sub(1);
        }
        let total: u32 = unknown.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 1.0;
        }
        let hits: u32 = self.faceup[seat.index()]
            .iter()
            .map(|c| u32::from(unknown[c.index()]))
            .sum();
        f64::from(hits) / f64::from(total)
    }

    /// The order `seat` keeps its stack in: cards that do not pair its
    /// face-up cards on top, lowest first.
    pub fn safe_order(&self, seat: Seat) -> Vec<Rank> {
        let faceup = &self.faceup[seat.index()];
        let mut order = self.stacks[seat.index()].clone();
        order.sort_by_key(|&c| (has_rank(faceup, c), c));
        order
    }
}

impl Heuristics for TrollGuts {
    /// Order the stack safe cards first. Buy a known safe top card when an
    /// unknown card is risky; otherwise draw or steal.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        if s.phase == Phase::Arrange {
            return Action::Arrange {
                order: s.safe_order(seat),
            };
        }
        let legal = Self::legal_actions(s, seat);
        let own_top_safe = s.stacks[seat.index()]
            .last()
            .is_some_and(|&c| !has_rank(&s.faceup[seat.index()], c));
        if own_top_safe && s.unknown_risk(seat) > 0.15 && legal.contains(&Action::Buy) {
            return Action::Buy;
        }
        let blind: Vec<&Action> = legal.iter().filter(|a| **a != Action::Buy).collect();
        blind
            .choose(rng)
            .map(|&a| a.clone())
            .or_else(|| legal.first().cloned())
            .unwrap_or(Action::Draw)
    }

    /// Ordering a stack: the safe order, or the plain ones.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Arrange {
            return Self::legal_actions(s, seat);
        }
        let safe = s.safe_order(seat);
        let mut up = s.stacks[seat.index()].clone();
        up.sort_unstable();
        let mut down = up.clone();
        down.reverse();
        let mut orders = vec![safe, up, down];
        orders.dedup();
        orders
            .into_iter()
            .map(|order| Action::Arrange { order })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::troll_guts::testing::table;
    use crate::troll_guts::{Action, Config, TrollGuts};

    #[test]
    fn buys_a_safe_top_card_when_blind_cards_are_risky() {
        let s = table(&[&[2, 9], &[3]], &[&[10, 8, 7], &[1]], &[5]);
        let action = TrollGuts::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(action, Action::Buy);
    }

    #[test]
    fn the_rollout_and_search_actions_are_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 3..=6u8 {
            for _ in 0..5 {
                let mut s = TrollGuts::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = TrollGuts::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let legal = TrollGuts::legal_actions(&s, seat);
                    for a in TrollGuts::search_actions(&s, seat) {
                        assert!(legal.contains(&a), "{a:?}");
                    }
                    let action = TrollGuts::rollout_action(&s, seat, &mut rng);
                    assert!(legal.contains(&action), "{action:?}");
                    TrollGuts::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
