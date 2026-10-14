use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, CaptainsOrders, Phase, State};
use crate::deck::{count_by_rank, full_deck, lowest, Rank};

/// Share of simulated moves played at random, for variety.
const NOISE: f64 = 0.1;
/// From this rank on, a Mutineer's best card is worth an Order.
const HIGH_ORDER: u8 = 7;

impl State {
    /// Per rank, the cards `seat` cannot place: not in its hand, its
    /// discards, the face-up discards nor the Order.
    fn unseen_for(&self, seat: Seat) -> [u8; 10] {
        let mut counts = count_by_rank(&full_deck());
        let order = self.order.map(|o| o.card);
        for card in self.hands[seat.index()]
            .iter()
            .chain(&self.discarded[seat.index()])
            .chain(self.supply.seen())
            .chain(order.as_ref())
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// The chance, from what `seat` sees, that no other hand holds a card
    /// lower than its lowest. For the AI: never in a view.
    pub fn captain_chance(&self, seat: Seat) -> f64 {
        let Some(low) = lowest(&self.hands[seat.index()]) else {
            return 0.0;
        };
        let unseen = self.unseen_for(seat);
        let total: u32 = unseen.iter().map(|&c| u32::from(c)).sum();
        let lower: u32 = unseen[..low.index()].iter().map(|&c| u32::from(c)).sum();
        let held: u32 = self
            .hands
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != seat.index())
            .map(|(_, h)| h.len() as u32)
            .sum();
        // Every lower card must be among the `total - held` cards outside
        // the other hands.
        (0..lower).fold(1.0, |p, k| {
            if k >= total.saturating_sub(held) {
                0.0
            } else {
                p * f64::from(total - held - k) / f64::from(total - k)
            }
        })
    }
}

/// Following or belaying with `card`, whichever the Order allows.
fn answer(s: &State, card: Rank) -> Action {
    match s.order {
        Some(order) if card < order.card => Action::Order { card },
        _ => Action::Follow { card },
    }
}

impl Heuristics for CaptainsOrders {
    /// From what `seat` sees only. A likely Captain sheds its highest card
    /// at every chance, keeping its lowest for last. A Mutineer gets rid of
    /// its lowest card, orders with a high card or else asks the biggest
    /// hand for a card. Accusations are left to the search.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let fallback = || Action::Accuse { seat };
        if rng.gen_bool(NOISE) {
            return legal.choose(rng).cloned().unwrap_or_else(fallback);
        }
        let hand = &s.hands[seat.index()];
        let (Some(&low), Some(&high)) = (hand.iter().min(), hand.iter().max()) else {
            return legal.first().cloned().unwrap_or_else(fallback);
        };
        let captain = s.captain_chance(seat) >= 0.5;
        let shed = if captain { high } else { low };
        let action = match s.phase {
            Phase::Turn if captain => {
                if hand.len() == 1 {
                    Action::Order { card: low }
                } else {
                    let others: Vec<Seat> =
                        (0..s.players).map(Seat).filter(|&x| x != seat).collect();
                    match others.choose(rng) {
                        Some(&to) => Action::Pass { card: high, to },
                        None => fallback(),
                    }
                }
            }
            Phase::Turn if high.value() >= HIGH_ORDER => Action::Order { card: high },
            Phase::Turn => {
                let biggest = (0..s.players)
                    .map(Seat)
                    .filter(|&x| x != seat)
                    .max_by_key(|x| s.hands[x.index()].len());
                match biggest {
                    Some(from) => Action::Ask { from },
                    None => fallback(),
                }
            }
            Phase::Respond => answer(s, shed),
            Phase::Give { .. } => Action::Give { card: shed },
            Phase::Discard { .. } => Action::Discard { card: shed },
            Phase::Over => fallback(),
        };
        if legal.contains(&action) {
            action
        } else {
            legal.choose(rng).cloned().unwrap_or_else(fallback)
        }
    }

    /// A pass only considers the lowest and the highest card: the search
    /// stays on sensible moves at eight players.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        let hand = &s.hands[seat.index()];
        let ends = [hand.iter().min(), hand.iter().max()];
        let narrowed: Vec<Action> = legal
            .iter()
            .filter(|a| match a {
                Action::Pass { card, .. } => ends.contains(&Some(card)),
                _ => true,
            })
            .cloned()
            .collect();
        if narrowed.is_empty() {
            legal
        } else {
            narrowed
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::captains_orders::testing::table;
    use crate::captains_orders::{Action, CaptainsOrders, Config, Order, Phase};
    use crate::testing::r;

    #[test]
    fn holding_the_one_makes_a_sure_captain() {
        let s = table(&[&[1, 9], &[5, 6], &[7, 8], &[3, 4]], &[]);
        assert_eq!(s.captain_chance(Seat(0)), 1.0);
    }

    #[test]
    fn a_high_lowest_card_is_rarely_the_captain() {
        let s = table(&[&[8, 9], &[5, 6], &[7, 8], &[3, 4]], &[]);
        assert!(s.captain_chance(Seat(0)) < 0.05);
    }

    #[test]
    fn a_likely_captain_sheds_its_highest_card() {
        let mut s = table(&[&[1, 9], &[5, 6], &[7, 8], &[3, 4]], &[]);
        s.phase = Phase::Respond;
        s.order = Some(Order {
            card: r(5),
            owner: Seat(3),
        });
        let mut rng = SmallRng::seed_from_u64(2);
        let picks: Vec<Action> = (0..20)
            .map(|_| CaptainsOrders::rollout_action(&s, Seat(0), &mut rng))
            .collect();
        let sheds = picks
            .iter()
            .filter(|&a| *a == Action::Follow { card: r(9) })
            .count();
        assert!(sheds >= 14, "{picks:?}");
    }

    #[test]
    fn the_search_passes_only_the_extreme_cards() {
        let s = table(&[&[2, 5, 9], &[5, 6], &[7, 8], &[3, 4]], &[]);
        let actions = CaptainsOrders::search_actions(&s, Seat(0));
        assert!(!actions.contains(&Action::Pass {
            card: r(5),
            to: Seat(1)
        }));
        assert!(actions.contains(&Action::Pass {
            card: r(9),
            to: Seat(1)
        }));
        assert!(actions.contains(&Action::Accuse { seat: Seat(2) }));
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 4..=8u8 {
            for _ in 0..5 {
                let mut s = CaptainsOrders::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = CaptainsOrders::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = CaptainsOrders::rollout_action(&s, seat, &mut rng);
                    assert!(
                        CaptainsOrders::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    CaptainsOrders::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
