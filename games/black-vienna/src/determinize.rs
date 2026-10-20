use ludo_core::{Determinize, Seat};
use rand::Rng;

use crate::knowledge::sample_world;
use crate::{BlackVienna, Phase, State};

impl Determinize for BlackVienna {
    /// The other hands and the trio (unless `observer` has seen it) are
    /// redrawn among the worlds consistent with every answer. While the
    /// others guess, their guesses are forgotten: they will guess again.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let (hands, trio) = sample_world(s, observer, rng);
        d.hands = hands;
        d.trio = trio;
        if d.phase == Phase::Guess {
            for (i, g) in d.guesses.iter_mut().enumerate() {
                if i != observer.index() && Some(Seat(i as u8)) != d.accuser {
                    *g = None;
                }
            }
        }
        d.seed = rng.next_u64();
        d
    }
}
