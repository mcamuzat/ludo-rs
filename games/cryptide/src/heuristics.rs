//! The AI: a policy by level after Cryptid-Bot's method [B], from what the
//! seat can see (the tokens and its own hint).

use ludo_core::ai::{Budget, Level};
use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::knowledge::{possible, query, Combos, Inference, CAP};
use crate::map::{Cell, Mask};
use crate::rules::{Action, Cryptide, Event, Phase, State};

/// [B] The share of the asked player's possible hints that would answer
/// with a cube, aimed at when choosing a question.
pub fn ideal_cube_share(players: u8) -> f64 {
    match players {
        0..=3 => 0.35,
        4 => 0.30,
        _ => 0.25,
    }
}

/// [B] `check_to_search`: at most this many candidate habitats …
const SEARCH_CANDIDATES: usize = 4;
/// … and the left neighbour likely enough to lay a disc.
const SEARCH_DISC_CHANCE: f64 = 0.49;

/// The share of `list` (hint indices) whose spaces contain `cell`.
fn share(s: &State, list: &[usize], cell: Cell) -> f64 {
    if list.is_empty() {
        return 0.0;
    }
    let table = s.table();
    list.iter()
        .filter(|&&i| table.masks[i].contains(cell))
        .count() as f64
        / list.len() as f64
}

/// Equal-best elements, one drawn at random.
fn best_by<T: Clone, R: Rng + ?Sized>(
    items: &[T],
    key: impl Fn(&T) -> f64,
    rng: &mut R,
) -> Option<T> {
    let best = items.iter().map(&key).fold(f64::NEG_INFINITY, f64::max);
    let top: Vec<T> = items
        .iter()
        .filter(|x| (key(x) - best).abs() < 1e-9)
        .cloned()
        .collect();
    top.choose(rng).cloned()
}

struct Seen<'a> {
    s: &'a State,
    me: Seat,
    legal: Vec<Action>,
    /// What the others can tell of `me`'s hint.
    mine_known: Vec<usize>,
}

