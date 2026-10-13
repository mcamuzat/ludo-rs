use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{GhostTown, Phase, State};
use crate::deck::Rank;

impl Determinize for GhostTown {
    /// The other hands, the draw pile and the face-down discards are redrawn
    /// among the cards `observer` cannot place. During setup the other
    /// seats' choices are forgotten: they will choose again from their new
    /// hands.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        {
            let mut others: Vec<&mut Vec<Rank>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, hand)| hand)
                .collect();
            d.supply.redeal_hidden_with(&[], &mut others, rng);
        }
        if d.phase == Phase::Setup {
            for (i, choice) in d.placed.iter_mut().enumerate() {
                if i != o {
                    *choice = None;
                }
            }
        }
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::ghost_town::testing::setup;
    use crate::ghost_town::{Action, Config, GhostTown};
    use crate::testing::r;

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = GhostTown::new(&Config::default(), 4, 3);
        let view = GhostTown::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = GhostTown::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(GhostTown::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 5));
            assert_eq!((d.pot, &d.chips), (s.pot, &s.chips));
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn during_setup_the_others_choose_again() {
        let mut s = setup(
            &[&[5, 6, 7, 8, 9], &[2, 3, 4, 8, 9], &[1, 4, 4, 10, 10]],
            &[3, 3],
        );
        for (seat, card) in [(0, 5), (1, 2)] {
            GhostTown::apply(&mut s, Seat(seat), &Action::Place { card: r(card) }).unwrap();
        }
        let d = GhostTown::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.placed, vec![Some(r(5)), None, None]);
        assert_eq!(
            GhostTown::visible_pending(&d, Viewer::Seat(Seat(0))),
            GhostTown::visible_pending(&s, Viewer::Seat(Seat(0)))
        );
    }
}
