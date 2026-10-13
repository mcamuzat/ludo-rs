use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{GoblinPoker, Phase, State};
use crate::deck::Rank;

impl Determinize for GoblinPoker {
    /// The other hands, the cards the others have pitched, the draw pile
    /// and the face-down discards are redrawn among the cards `observer`
    /// cannot place. During setup the other seats' choices are forgotten:
    /// they will choose again from their new hands.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        // Each pitched card of another seat is a hidden hand of one card.
        let mut pool: Vec<Vec<Rank>> = d
            .pitched
            .iter_mut()
            .enumerate()
            .map(|(i, card)| match card {
                Some(_) if i != o => card.take().into_iter().collect(),
                _ => Vec::new(),
            })
            .collect();
        {
            let mut others: Vec<&mut Vec<Rank>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, hand)| hand)
                .chain(pool.iter_mut())
                .collect();
            d.supply.redeal_hidden_with(&[], &mut others, rng);
        }
        for (i, cards) in pool.into_iter().enumerate() {
            if let Some(&card) = cards.first() {
                d.pitched[i] = Some(card);
            }
        }
        if d.phase == Phase::Setup {
            for (i, choice) in d.placed.iter_mut().enumerate() {
                if i != o {
                    *choice = None;
                }
            }
        }
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::goblin_poker::testing::{setup, table};
    use crate::goblin_poker::{Action, Config, GoblinPoker};
    use crate::testing::r;

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = GoblinPoker::new(&Config::default(), 4, 3);
        let view = GoblinPoker::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = GoblinPoker::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(GoblinPoker::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 6));
            assert_eq!((d.pot, &d.chips), (s.pot, &s.chips));
            assert_ne!(d.seed, s.seed);
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn the_cards_the_others_pitched_are_redrawn() {
        let rest: Vec<u8> = (1..=10u8).flat_map(|v| vec![v; usize::from(v)]).collect();
        let mut s = table(&[&[8, 9], &[8, 9], &[8, 9]], &[&[5], &[3], &[7]], &rest);
        for seat in [0, 1] {
            GoblinPoker::apply(&mut s, Seat(seat), &Action::Pitch { card: r(8) }).unwrap();
        }
        let view = GoblinPoker::view(&s, Viewer::Seat(Seat(1)));
        let mut pitched = Vec::new();
        for seed in 0..30 {
            let d = GoblinPoker::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(GoblinPoker::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.pitched[1], Some(r(8)));
            assert_eq!(d.pitched[2], None);
            assert_eq!((d.hands[0].len(), d.hands[2].len()), (1, 2));
            pitched.push(d.pitched[0].expect("seat 0 has pitched"));
        }
        pitched.sort_unstable();
        pitched.dedup();
        assert!(pitched.len() > 1, "seat 0 always pitched the same card");
    }

    #[test]
    fn during_setup_the_others_choose_again() {
        let mut s = setup(
            &[
                &[5, 6, 7, 8, 9, 9],
                &[2, 3, 4, 8, 9, 10],
                &[1, 4, 4, 6, 10, 10],
            ],
            &[3, 3],
        );
        for (seat, card) in [(0, 5), (1, 2)] {
            GoblinPoker::apply(&mut s, Seat(seat), &Action::Place { card: r(card) }).unwrap();
        }
        let d = GoblinPoker::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.placed, vec![Some(r(5)), None, None]);
        assert_eq!(
            GoblinPoker::visible_pending(&d, Viewer::Seat(Seat(0))),
            GoblinPoker::visible_pending(&s, Viewer::Seat(Seat(0)))
        );
    }
}
