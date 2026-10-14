use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{CaptainsOrders, State};
use crate::deck::Rank;

/// Redeals drawn to pick one from, by how well each explains the moves
/// seen (0.3 per misfit and 8 redeals measured best).
const SAMPLES: usize = 8;

/// The other hands, the undealt cards and the other seats' face-down
/// discards redrawn among the cards `observer` cannot place; its own
/// discards stay where they are.
fn redeal<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
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

impl Determinize for CaptainsOrders {
    /// A redeal of everything `observer` cannot place. Once the others have
    /// shown tells, one of several redeals, picked with a chance
    /// proportional to how well it explains them: the AI reads the table.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let telling = (0..s.players)
            .map(Seat)
            .any(|x| x != observer && s.tells(x) != Default::default());
        if !telling {
            return redeal(s, observer, rng);
        }
        let mut candidates: Vec<(State, f64)> = (0..SAMPLES)
            .map(|_| {
                let d = redeal(s, observer, rng);
                let w = d.tells_likelihood(observer);
                (d, w)
            })
            .collect();
        let total: f64 = candidates.iter().map(|(_, w)| w).sum();
        let mut pick = rng.gen::<f64>() * total;
        let mut chosen = candidates.len() - 1;
        for (i, (_, w)) in candidates.iter().enumerate() {
            if pick < *w {
                chosen = i;
                break;
            }
            pick -= w;
        }
        candidates.swap_remove(chosen).0
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::captains_orders::testing::table;
    use crate::captains_orders::{Action, CaptainsOrders, Config, Event};
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

    #[test]
    fn a_seat_that_keeps_passing_is_more_often_the_captain() {
        let quiet = CaptainsOrders::new(&Config::default(), 4, 9);
        let mut telling = quiet.clone();
        for _ in 0..3 {
            telling.log.push(Event::Passed {
                from: Seat(2),
                to: Seat(1),
                card: None,
            });
        }
        let share = |s: &crate::captains_orders::State| {
            let mut rng = SmallRng::seed_from_u64(4);
            (0..2000)
                .filter(|_| {
                    CaptainsOrders::determinize(s, Seat(0), &mut rng)
                        .captains()
                        .contains(&Seat(2))
                })
                .count()
        };
        let (before, after) = (share(&quiet), share(&telling));
        assert!(after > before * 2, "{before} → {after}");
    }
}
