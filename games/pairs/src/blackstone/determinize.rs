use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Blackstone, State};

impl Determinize for Blackstone {
    /// Only the deck's order is hidden, and the shuffles to come.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        d.supply.redeal_hidden(rng);
        d.seed = rng.next_u64();
        d
    }
}
