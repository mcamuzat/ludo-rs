use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Auberge, Phase, State};

/// Deals tried to find one where no seat proven innocent holds a pair.
const REDEALS: usize = 256;
use crate::deck::Rank;

impl Determinize for Auberge {
    /// The other hands, the draw pile and the face-down discards are redrawn
    /// among the cards `observer` cannot place; the face-down discards it
    /// has seen stay, and a seat everyone knows to be pair-free is not dealt
    /// a pair. During setup the other seats' choices are forgotten:
    /// they will choose again from their new hands.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let keep: Vec<Rank> = d.known.get(o).cloned().unwrap_or_default();
        for _ in 0..REDEALS {
            let mut others: Vec<&mut Vec<Rank>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, hand)| hand)
                .collect();
            d.supply.redeal_hidden_with(&keep, &mut others, rng);
            let contradicts = d.seats().any(|seat| {
                seat.index() != o && d.clean[seat.index()] && d.own_pair(seat).is_some()
            });
            if !contradicts {
                break;
            }
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

    use crate::auberge::testing::{setup, table};
    use crate::auberge::{Action, Auberge, Config};
    use crate::testing::{r, ranks};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = Auberge::new(&Config::default(), 4, 3);
        let view = Auberge::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Auberge::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Auberge::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 3));
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn a_hand_the_observer_saw_discarded_is_not_dealt_again() {
        // Seat 0 rightly accused seat 1 and saw its two 10s go face down.
        let mut s = table(&[&[5], &[10, 10], &[7]], &[&[2], &[3], &[4]], &[1, 6, 8, 9]);
        Auberge::apply(&mut s, Seat(0), &Action::Accuse { target: Seat(1) }).unwrap();
        assert_eq!(s.known[0], ranks(&[10, 10]));
        for seed in 0..30 {
            let d = Auberge::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(!d.hands[2].contains(&r(10)), "seed {seed}");
            assert_eq!(d.supply.discard_len(), s.supply.discard_len());
        }
        // Seat 2 did not see them: for it they may be anywhere.
        let dealt = (0..60).any(|seed| {
            let d = Auberge::determinize(&s, Seat(2), &mut SmallRng::seed_from_u64(seed));
            d.hands[0].contains(&r(10))
        });
        assert!(dealt);
    }

    #[test]
    fn during_setup_the_others_choose_again() {
        let mut s = setup(&[&[5, 6, 7], &[2, 8, 9], &[4, 4, 10]], &[1, 3]);
        for (seat, card) in [(0, 5), (1, 2)] {
            Auberge::apply(&mut s, Seat(seat), &Action::Place { card: r(card) }).unwrap();
        }
        let d = Auberge::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.placed, vec![Some(r(5)), None, None]);
        assert_eq!(
            Auberge::visible_pending(&d, Viewer::Seat(Seat(0))),
            Auberge::visible_pending(&s, Viewer::Seat(Seat(0)))
        );
    }

    #[test]
    fn a_player_proven_innocent_is_never_dealt_a_pair() {
        // Seat 0 wrongly accused seat 1, who holds seven cards without a pair.
        let mut s = table(
            &[&[5], &[1, 2, 3, 4, 6, 8, 9], &[7]],
            &[&[], &[], &[]],
            &[10, 10, 10, 9, 9, 8],
        );
        Auberge::apply(&mut s, Seat(0), &Action::Accuse { target: Seat(1) }).unwrap();
        assert!(s.clean[1]);
        for seed in 0..40 {
            let d = Auberge::determinize(&s, Seat(2), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(d.own_pair(Seat(1)), None, "seed {seed}");
            assert_eq!(d.hands[1].len(), 7);
        }
    }
}
