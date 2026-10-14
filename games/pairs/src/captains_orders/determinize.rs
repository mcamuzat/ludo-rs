use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{CaptainsOrders, State};
use crate::deck::Rank;

impl Determinize for CaptainsOrders {
    /// The other hands, the undealt cards and the other seats' face-down
    /// discards are redrawn among the cards `observer` cannot place. Its
    /// own discards stay where they are.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let keep = d.discarded[o].clone();
        let mut others: Vec<&mut Vec<Rank>> = d
            .hands
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != o)
            .map(|(_, hand)| hand)
            .collect();
        d.supply.redeal_hidden_with(&keep, &mut others, rng);
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::captains_orders::testing::table;
    use crate::captains_orders::{Action, CaptainsOrders, Config};
    use crate::testing::r;

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = CaptainsOrders::new(&Config::default(), 5, 3);
        let view = CaptainsOrders::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = CaptainsOrders::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(CaptainsOrders::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 6));
            assert_eq!(d.supply.deck_len(), s.supply.deck_len());
            assert_ne!(d.seed, s.seed);
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn the_observer_keeps_its_discards() {
        let mut s = table(&[&[2, 9], &[5, 6], &[7, 8], &[3, 4]], &[1, 10, 10]);
        CaptainsOrders::apply(
            &mut s,
            Seat(0),
            &Action::Pass {
                card: r(9),
                to: Seat(1),
            },
        )
        .unwrap();
        CaptainsOrders::apply(&mut s, Seat(1), &Action::Discard { card: r(5) }).unwrap();
        for seed in 0..10 {
            let d = CaptainsOrders::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(d.supply.deck_len(), s.supply.deck_len());
            assert_eq!(d.supply.discard_len(), s.supply.discard_len());
            assert_eq!(d.discarded[1], vec![r(5)]);
            assert_eq!(d.hands[1], s.hands[1]);
        }
    }
}