impl Seen<'_> {
    fn cubes(&self) -> Vec<Cell> {
        self.legal
            .iter()
            .filter_map(|a| match *a {
                Action::Cube(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    /// [B] A cube where it rules out the fewest of one's hints, as the
    /// others see them.
    fn quiet_cube<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<Action> {
        let cells = self.cubes();
        best_by(&cells, |&c| -share(self.s, &self.mine_known, c), rng).map(Action::Cube)
    }

    /// A relocated disc where it rules out the fewest of one's hints.
    fn quiet_disc<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<Action> {
        let cells: Vec<Cell> = self
            .legal
            .iter()
            .filter_map(|a| match *a {
                Action::Disc(c) => Some(c),
                _ => None,
            })
            .collect();
        best_by(&cells, |&c| share(self.s, &self.mine_known, c), rng).map(Action::Disc)
    }

    fn may_search(&self, c: Cell) -> bool {
        self.legal.contains(&Action::Search(c))
    }

    fn asks_of(&self, target: Seat) -> Vec<Cell> {
        self.legal
            .iter()
            .filter_map(|a| match *a {
                Action::Ask { cell, target: t } if t == target => Some(cell),
                _ => None,
            })
            .collect()
    }

    fn others(&self) -> impl Iterator<Item = Seat> + '_ {
        (0..self.s.players).map(Seat).filter(move |&p| p != self.me)
    }

    fn left(&self) -> Seat {
        Seat((self.me.0 + 1) % self.s.players)
    }
}

/// Easy: each opponent's possible hints on their own; search once only one
/// space fits them all, else a random question; cubes at random.
fn easy<R: Rng + ?Sized>(seen: &Seen, rng: &mut R) -> Option<Action> {
    let s = seen.s;
    match s.phase {
        Phase::Setup | Phase::Penalty | Phase::Relocate { .. } => seen.legal.choose(rng).cloned(),
        Phase::Turn => {
            let table = s.table();
            let mut left = s.hint_mask(seen.me);
            for p in seen.others() {
                let union = possible(table, &s.tokens, p)
                    .iter()
                    .fold(Mask::empty(), |m, &i| m | table.masks[i]);
                left = left & union;
            }
            let fits: Vec<Cell> = left.iter().filter(|&c| seen.may_search(c)).collect();
            if fits.len() == 1 {
                return Some(Action::Search(fits[0]));
            }
            let asks: Vec<Action> = seen
                .legal
                .iter()
                .filter(|a| matches!(a, Action::Ask { .. }))
                .cloned()
                .collect();
            asks.choose(rng)
                .cloned()
                .or_else(|| seen.legal.choose(rng).cloned())
        }
        Phase::Over => None,
    }
}

/// The opponents' hint lists: from the combinations, else from the tokens.
fn lists(seen: &Seen, k: &Combos) -> Vec<Vec<usize>> {
    let s = seen.s;
    (0..s.players)
        .map(|p| {
            if p == seen.me.0 {
                Vec::new()
            } else if k.complete && !k.per_seat[usize::from(p)].is_empty() {
                k.per_seat[usize::from(p)].clone()
            } else {
                possible(s.table(), &s.tokens, Seat(p))
            }
        })
        .collect()
}

/// [U] During the first table round the AI searches only when sure:
/// otherwise, at three, it would often win before anyone else has played.
fn first_round(s: &State) -> bool {
    let turns = s
        .log
        .iter()
        .filter(|e| matches!(e, Event::Asked { .. } | Event::Searched { .. }))
        .count();
    turns < usize::from(s.players)
}

/// [B] `check_to_search`: one candidate, or few with a likely disc from
/// the left neighbour (not in the first round); the most named of those.
fn search_choice(seen: &Seen, k: &Combos, lists: &[Vec<usize>]) -> Option<Cell> {
    let candidates: Vec<(Cell, u32)> = k
        .cells
        .iter()
        .copied()
        .filter(|&(c, _)| seen.may_search(c))
        .collect();
    if !k.complete || candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(candidates[0].0);
    }
    if candidates.len() > SEARCH_CANDIDATES || first_round(seen.s) {
        return None;
    }
    let left = &lists[seen.left().index()];
    candidates
        .iter()
        .filter(|&&(c, _)| share(seen.s, left, c) > SEARCH_DISC_CHANCE)
        .max_by_key(|&&(_, n)| n)
        .map(|&(c, _)| c)
}

/// [B] The opponent with the most possible hints (who can still be asked).
fn least_known<R: Rng + ?Sized>(seen: &Seen, lists: &[Vec<usize>], rng: &mut R) -> Option<Seat> {
    let askable: Vec<Seat> = seen
        .others()
        .filter(|&p| !seen.asks_of(p).is_empty())
        .collect();
    best_by(&askable, |p| lists[p.index()].len() as f64, rng)
}

/// [B] A space where the share of cube answers is closest to the aim;
/// spaces without any token first.
fn bot_question<R: Rng + ?Sized>(seen: &Seen, lists: &[Vec<usize>], rng: &mut R) -> Option<Action> {
    let target = least_known(seen, lists, rng)?;
    let cells = seen.asks_of(target);
    let fresh: Vec<Cell> = cells
        .iter()
        .copied()
        .filter(|c| {
            let t = &seen.s.tokens[c.index()];
            t.cube.is_none() && t.discs.is_empty()
        })
        .collect();
    let pool = if fresh.is_empty() { cells } else { fresh };
    let aim = ideal_cube_share(seen.s.players);
    let list = &lists[target.index()];
    best_by(
        &pool,
        |&c| -((1.0 - share(seen.s, list, c)) - aim).abs(),
        rng,
    )
    .map(|cell| Action::Ask { cell, target })
}

fn bot<R: Rng + ?Sized>(seen: &Seen, level: Level, rng: &mut R) -> Option<Action> {
    let s = seen.s;
    match s.phase {
        Phase::Setup | Phase::Penalty => seen.quiet_cube(rng),
        Phase::Relocate { .. } => seen.quiet_disc(rng),
        Phase::Turn => {
            // Medium reasons as [B] does; Hard also knows what the
            // generator guarantees (each hint needed, two terrain hints at
            // most).
            let inference = match level {
                Level::Hard => Inference::Puzzle,
                _ => Inference::Bot,
            };
            let k = query(s, seen.me, inference, CAP)?.combos();
            let lists = lists(seen, &k);
            if let Some(cell) = search_choice(seen, &k, &lists) {
                return Some(Action::Search(cell));
            }
            bot_question(seen, &lists, rng).or_else(|| seen.legal.choose(rng).cloned())
        }
        Phase::Over => None,
    }
}

impl Heuristics for Cryptide {
    /// The policy decides every move: no search.
    fn budget(_level: Level, _timed: bool) -> Option<Budget> {
        None
    }

    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        Self::policy_action(s, seat, Level::Easy, rng)
            .or_else(|| Self::legal_actions(s, seat).into_iter().next())
            .unwrap_or(Action::Search(Cell(0)))
    }

    /// From the tokens and `seat`'s own hint only.
    fn policy_action<R: Rng + ?Sized>(
        s: &State,
        seat: Seat,
        level: Level,
        rng: &mut R,
    ) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return None;
        }
        let seen = Seen {
            s,
            me: seat,
            legal,
            mine_known: possible(s.table(), &s.tokens, seat),
        };
        match level {
            Level::Easy => easy(&seen, rng),
            Level::Medium | Level::Hard => bot(&seen, level, rng),
        }
    }
}
