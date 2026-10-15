use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Rocket, State};

impl Determinize for Rocket {
    /// Only the draw pile's order is hidden.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        d.supply.redeal_hidden(rng);
        d
    }
}
