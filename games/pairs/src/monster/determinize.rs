use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Monster, State};
use crate::deck::Rank;

impl Determinize for Monster {
    /// The other hands, the other seats' discards and folded cards, and the
    /// undealt cards are redrawn among the cards `observer` cannot place,
    /// each keeping its size. The observer's own discards stay.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        {
            let mut hidden: Vec<&mut Vec<Rank>> = Vec::new();
            for (i, ((hand, discards), folded)) in d
                .hands
                .iter_mut()
                .zip(d.discards.iter_mut())
                .zip(d.folded.iter_mut())
                .enumerate()
            {
                if i != o {
                    hidden.push(hand);
                    hidden.push(discards);
                    hidden.push(folded);
                }
            }
            d.supply.redeal_hidden_with(&[], &mut hidden, rng);
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

    use crate::monster::{Config, Monster};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let mut s = Monster::new(&Config::default(), 4, 3);
        let seat = s.turn.unwrap();
        let action = Monster::legal_actions(&s, seat)[0].clone();
        Monster::apply(&mut s, seat, &action).unwrap();
        let view = Monster::view(&s, Viewer::Seat(Seat(0)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Monster::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Monster::view(&d, Viewer::Seat(Seat(0))), view);
            assert_eq!(d.hands[0], s.hands[0]);
            hands.push(d.hands[2].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1);
    }
}
