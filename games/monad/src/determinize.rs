use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{Color, Monad, Rank, State};

impl Determinize for Monad {
    /// Redraws the colours `observer` cannot see: for each rank, the cards
    /// in the other hands (and, for Units, the face-down deck), keeping how
    /// many cards of each rank every hand holds and the deck's size.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let others: Vec<usize> = (0..d.hands.len())
            .filter(|&i| i != observer.index())
            .collect();
        for rank in Rank::ALL {
            let mut pool: Vec<Color> = others
                .iter()
                .flat_map(|&i| d.hands[i].iter().filter(|c| c.rank == rank))
                .map(|c| c.color)
                .collect();
            if rank == Rank::Unit {
                pool.append(&mut d.deck);
            }
            pool.shuffle(rng);
            for &i in &others {
                for card in d.hands[i].iter_mut().filter(|c| c.rank == rank) {
                    card.color = pool.pop().expect("as many colours as cards");
                }
            }
            if rank == Rank::Unit {
                d.deck = pool;
            }
        }
        for &i in &others {
            d.hands[i].sort();
        }
        // Units drawn by the others stay unknown.
        for event in d.this_turn.iter_mut().chain(d.last_turn.iter_mut()) {
            if event.seat != observer {
                event.drawn = event.drawn.map(|_| Color::Red);
            }
        }
        d
    }
}
