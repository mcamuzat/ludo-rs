use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Deadfall, Phase, State};
use crate::deck::Rank;

impl Determinize for Deadfall {
    /// The other hands and the undealt cards are redrawn among the cards
    /// `observer` cannot place. While door cards are chosen the other
    /// seats' choices are forgotten: they will choose again.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        {
            let mut others: Vec<&mut Vec<Rank>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, hand)| hand)
                .collect();
            d.supply.redeal_hidden_with(&[], &mut others, rng);
        }
        if d.phase == Phase::Door {
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

    use crate::deadfall::testing::door;
    use crate::deadfall::{Action, Config, Deadfall};
    use crate::testing::r;

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = Deadfall::new(&Config::default(), 4, 3);
        let view = Deadfall::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Deadfall::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Deadfall::view(&d, Viewer::Seat(Seat(1))), view);
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
    fn the_others_choose_their_door_again() {
        let mut s = door(&[&[5, 6], &[2, 3], &[4, 9]], &[1, 7, 7]);
        for (seat, card) in [(0, 5), (1, 2)] {
            Deadfall::apply(&mut s, Seat(seat), &Action::Door { card: r(card) }).unwrap();
        }
        let d = Deadfall::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.placed, vec![Some(r(5)), None, None]);
    }
}
