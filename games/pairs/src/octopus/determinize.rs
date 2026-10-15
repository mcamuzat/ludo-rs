use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Octopus, State};
use crate::deck::Rank;

impl Determinize for Octopus {
    /// The other hands and the undealt cards are redrawn among the cards
    /// `observer` cannot place, each keeping its size. The card `observer`
    /// last passed stays with the player it went to while it is still
    /// there. The other seats' secret choices are forgotten.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        // Where the observer's card went, while the receiver has not
        // played that rank since (played cards are public).
        let pinned: Option<(usize, Rank)> = s.tracked[o]
            .map(|card| (s.next_active(observer).index(), card))
            .filter(|&(to, _)| to != o);
        let mut pool: Vec<Rank> = Vec::new();
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i != o {
                pool.append(hand);
            }
        }
        if let Some((_, card)) = pinned {
            if let Some(at) = pool.iter().position(|&c| c == card) {
                pool.swap_remove(at);
            }
        }
        pool.append(&mut d.undealt);
        pool.shuffle(rng);
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i == o {
                continue;
            }
            let mut size = s.hands[i].len();
            if let Some((to, card)) = pinned {
                if to == i {
                    hand.push(card);
                    size -= 1;
                }
            }
            hand.extend(pool.drain(pool.len() - size..));
        }
        d.undealt = pool;
        for i in 0..d.placed.len() {
            if i != o {
                d.placed[i] = None;
                d.passed[i] = None;
                d.tracked[i] = None;
                if pinned.is_none_or(|(to, _)| to != i) {
                    d.received[i] = None;
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

    use crate::octopus::{Config, Octopus};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = Octopus::new(&Config::default(), 4, 3);
        let view = Octopus::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Octopus::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Octopus::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 8));
            assert_ne!(d.seed, s.seed);
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }
}
