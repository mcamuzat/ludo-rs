use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::knowledge::sample_world;
use crate::{BlackVienna, Phase, State};

impl Determinize for BlackVienna {
    /// The other hands and the trio (unless `observer` has seen it) are
    /// redrawn among the worlds consistent with every answer, and the cards
    /// under the top of each pile reshuffled. While the others guess, their
    /// guesses are forgotten (they will guess again); a right accuser's
    /// names the drawn trio.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let (hands, trio) = sample_world(s, observer, rng);
        d.hands = hands;
        d.trio = trio;
        if d.phase == Phase::Guess {
            for (i, g) in d.guesses.iter_mut().enumerate() {
                let seat = Seat(i as u8);
                if Some(seat) == d.accuser {
                    // A right accusation: it names the (drawn) trio.
                    *g = Some(trio);
                } else if seat != observer {
                    *g = None;
                }
            }
        }
        // Only the top of each pile is face up.
        for pile in &mut d.piles {
            if let Some((_, under)) = pile.split_last_mut() {
                under.shuffle(rng);
            }
        }
        d.seed = rng.next_u64();
        d
    }
}
