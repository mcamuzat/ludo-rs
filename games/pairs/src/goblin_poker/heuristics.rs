use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, GoblinPoker, Phase, State};
use crate::deck::{has_rank, lowest, Rank};

impl Heuristics for GoblinPoker {
    /// From what `seat` sees only. Fold when the card coming back is
    /// expected to cost more than folding plus a share of the pot;
    /// otherwise pitch the card that hurts the most stacks, and not a rank
    /// of one's own stack if there is another choice.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Play {
            return legal.choose(rng).cloned().unwrap_or(Action::Fold);
        }
        let active = s.active();
        let share = f64::from(s.pot) / active.len().max(1) as f64;
        let cost = f64::from(s.fold_cost().unwrap_or(0));
        if s.catch_cost(seat) > cost + share {
            return Action::Fold;
        }
        let mine = &s.stacks[seat.index()];
        // (not in my stack, stacks it would pair, rank): the highest wins.
        let worth = |card: Rank| {
            let hurt = active
                .iter()
                .filter(|&&other| other != seat && has_rank(&s.stacks[other.index()], card))
                .count();
            (!has_rank(mine, card), hurt, card)
        };
        legal
            .iter()
            .filter_map(|a| match a {
                Action::Pitch { card } => Some(*card),
                _ => None,
            })
            .max_by_key(|&card| worth(card))
            .map_or(Action::Fold, |card| Action::Pitch { card })
    }

    /// The first card is not worth a search (and a table of AIs would
    /// think one after the other): the lowest, the rank with the fewest
    /// copies to catch. In play, every legal action.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Setup || legal.is_empty() {
            return legal;
        }
        match lowest(&s.hands[seat.index()]) {
            Some(card) => vec![Action::Place { card }],
            None => legal,
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::goblin_poker::testing::{setup, table};
    use crate::goblin_poker::{Action, Config, GoblinPoker, State};
    use crate::testing::r;

    fn rollout(s: &State, seat: u8, seed: u64) -> Action {
        GoblinPoker::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(seed))
    }

    fn pitch(card: u8) -> Action {
        Action::Pitch { card: r(card) }
    }

    #[test]
    fn the_cost_of_a_catch_counts_the_cards_the_seat_cannot_place() {
        // 51 cards unseen by seat 0, nine of them 10s.
        let s = table(&[&[2], &[3], &[4]], &[&[10], &[5], &[6]], &[]);
        assert!((s.catch_cost(Seat(0)) - 90.0 / 51.0).abs() < 1e-9);
    }

    #[test]
    fn pitches_the_rank_most_stacks_hold() {
        let s = table(&[&[4, 6, 9], &[1], &[1]], &[&[4], &[6], &[6, 9]], &[]);
        for seed in 0..20 {
            assert_eq!(rollout(&s, 0, seed), pitch(6));
        }
    }

    #[test]
    fn keeps_the_ranks_of_its_own_stack_when_it_can() {
        let s = table(&[&[4, 2], &[1], &[1]], &[&[4], &[4], &[4]], &[]);
        for seed in 0..20 {
            assert_eq!(rollout(&s, 0, seed), pitch(2));
        }
        let forced = table(&[&[4], &[1], &[1]], &[&[4], &[4], &[4]], &[]);
        assert_eq!(rollout(&forced, 0, 1), pitch(4));
    }

    #[test]
    fn folds_when_the_risk_exceeds_the_cost() {
        let s = table(&[&[3], &[1], &[1]], &[&[7, 8, 9, 10], &[1], &[2]], &[]);
        assert!(s.catch_cost(Seat(0)) > 4.0);
        for seed in 0..20 {
            assert_eq!(rollout(&s, 0, seed), Action::Fold);
        }
    }

    #[test]
    fn stays_when_folding_is_expensive() {
        let s = table(&[&[3], &[1], &[1]], &[&[10], &[9], &[8]], &[]);
        assert_eq!(s.fold_cost(), Some(8));
        for seed in 0..20 {
            assert_eq!(rollout(&s, 0, seed), pitch(3));
        }
    }

    #[test]
    fn the_search_does_not_deliberate_over_the_first_card() {
        let s = setup(
            &[
                &[9, 4, 9, 6, 7, 7],
                &[2, 8, 3, 5, 5, 6],
                &[1, 1, 1, 1, 1, 1],
            ],
            &[],
        );
        assert_eq!(
            GoblinPoker::search_actions(&s, Seat(0)),
            vec![Action::Place { card: r(4) }]
        );
        let p = table(&[&[6, 7], &[8], &[8]], &[&[6], &[2], &[3]], &[]);
        assert_eq!(
            GoblinPoker::search_actions(&p, Seat(0)),
            GoblinPoker::legal_actions(&p, Seat(0))
        );
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 3..=8u8 {
            for _ in 0..5 {
                let mut s = GoblinPoker::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = GoblinPoker::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = GoblinPoker::rollout_action(&s, seat, &mut rng);
                    assert!(
                        GoblinPoker::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    GoblinPoker::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
