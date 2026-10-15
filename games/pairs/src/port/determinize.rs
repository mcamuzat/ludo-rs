use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Port, State};

impl Determinize for Port {
    /// Only the draw pile's order and the burned cards are hidden.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        d.supply.redeal_hidden(rng);
        d
    }
}
