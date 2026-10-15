use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Pendleton, State};

impl Determinize for Pendleton {
    /// Only the order of the draw pile is hidden, and the deals to come.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        d.supply.redeal_hidden(rng);
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::pendleton::{Config, Pendleton};

    #[test]
    fn the_draw_pile_changes_but_not_the_view() {
        let s = Pendleton::new(&Config::default(), 2, 3);
        let view = Pendleton::view(&s, Viewer::Seat(Seat(0)));
        let mut piles = Vec::new();
        for seed in 0..10 {
            let d = Pendleton::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Pendleton::view(&d, Viewer::Seat(Seat(0))), view);
            assert_ne!(d.seed, s.seed);
            piles.push(d.supply);
        }
        piles.dedup();
        assert!(piles.len() > 1, "the draw pile never moved");
    }
}
