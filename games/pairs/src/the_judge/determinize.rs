use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{State, TheJudge};
use crate::deck::Rank;

impl Determinize for TheJudge {
    /// The other hands, the Judge and the undealt cards are redrawn among
    /// the cards `observer` cannot place.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        {
            let mut hidden: Vec<&mut Vec<Rank>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, hand)| hand)
                .collect();
            hidden.push(&mut d.judge);
            d.supply.redeal_hidden_with(&[], &mut hidden, rng);
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

    use crate::the_judge::{Config, TheJudge, HAND_SIZE, JUDGE_SIZE};

    #[test]
    fn the_judge_and_other_hands_change_but_not_what_the_observer_sees() {
        let s = TheJudge::new(&Config::default(), 4, 3);
        let view = TheJudge::view(&s, Viewer::Seat(Seat(1)));
        let mut judges = Vec::new();
        for seed in 0..20 {
            let d = TheJudge::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(TheJudge::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == HAND_SIZE));
            assert_eq!(d.judge.len(), JUDGE_SIZE);
            assert_eq!(d.supply.deck_len(), s.supply.deck_len());
            assert_ne!(d.seed, s.seed);
            judges.push(d.judge.clone());
        }
        judges.dedup();
        assert!(judges.len() > 1, "the Judge never changed");
    }
}
