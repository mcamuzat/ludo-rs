use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Hattrick, State};
use crate::deck::redeal_where;

impl Determinize for Hattrick {
    /// The other hands are redealt among themselves, sizes kept (playing
    /// another colour or passing is always a choice: it shows no void), and
    /// the seed of the next rounds is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let mut others: Vec<&mut Vec<_>> = d
            .hands
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != observer.index())
            .map(|(_, h)| h)
            .collect();
        redeal_where(&mut others, &|_, _| true, rng);
        d.seed = rng.gen();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::hattrick::{Config, Hattrick};

    #[test]
    fn the_other_hands_are_redealt() {
        let s = Hattrick::new(&Config::default(), 4, 3);
        let view = Hattrick::view(&s, Viewer::Seat(Seat(1)));
        let mut changed = false;
        for seed in 0..20 {
            let d = Hattrick::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 15));
            assert_ne!(d.seed, s.seed);
            assert_eq!(Hattrick::view(&d, Viewer::Seat(Seat(1))), view);
            changed |= d.hands[0] != s.hands[0];
        }
        assert!(changed);
    }
}
