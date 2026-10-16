use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{suit_index, Collusion, State};
use crate::deck::redeal_where;

impl Determinize for Collusion {
    /// The other three hands are redealt among themselves, sizes kept, and
    /// nobody gets a suit they failed to follow; the seed of the next deals
    /// is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let seats: Vec<usize> = (0..d.hands.len())
            .filter(|&i| i != observer.index())
            .collect();
        let mut others: Vec<&mut Vec<_>> = d
            .hands
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != observer.index())
            .map(|(_, h)| h)
            .collect();
        let voids = s.voids;
        redeal_where(
            &mut others,
            &|i, card| {
                card.suit()
                    .is_some_and(|suit| !voids[seats[i]][suit_index(suit)])
            },
            rng,
        );
        d.seed = rng.gen();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::collusion::testing::table;
    use crate::collusion::{Action, Collusion, Config};
    use crate::deck::Suit;

    #[test]
    fn determinize_respects_voids() {
        let mut s = table(
            ["2h 3h 4h 5h", "6h 7h 8h 9h", "2s 3s 4s 5s", "Th Jh Qh Kh"],
            0,
        );
        s.hands[2].push(crate::collusion::testing::c("2c"));
        // Seat 2 cannot follow hearts: it is void there.
        for seat in 0..3u8 {
            let card = Collusion::legal_actions(&s, Seat(seat))
                .into_iter()
                .find(|a| matches!(a, Action::Play { .. }))
                .unwrap();
            Collusion::apply(&mut s, Seat(seat), &card).unwrap();
        }
        assert!(s.voids[2][1]);
        let view = Collusion::view(&s, Viewer::Seat(Seat(0)));
        let mut changed = false;
        for seed in 0..100 {
            let d = Collusion::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(
                !d.hands[2].iter().any(|c| c.suit() == Some(Suit::Hearts)),
                "seed {seed}"
            );
            assert_eq!(d.hands[0], s.hands[0]);
            let sizes: Vec<usize> = d.hands.iter().map(Vec::len).collect();
            assert_eq!(sizes, s.hands.iter().map(Vec::len).collect::<Vec<_>>());
            assert_ne!(d.seed, s.seed);
            assert_eq!(Collusion::view(&d, Viewer::Seat(Seat(0))), view);
            changed |= d.hands[1] != s.hands[1];
        }
        assert!(changed, "the other hands are redealt");
    }

    #[test]
    fn a_fresh_deal_is_redealt() {
        let s = Collusion::new(&Config::default(), 4, 3);
        let d = Collusion::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.hands[1], s.hands[1]);
        assert_ne!(d.hands[0], s.hands[0]);
    }
}
