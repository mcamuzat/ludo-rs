use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Starboard, State};

impl Determinize for Starboard {
    /// Only the draw pile's order is hidden, and the shuffles to come.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        d.supply.redeal_hidden(rng);
        d.seed = rng.next_u64();
        d
    }
}
