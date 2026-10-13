use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{ContinuousPairs, State};

impl Determinize for ContinuousPairs {
    /// Nobody sees the draw pile or the burned cards, whoever observes:
    /// their order is redrawn, everything else is public.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        d.supply.redeal_hidden(rng);
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::continuous::{Action, Config, ContinuousPairs};

    #[test]
    fn the_next_card_changes_but_not_the_view() {
        let s = ContinuousPairs::new(&Config::default(), 3, 5);
        let seat = ContinuousPairs::pending(&s).0[0];
        let view = ContinuousPairs::view(&s, Viewer::Seat(seat));
        let mut next_cards = Vec::new();
        for seed in 0..20 {
            let mut d = ContinuousPairs::determinize(&s, seat, &mut SmallRng::seed_from_u64(seed));
            assert_eq!(ContinuousPairs::view(&d, Viewer::Seat(seat)), view);
            ContinuousPairs::apply(&mut d, seat, &Action::Hit).unwrap();
            next_cards.push(d.events.clone());
        }
        next_cards.dedup();
        assert!(
            next_cards.len() > 1,
            "every determinization drew the same card"
        );
    }

    #[test]
    fn two_determinizations_reshuffle_differently() {
        let s = ContinuousPairs::new(&Config::default(), 2, 5);
        let a = ContinuousPairs::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        let b = ContinuousPairs::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(2));
        assert_ne!(a.supply, b.supply);
    }
}
