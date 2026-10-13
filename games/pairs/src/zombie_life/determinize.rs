use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{State, ZombieLife};

impl Determinize for ZombieLife {
    /// Everything on the table is public, whoever observes: only the order
    /// of the draw pile and the seed of the tie-breaks are redrawn.
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

    use crate::zombie_life::testing::dealing;
    use crate::zombie_life::{Action, Config, ZombieLife};

    #[test]
    fn the_next_card_changes_but_not_the_view() {
        let s = ZombieLife::new(&Config::default(), 4, 5);
        let seat = ZombieLife::pending(&s).0[0];
        let view = ZombieLife::view(&s, Viewer::Seat(seat));
        let mut next_cards = Vec::new();
        for seed in 0..20 {
            let mut d = ZombieLife::determinize(&s, seat, &mut SmallRng::seed_from_u64(seed));
            assert_eq!(ZombieLife::view(&d, Viewer::Seat(seat)), view);
            ZombieLife::apply(&mut d, seat, &Action::Draw).unwrap();
            next_cards.push(d.events.clone());
        }
        next_cards.dedup();
        assert!(
            next_cards.len() > 1,
            "every determinization drew the same card"
        );
    }

    #[test]
    fn the_dealer_keeps_its_cards() {
        let s = dealing(&[3, 0, 3], 1, &[4, 8, 2], &[1, 5, 9, 9, 7]);
        for seed in 0..10 {
            let d = ZombieLife::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(d.dealing, s.dealing);
            assert_eq!(
                ZombieLife::view(&d, Viewer::Seat(Seat(0))),
                ZombieLife::view(&s, Viewer::Seat(Seat(0)))
            );
        }
    }

    #[test]
    fn two_determinizations_differ_in_pile_and_seed() {
        let s = ZombieLife::new(&Config::default(), 3, 5);
        let a = ZombieLife::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        let b = ZombieLife::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(2));
        assert_ne!(a.supply, b.supply);
        assert_ne!(a.seed, b.seed);
        assert_ne!(a.seed, s.seed);
    }
}
