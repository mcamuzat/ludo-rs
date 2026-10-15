use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{standings, Action, Phase, President, State};

impl State {
    /// A play that spends a joker or the strongest natural rank.
    fn spends_top_card(&self, play: &Action) -> bool {
        matches!(play, Action::Play { rank, jokers, .. }
            if *jokers > 0 || *rank == Some(self.top_rank()))
    }
}

impl Heuristics for President {
    /// From what `seat` sees only. Hand back the weakest cards; lead the
    /// whole set of the weakest rank; on a trick, climb as little as
    /// possible, but pass half the time rather than spend a top card.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let weakest = legal
            .iter()
            .filter(|a| **a != Action::Pass)
            .min_by_key(|a| s.cost(a))
            .cloned();
        let chosen = match s.phase {
            Phase::Exchange => weakest,
            Phase::Play => match weakest.clone() {
                None => Some(Action::Pass),
                Some(Action::Play { rank, .. }) if s.trick.is_none() => legal
                    .iter()
                    .filter(|a| matches!(a, Action::Play { rank: r, jokers: 0, .. } if *r == rank))
                    .max_by_key(|a| s.cost(a))
                    .cloned()
                    .or(weakest),
                Some(play) if s.spends_top_card(&play) && rng.gen_bool(0.5) => Some(Action::Pass),
                Some(play) => Some(play),
            },
            Phase::Over => None,
        };
        chosen
            .or_else(|| legal.choose(rng).cloned())
            .unwrap_or(Action::Pass)
    }

    /// A simulation stops once the round it started in is scored: the next
    /// deal is chance.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.round_no > root.round_no).then(|| Outcome::Ranked(standings(&sim.scores)).rewards())
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::president::{Config, Jokers, President};

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        let configs = [
            Config::default(),
            Config {
                rounds: 2,
                revolution: true,
                no_finish_on_two: true,
                jokers: Jokers::Wild,
            },
            Config {
                rounds: 2,
                revolution: true,
                no_finish_on_two: false,
                jokers: Jokers::High,
            },
        ];
        for config in &configs {
            for players in 4..=7u8 {
                let mut s = President::new(config, players, rng.gen());
                loop {
                    let pending = President::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = President::rollout_action(&s, seat, &mut rng);
                    assert!(
                        President::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    President::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }

    #[test]
    fn a_simulation_stops_when_its_round_is_scored() {
        let root = President::new(&Config::default(), 4, 2);
        assert_eq!(President::evaluate(&root, &root), None);
        let mut later = root.clone();
        later.round_no += 1;
        later.scores = vec![3, 2, 1, 0];
        assert_eq!(
            President::evaluate(&root, &later),
            Some(vec![1.0, 2.0 / 3.0, 1.0 / 3.0, 0.0])
        );
    }
}
