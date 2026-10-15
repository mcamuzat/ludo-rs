use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Phase, State, TrollGuts};
use crate::deck::Rank;

impl Determinize for TrollGuts {
    /// The other stacks (cards and order) and the deck are redrawn among
    /// the cards `observer` cannot place. While stacks are ordered, the
    /// other seats' choices are forgotten.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        {
            let mut others: Vec<&mut Vec<Rank>> = d
                .stacks
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, stack)| stack)
                .collect();
            d.supply.redeal_hidden_with(&[], &mut others, rng);
        }
        if d.phase == Phase::Arrange {
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

    use crate::troll_guts::{Config, TrollGuts};

    #[test]
    fn other_stacks_change_but_not_what_the_observer_sees() {
        let s = TrollGuts::new(&Config::default(), 4, 3);
        let view = TrollGuts::view(&s, Viewer::Seat(Seat(1)));
        let mut stacks = Vec::new();
        for seed in 0..20 {
            let d = TrollGuts::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(TrollGuts::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.stacks[1], s.stacks[1]);
            stacks.push(d.stacks[0].clone());
        }
        stacks.dedup();
        assert!(stacks.len() > 1);
    }
}
