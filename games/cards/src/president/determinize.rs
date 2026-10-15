use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{President, State};
use crate::deck::{redeal, Card};

impl Determinize for President {
    /// The other hands are redrawn among themselves, each keeping its size.
    /// Cards the observer handed over stay with whoever received them while
    /// they are still there; the other seats' exchange is forgotten, and
    /// the seed of future deals is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let mut pinned: Vec<(usize, Card)> = Vec::new();
        for t in s.transfers.iter().filter(|t| t.from == observer) {
            let hand = &mut d.hands[t.to.index()];
            for card in &t.cards {
                if let Some(at) = hand.iter().position(|c| c == card) {
                    pinned.push((t.to.index(), hand.remove(at)));
                }
            }
        }
        let mut others: Vec<&mut Vec<Card>> = d
            .hands
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != o)
            .map(|(_, h)| h)
            .collect();
        redeal(&mut others, rng);
        for (i, card) in pinned {
            d.hands[i].push(card);
        }
        d.transfers
            .retain(|t| t.from == observer || t.to == observer);
        d.seed = rng.gen();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::president::testing::{c, next_round};
    use crate::president::{Config, President, Title};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = President::new(&Config::default(), 4, 3);
        let view = President::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = President::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(President::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 13));
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn cards_handed_over_stay_with_who_received_them() {
        let s = next_round(
            Config::default(),
            &[
                Title::Scum,
                Title::President,
                Title::Secretary,
                Title::VicePresident,
            ],
            &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
        );
        for seed in 0..20 {
            let d = President::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(d.hands[1].contains(&c("2h")) && d.hands[1].contains(&c("Ks")));
            assert!(d
                .transfers
                .iter()
                .all(|t| t.from == Seat(0) || t.to == Seat(0)));
            let d = President::determinize(&s, Seat(2), &mut SmallRng::seed_from_u64(seed));
            assert!(d.hands[3].contains(&c("Ah")));
        }
    }
}
