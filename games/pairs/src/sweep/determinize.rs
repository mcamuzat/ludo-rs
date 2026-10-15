use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{State, Sweep};
use crate::deck::Rank;

impl Determinize for Sweep {
    /// The unknown part of the other hands and the deck are redrawn among
    /// the cards `observer` cannot place; captured cards everyone saw stay
    /// in their hands.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let mut unknown: Vec<Vec<Rank>> = Vec::new();
        for (i, hand) in s.hands.iter().enumerate() {
            let mut rest = hand.clone();
            if i != o {
                for card in &s.known[i] {
                    if let Some(at) = rest.iter().position(|c| c == card) {
                        rest.remove(at);
                    }
                }
            }
            unknown.push(rest);
        }
        {
            let mut others: Vec<&mut Vec<Rank>> = unknown
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, h)| h)
                .collect();
            d.supply.redeal_hidden_with(&[], &mut others, rng);
        }
        for (i, rest) in unknown.into_iter().enumerate() {
            if i != o {
                let mut hand = s.known[i].clone();
                hand.extend(rest);
                d.hands[i] = hand;
            }
        }
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::sweep::testing::table;
    use crate::sweep::{Config, Sweep};
    use crate::testing::{r, ranks};

    #[test]
    fn captured_cards_stay_and_the_rest_moves() {
        let mut s = Sweep::new(&Config::default(), 4, 3);
        s.known[2] = vec![s.hands[2][0]];
        let view = Sweep::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Sweep::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Sweep::view(&d, Viewer::Seat(Seat(1))), view);
            assert!(d.hands[2].contains(&s.known[2][0]));
            assert_eq!(d.hands[1], s.hands[1]);
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1);
        let t = table(&[&[1, 2], &[3]], &[9], 0, &[]);
        let d = Sweep::determinize(&t, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.hands[0], ranks(&[1, 2]));
        assert_eq!(d.hands[1], vec![r(3)]);
    }
}
