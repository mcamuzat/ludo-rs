use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{remove_one, State, ToTheMoon};
use crate::deck::Rank;

impl Determinize for ToTheMoon {
    /// The other hands, except their known cards, and the order of the deck
    /// are redrawn among the cards `observer` cannot place. The seed of the
    /// next rounds' deals is redrawn too.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        // The unknown part of each other hand.
        let mut hidden: Vec<Vec<Rank>> = d
            .hands
            .iter_mut()
            .zip(&d.known)
            .enumerate()
            .map(|(i, (hand, known))| {
                if i == o {
                    return Vec::new();
                }
                let mut rest = std::mem::take(hand);
                for &card in known {
                    remove_one(&mut rest, card);
                    hand.push(card);
                }
                rest
            })
            .collect();
        {
            let mut others: Vec<&mut Vec<Rank>> = hidden.iter_mut().collect();
            d.supply.redeal_hidden_with(&[], &mut others, rng);
        }
        for (hand, rest) in d.hands.iter_mut().zip(hidden) {
            hand.extend(rest);
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

    use crate::testing::r;
    use crate::to_the_moon::testing::table;
    use crate::to_the_moon::{Action, Config, ToTheMoon};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = ToTheMoon::new(&Config::default(), 4, 3);
        let view = ToTheMoon::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = ToTheMoon::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(ToTheMoon::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert_eq!(d.trick, s.trick);
            assert_ne!(d.seed, s.seed);
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn a_card_everyone_saw_go_in_stays_in_its_hand() {
        let mut s = table(&[&[1, 2], &[3, 4, 5], &[6, 7]], &[9, 8], &[10, 10, 10]);
        ToTheMoon::apply(&mut s, Seat(1), &Action::Withdraw { take: Some(r(9)) }).unwrap();
        let view = ToTheMoon::view(&s, Viewer::Seat(Seat(2)));
        for seed in 0..20 {
            let d = ToTheMoon::determinize(&s, Seat(2), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(ToTheMoon::view(&d, Viewer::Seat(Seat(2))), view);
            assert!(d.hands[1].contains(&r(9)));
            assert_eq!(d.hands[1].len(), 4);
            assert_eq!(d.hands[2], s.hands[2]);
        }
    }
}
