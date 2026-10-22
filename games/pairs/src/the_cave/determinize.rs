use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{State, TheCave};
use crate::deck::{full_deck, Rank};

impl Determinize for TheCave {
    /// The other hands and the deck are redrawn, at their sizes, among the
    /// cards `observer` cannot place: not in its hand, the Cave, nor anyone's
    /// captures.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let mut unseen = full_deck();
        for known in s.hands[o]
            .iter()
            .chain(&s.cave)
            .chain(s.captured.iter().flatten())
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

    use crate::deck::full_deck;
    use crate::the_cave::{Config, TheCave};

    #[test]
    fn hands_and_deck_change_but_not_what_the_observer_sees() {
        let s = TheCave::new(&Config::default(), 4, 3);
        let view = TheCave::view(&s, Viewer::Seat(Seat(1)));
        let mut decks = Vec::new();
        for seed in 0..20 {
            let d = TheCave::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(TheCave::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            let mut all: Vec<_> = d
                .deck
                .iter()
                .chain(d.hands.iter().flatten())
                .chain(&d.cave)
                .chain(d.captured.iter().flatten())
                .copied()
                .collect();
            all.sort_unstable();
            assert_eq!(all, full_deck());
            assert_ne!(d.seed, s.seed);
            decks.push(d.deck.clone());
        }
        decks.dedup();
        assert!(decks.len() > 1);
    }
}
