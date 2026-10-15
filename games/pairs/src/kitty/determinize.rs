use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Kitty, Phase, State};
use crate::deck::Rank;

impl Determinize for Kitty {
    /// The other hands, the kitty and the removed cards are redrawn among
    /// the cards `observer` cannot place, each keeping its size. The card
    /// `observer` passed stays with the player on its left while it is
    /// still there. The other seats' secret choices are forgotten.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let left = s.left(observer).index();
        // Still there unless the player on the left bid that rank (bids
        // are public; a card passed only leaves a hand as a bid).
        let pinned: Option<Rank> = s.passed[o].filter(|&card| s.bids.get(left) != Some(&card));
        let mut pool: Vec<Rank> = Vec::new();
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i == o {
                continue;
            }
            pool.append(hand);
        }
        if let Some(card) = pinned {
            if let Some(at) = pool.iter().position(|&c| c == card) {
                pool.swap_remove(at);
            }
        }
        pool.append(&mut d.kitty);
        pool.append(&mut d.removed);
        pool.shuffle(rng);
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i == o {
                continue;
            }
            let mut size = s.hands[i].len();
            if i == left {
                if let Some(card) = pinned {
                    hand.push(card);
                    size -= 1;
                }
            }
            hand.extend(pool.drain(pool.len() - size..));
        }
        d.kitty = pool.drain(pool.len() - s.kitty.len()..).collect();
        d.removed = pool;
        if matches!(d.phase, Phase::Pass | Phase::Bid) {
            for (i, choice) in d.placed.iter_mut().enumerate() {
                if i != o {
                    *choice = None;
                }
            }
        }
        // What the others passed and received is theirs to know.
        for i in 0..d.passed.len() {
            if i != o {
                d.passed[i] = None;
                if i != left {
                    d.received[i] = None;
                }
            }
        }
        if let Some(card) = pinned {
            d.received[left] = Some(card);
        }
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::kitty::{Action, Config, Kitty};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = Kitty::new(&Config::default(), 4, 3);
        let view = Kitty::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Kitty::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Kitty::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 12));
            assert_eq!(d.kitty.len(), s.kitty.len());
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn the_passed_card_stays_on_the_left() {
        let mut s = Kitty::new(&Config::default(), 4, 5);
        for seat in 0..4u8 {
            let card = s.hands[usize::from(seat)][0];
            Kitty::apply(&mut s, Seat(seat), &Action::Pass { card }).unwrap();
        }
        let passed = s.passed[0].unwrap();
        for seed in 0..20 {
            let d = Kitty::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(d.hands[1].contains(&passed));
            assert_eq!(d.received[1], Some(passed));
            assert_eq!(d.passed[1], None);
        }
    }
}
