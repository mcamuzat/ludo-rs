use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Card, Cobras, State};
use crate::deck::redeal_where;

impl Determinize for Cobras {
    /// The other hands, the cards set aside and the dummy's hidden cards are
    /// redealt among themselves, sizes kept, no hand getting a suit its
    /// player is known to lack; the other players' secret basket cards are
    /// forgotten and the seed of the next rounds is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        for (i, chosen) in d.chosen.iter_mut().enumerate() {
            if i != o {
                *chosen = None;
            }
        }
        // The dummy's top card stays when it is face up.
        let keep = usize::from(d.dummy_top_shown()).min(d.dummy.len());
        let mut dummy_hidden = d.dummy.split_off(keep);
        let voids: Vec<Option<[bool; 4]>> = (0..d.hands.len())
            .filter(|&i| i != o)
            .map(|i| Some(d.void[i]))
            .chain([None, None])
            .collect();
        {
            let mut piles: Vec<&mut Vec<Card>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, h)| h)
                .collect();
            piles.push(&mut d.set_aside);
            piles.push(&mut dummy_hidden);
            redeal_where(
                &mut piles,
                &|k, card: Card| voids[k].is_none_or(|v| !v[card.suit.index()]),
                rng,
            );
        }
        d.dummy.append(&mut dummy_hidden);
        d.seed = rng.gen();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::cobras::{Action, Cobras, Config, DummyPile, Suit};

    #[test]
    fn hidden_cards_move_but_not_what_the_observer_sees() {
        let mut s = Cobras::new(&Config::default(), 3, 5);
        let card = s.hands[0][0];
        Cobras::apply(&mut s, Seat(0), &Action::Seed { card }).unwrap();
        let view = Cobras::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Cobras::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Cobras::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert_eq!(d.chosen[0], None);
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1);
    }

    #[test]
    fn a_known_void_is_kept() {
        let mut s = Cobras::new(&Config::default(), 4, 9);
        s.void[2][Suit::Red.index()] = true;
        // Make the state agree with the void: give seat 2's reds to seat 3.
        let reds: Vec<_> = s.hands[2]
            .iter()
            .copied()
            .filter(|c| c.suit == Suit::Red)
            .collect();
        for red in reds {
            let swap = s.hands[3].iter().position(|c| c.suit != Suit::Red).unwrap();
            let other = s.hands[3][swap];
            s.hands[3][swap] = red;
            let at = s.hands[2].iter().position(|&c| c == red).unwrap();
            s.hands[2][at] = other;
        }
        for seed in 0..20 {
            let d = Cobras::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(d.hands[2].iter().all(|c| c.suit != Suit::Red));
        }
    }

    #[test]
    fn the_dummy_top_stays_once_shown() {
        let mut s = Cobras::new(&Config::default(), 2, 1);
        for seat in [Seat(0), Seat(1)] {
            let card = s.hands[seat.index()][0];
            Cobras::apply(&mut s, seat, &Action::Seed { card }).unwrap();
        }
        let top = s.dummy[0];
        for seed in 0..10 {
            let d = Cobras::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(d.dummy[0], top);
            assert_eq!(d.dummy.len(), s.dummy.len());
        }
        let hidden = Config {
            dummy: DummyPile::Hidden,
        };
        let mut s = Cobras::new(&hidden, 2, 1);
        for seat in [Seat(0), Seat(1)] {
            let card = s.hands[seat.index()][0];
            Cobras::apply(&mut s, seat, &Action::Seed { card }).unwrap();
        }
        let tops: Vec<_> = (0..20)
            .map(|seed| {
                Cobras::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed)).dummy[0]
            })
            .collect();
        assert!(tops.iter().any(|&t| t != tops[0]));
    }
}
