use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Card, DavidGoliath, State};
use crate::deck::redeal_where;

impl Determinize for DavidGoliath {
    /// The other hands are redealt among themselves, sizes kept, none
    /// getting a colour it is known to lack; the seed of the next rounds is
    /// redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let seats: Vec<usize> = (0..d.hands.len())
            .filter(|&i| i != observer.index())
            .collect();
        let voids: Vec<[bool; 5]> = seats.iter().map(|&i| d.void[i]).collect();
        let mut others: Vec<&mut Vec<Card>> = d
            .hands
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != observer.index())
            .map(|(_, h)| h)
            .collect();
        redeal_where(
            &mut others,
            &|k, card: Card| !voids[k][card.color.index()],
            rng,
        );
        d.seed = rng.gen();
        d
    }
}
