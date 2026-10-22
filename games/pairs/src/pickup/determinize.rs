use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Pickup, State};
use crate::deck::{full_deck, Rank};

impl Determinize for Pickup {
    /// The other players' secret choices are forgotten; their hands and the
    /// deck are redrawn, at their sizes, among the cards `observer` cannot
    /// place (not in its hand, the pool, the discards, the piles, nor on the
    /// table).
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let mut unseen = full_deck();
        let table = s
            .waiting
            .iter()
            .map(|(_, c)| c)
            .chain(s.taking.as_ref().map(|(_, c)| c));
        for known in s.hands[o]
            .iter()
            .chain(&s.pool)
            .chain(&s.discard)
            .chain(s.piles.iter().flatten())
            .chain(table)
        {
            if let Some(at) = unseen.iter().position(|c| c == known) {
                unseen.swap_remove(at);
            }
        }
        unseen.shuffle(rng);
        let mut deal = |n: usize| -> Vec<Rank> { unseen.split_off(unseen.len() - n) };
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i != o {
                *hand = deal(hand.len());
                d.chosen[i] = None;
            }
        }
        d.deck = deal(d.deck.len());
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::pickup::testing::table;
    use crate::pickup::{Action, Config, Pickup};
    use crate::testing::r;

    #[test]
    fn hands_deck_and_choices_change_but_not_what_the_observer_sees() {
        let mut s = Pickup::new(&Config::default(), 4, 3);
        let card = s.hands[0][0];
        Pickup::apply(&mut s, Seat(0), &Action::Choose { card }).unwrap();
        let view = Pickup::view(&s, Viewer::Seat(Seat(1)));
        let mut decks = Vec::new();
        for seed in 0..20 {
            let d = Pickup::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Pickup::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert_eq!(d.chosen[0], None);
            assert_ne!(d.seed, s.seed);
            decks.push(d.deck.clone());
        }
        decks.dedup();
        assert!(decks.len() > 1);
    }

    #[test]
    fn cards_on_the_table_are_not_redealt() {
        let mut s = table(&[&[9, 1], &[1, 1]], &[9, 2], &[3, 4, 5]);
        Pickup::apply(&mut s, Seat(0), &Action::Choose { card: r(9) }).unwrap();
        Pickup::apply(&mut s, Seat(1), &Action::Choose { card: r(1) }).unwrap();
        let d = Pickup::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.taking, s.taking);
        assert_eq!(d.hands[0].len(), 1);
    }
}
