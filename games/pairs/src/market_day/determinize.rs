use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{MarketDay, State};
use crate::deck::{full_deck, Rank};

impl Determinize for MarketDay {
    /// The other hands and the undealt cards are redrawn, at their sizes,
    /// among the cards `observer` has not seen (its hand, the round, the
    /// cards of earlier rounds).
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let mut unseen = full_deck();
        for known in s.hands[o]
            .iter()
            .chain(s.trick.iter().map(|(_, c)| c))
            .chain(&s.discard)
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
        d.undealt = deal(d.undealt.len());
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
    use crate::market_day::{Config, MarketDay};

    #[test]
    fn hands_change_but_not_what_the_observer_sees() {
        let mut s = MarketDay::new(&Config::default(), 4, 3);
        let seat = s.turn;
        let card = s.hands[seat.index()][0];
        MarketDay::apply(&mut s, seat, &crate::market_day::Action::Play { card }).unwrap();
        let view = MarketDay::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = MarketDay::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(MarketDay::view(&d, Viewer::Seat(Seat(1))), view);
            let mut all: Vec<_> = d
                .undealt
                .iter()
                .chain(d.hands.iter().flatten())
                .chain(d.trick.iter().map(|(_, c)| c))
                .chain(&d.discard)
                .copied()
                .collect();
            all.sort_unstable();
            assert_eq!(all, full_deck());
            hands.push(d.hands[(seat.index() + 2) % 4].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1);
    }
}
