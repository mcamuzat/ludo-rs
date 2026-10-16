//! Generic search: single-observer ISMCTS with per-seat rewards (max-n).
//! For a perfect-information game the determinization is skipped and this is
//! plain UCT.

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::{Determinize, Game, Heuristics, Illegal, Seat};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
    Easy,
    Medium,
    Hard,
}

impl Level {
    /// The platform's default budget: 100 / 1000 / 3000 ms in the browser,
    /// 100 / 1000 / 3000 iterations for the CLI and tests.
    pub fn default_budget(self, timed: bool) -> Budget {
        let amount = match self {
            Level::Easy => 100,
            Level::Medium => 1000,
            Level::Hard => 3000,
        };
        if timed {
            Budget::Millis(amount)
        } else {
            Budget::Iterations(amount)
        }
    }

    /// Probability of playing a random legal move instead of searching.
    pub fn blunder(self) -> f64 {
        match self {
            Level::Easy => 0.15,
            Level::Medium | Level::Hard => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    Iterations(u32),
    Millis(u32),
}

#[derive(Clone, Copy, Debug)]
pub struct Search {
    /// `None`: the rollout policy alone.
    pub budget: Option<Budget>,
    pub blunder: f64,
    pub seed: u64,
}

impl Search {
    pub fn for_game<G: Heuristics>(level: Level, timed: bool, seed: u64) -> Self {
        Self {
            budget: G::budget(level, timed),
            blunder: level.blunder(),
            seed,
        }
    }
}

/// UCB exploration constant, for rewards in [0, 1].
const EXPLORATION: f64 = 0.7;
/// The clock is read every this many iterations.
const CLOCK_EVERY: u32 = 16;

struct Node<A> {
    /// The action that leads here from the parent (none for the root).
    action: Option<A>,
    /// Who chose `action`: `reward` is theirs.
    mover: Seat,
    parent: Option<usize>,
    children: Vec<usize>,
    visits: u32,
    availability: u32,
    reward: f64,
}

impl<A> Node<A> {
    fn ucb(&self) -> f64 {
        let visits = f64::from(self.visits);
        self.reward / visits + EXPLORATION * (f64::from(self.availability).ln() / visits).sqrt()
    }
}

/// The action `seat` should take. `now_ms` is only read for `Budget::Millis`.
/// The search starts from a determinization, so it never uses what `seat`
/// cannot see even though it receives the full state.
pub fn choose<G: Game + Determinize + Heuristics>(
    state: &G::State,
    seat: Seat,
    search: &Search,
    now_ms: &dyn Fn() -> f64,
) -> Result<G::Action, Illegal> {
    let legal = G::legal_actions(state, seat);
    if legal.is_empty() {
        return Err(Illegal::new(format!(
            "Le siège {} n'a rien à décider.",
            seat.0
        )));
    }
    let mut rng = SmallRng::seed_from_u64(search.seed);
    if legal.len() == 1 {
        return Ok(legal[0].clone());
    }
    if let Some(action) = G::scripted_action(state, seat) {
        return Ok(action);
    }
    if search.blunder > 0.0 && rng.gen_bool(search.blunder) {
        if let Some(action) = G::random_actions(state, seat).choose(&mut rng) {
            return Ok(action.clone());
        }
    }
    let Some(budget) = search.budget else {
        // Policy alone, on what `seat` can see.
        let seen = if G::meta().hidden_info {
            G::determinize(state, seat, &mut rng)
        } else {
            state.clone()
        };
        return Ok(G::rollout_action(&seen, seat, &mut rng));
    };
    if let [only] = G::search_actions(state, seat).as_slice() {
        return Ok(only.clone());
    }
    let mut tree = vec![Node {
        action: None,
        mover: seat,
        parent: None,
        children: Vec::new(),
        visits: 0,
        availability: 0,
        reward: 0.0,
    }];
    let start = now_ms();
    let mut done = 0u32;
    loop {
        iterate::<G>(&mut tree, state, seat, &mut rng);
        done += 1;
        let stop = match budget {
            Budget::Iterations(n) => done >= n,
            Budget::Millis(ms) => {
                done.is_multiple_of(CLOCK_EVERY) && now_ms() - start >= f64::from(ms)
            }
        };
        if stop {
            break;
        }
    }
    // The most visited action is the most robust choice.
    let best = tree[0]
        .children
        .iter()
        .map(|&c| &tree[c])
        .filter(|n| n.mover == seat)
        .max_by_key(|n| n.visits)
        .and_then(|n| n.action.clone());
    Ok(best
        .filter(|a| legal.contains(a))
        .unwrap_or_else(|| legal[0].clone()))
}

fn iterate<G: Game + Determinize + Heuristics>(
    tree: &mut Vec<Node<G::Action>>,
    root: &G::State,
    me: Seat,
    rng: &mut SmallRng,
) {
    let mut sim = if G::meta().hidden_info {
        G::determinize(root, me, rng)
    } else {
        root.clone()
    };
    let mut current = 0;
    // Selection, then expansion of one new node.
    loop {
        let pending = G::pending(&sim);
        let Some(&first) = pending.0.first() else {
            break;
        };
        // At the root the searching seat decides first, even if others are pending too;
        // elsewhere simultaneous decisions are taken in seat order.
        let mover = if current == 0 { me } else { first };
        let legal = G::search_actions(&sim, mover);
        let available: Vec<usize> = tree[current]
            .children
            .iter()
            .copied()
            .filter(|&c| {
                tree[c].mover == mover && tree[c].action.as_ref().is_some_and(|a| legal.contains(a))
            })
            .collect();
        for &c in &available {
            tree[c].availability += 1;
        }
        let untried: Vec<&G::Action> = legal
            .iter()
            .filter(|a| {
                !available
                    .iter()
                    .any(|&c| tree[c].action.as_ref() == Some(*a))
            })
            .collect();
        if let Some(&action) = untried.choose(rng) {
            let action = action.clone();
            G::apply(&mut sim, mover, &action).expect("legal action");
            tree.push(Node {
                action: Some(action),
                mover,
                parent: Some(current),
                children: Vec::new(),
                visits: 0,
                availability: 1,
                reward: 0.0,
            });
            let child = tree.len() - 1;
            tree[current].children.push(child);
            current = child;
            break;
        }
        let best = available
            .into_iter()
            .max_by(|&a, &b| tree[a].ucb().total_cmp(&tree[b].ucb()))
            .expect("a pending seat has legal actions");
        let action = tree[best]
            .action
            .clone()
            .expect("only the root has no action");
        G::apply(&mut sim, mover, &action).expect("legal action");
        current = best;
    }
    let rewards = rollout::<G>(root, sim, rng);
    let mut node = Some(current);
    while let Some(i) = node {
        tree[i].visits += 1;
        tree[i].reward += rewards[tree[i].mover.index()];
        node = tree[i].parent;
    }
}

fn rollout<G: Game + Heuristics>(
    root: &G::State,
    mut sim: G::State,
    rng: &mut SmallRng,
) -> Vec<f64> {
    loop {
        if let Some(outcome) = G::outcome(&sim) {
            return outcome.rewards();
        }
        if let Some(estimate) = G::evaluate(root, &sim) {
            return estimate;
        }
        let seat = *G::pending(&sim)
            .0
            .first()
            .expect("an unfinished game has a pending seat");
        let action = G::rollout_action(&sim, seat, rng);
        G::apply(&mut sim, seat, &action).expect("rollout_action returns a legal action");
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::testgames::{Choice, HighCard, HighCardState, Nim, NimConfig};
    use crate::Heuristics;

    fn no_clock() -> f64 {
        0.0
    }

    fn counted(iterations: u32, seed: u64) -> Search {
        Search {
            budget: Some(Budget::Iterations(iterations)),
            blunder: 0.0,
            seed,
        }
    }

    #[test]
    fn nim_leaves_a_multiple_of_four() {
        for (pile, best) in [(5, 1), (6, 2), (7, 3)] {
            let s = Nim::new(&NimConfig { pile }, 2, 0);
            assert_eq!(
                choose::<Nim>(&s, Seat(0), &counted(3000, 1), &no_clock).unwrap(),
                best,
                "pile {pile}"
            );
        }
    }

    #[test]
    fn same_seed_same_move() {
        let s = Nim::new(&NimConfig { pile: 10 }, 3, 0);
        let a = choose::<Nim>(&s, Seat(0), &counted(500, 9), &no_clock).unwrap();
        let b = choose::<Nim>(&s, Seat(0), &counted(500, 9), &no_clock).unwrap();
        assert_eq!(a, b);
    }

    fn highcard(cards: Vec<u8>) -> HighCardState {
        let deck = (0..=6).filter(|c| !cards.contains(c)).collect();
        HighCardState {
            answers: vec![None; cards.len()],
            cards,
            deck,
        }
    }

    #[test]
    fn stays_with_the_best_card() {
        let s = highcard(vec![6, 2]);
        assert_eq!(
            choose::<HighCard>(&s, Seat(0), &counted(2000, 3), &no_clock).unwrap(),
            Choice::Stay
        );
    }

    #[test]
    fn does_not_read_hidden_cards() {
        for seed in 0..10 {
            let a = choose::<HighCard>(
                &highcard(vec![3, 1]),
                Seat(0),
                &counted(300, seed),
                &no_clock,
            )
            .unwrap();
            let b = choose::<HighCard>(
                &highcard(vec![3, 5]),
                Seat(0),
                &counted(300, seed),
                &no_clock,
            )
            .unwrap();
            assert_eq!(a, b, "seed {seed}");
        }
    }

    #[test]
    fn a_time_budget_stops_the_search() {
        let t = Cell::new(0.0);
        let clock = || {
            t.set(t.get() + 1.0);
            t.get()
        };
        let s = Nim::new(&NimConfig { pile: 20 }, 2, 0);
        let search = Search {
            budget: Some(Budget::Millis(50)),
            blunder: 0.0,
            seed: 0,
        };
        assert!(choose::<Nim>(&s, Seat(0), &search, &clock).is_ok());
    }

    #[test]
    fn a_blunder_is_still_legal() {
        let s = Nim::new(&NimConfig { pile: 10 }, 2, 0);
        let search = Search {
            budget: Some(Budget::Iterations(10)),
            blunder: 1.0,
            seed: 4,
        };
        let action = choose::<Nim>(&s, Seat(0), &search, &no_clock).unwrap();
        assert!(Nim::legal_actions(&s, Seat(0)).contains(&action));
    }

    #[test]
    fn scripted_action_is_played_without_search() {
        use crate::testgames::SteeredNim;
        let s = SteeredNim::new(&NimConfig { pile: 10 }, 2, 0);
        let search = Search {
            budget: Some(Budget::Iterations(10)),
            blunder: 1.0,
            seed: 4,
        };
        assert_eq!(
            choose::<SteeredNim>(&s, Seat(0), &search, &no_clock).unwrap(),
            3
        );
    }

    #[test]
    fn blunder_draws_from_random_actions() {
        use crate::testgames::SteeredNim;
        let s = SteeredNim::new(&NimConfig { pile: 9 }, 2, 0);
        for seed in 0..50 {
            let search = Search {
                budget: Some(Budget::Iterations(10)),
                blunder: 1.0,
                seed,
            };
            assert_eq!(
                choose::<SteeredNim>(&s, Seat(0), &search, &no_clock).unwrap(),
                1,
                "seed {seed}"
            );
        }
    }

    #[test]
    fn a_seat_without_decision_is_an_error() {
        let s = Nim::new(&NimConfig { pile: 10 }, 2, 0);
        assert!(choose::<Nim>(&s, Seat(1), &counted(10, 0), &no_clock).is_err());
    }

    #[test]
    fn evaluate_cuts_endless_simulations() {
        use crate::testgames::{Counter, CounterState};
        let s = CounterState { n: 0 };
        let search = Search {
            budget: Some(Budget::Iterations(50)),
            blunder: 0.0,
            seed: 1,
        };
        assert!(choose::<Counter>(&s, Seat(0), &search, &no_clock).is_ok());
    }

    #[test]
    fn no_budget_means_the_rollout_policy_alone() {
        let s = Nim::new(&NimConfig { pile: 10 }, 2, 0);
        let search = Search {
            budget: None,
            blunder: 0.0,
            seed: 7,
        };
        let expected = Nim::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(7));
        assert_eq!(
            choose::<Nim>(&s, Seat(0), &search, &no_clock).unwrap(),
            expected
        );
    }

    #[test]
    fn default_budgets_follow_the_level() {
        let s = Search::for_game::<Nim>(Level::Hard, true, 3);
        assert_eq!(s.budget, Some(Budget::Millis(3000)));
        let s = Search::for_game::<Nim>(Level::Easy, false, 3);
        assert_eq!(s.budget, Some(Budget::Iterations(100)));
        assert_eq!(s.blunder, 0.15);
    }

    #[test]
    fn the_search_only_considers_the_games_candidates() {
        use crate::testgames::{Counter, CounterState};
        for seed in 0..20 {
            let search = Search {
                budget: Some(Budget::Iterations(50)),
                blunder: 0.0,
                seed,
            };
            let s = CounterState { n: 0 };
            assert_eq!(
                choose::<Counter>(&s, Seat(0), &search, &no_clock).unwrap(),
                1,
                "seed {seed}"
            );
        }
    }
}
