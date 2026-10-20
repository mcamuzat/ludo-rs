//! What the tokens on the map say about each seat's hint [B], read from
//! public data and the observer's own hint only.

use ludo_core::Seat;
use rand::Rng;

use crate::clues::{habitat, HintTable, MAX_TERRAIN_CLUES};
use crate::map::{Cell, Mask, CELLS};
use crate::rules::{State, Tokens};
use crate::Clue;

/// Nodes visited before an enumeration gives up. Once the setup cubes are
/// down a full walk takes 1 to 4 ms natively at five in advanced mode, the
/// AI's own cubes included (`enumeration_is_fast_at_five_advanced`); the
/// cap only binds before them, when nothing is known yet.
pub const CAP: u64 = 400_000;

/// The spaces where `seat` laid discs, and cubes.
fn marks(tokens: &[Tokens], seat: Seat) -> (Mask, Mask) {
    let mut discs = Mask::empty();
    let mut cubes = Mask::empty();
    for (i, t) in tokens.iter().enumerate() {
        let c = Cell(i as u8);
        if t.discs.contains(&seat) {
            discs.insert(c);
        }
        if t.cube == Some(seat) {
            cubes.insert(c);
        }
    }
    (discs, cubes)
}

/// [B] `update_disc` / `update_cube`: the hints (indices in `table`) that
/// agree with every token of `seat`'s.
pub fn possible(table: &HintTable, tokens: &[Tokens], seat: Seat) -> Vec<usize> {
    let (discs, cubes) = marks(tokens, seat);
    (0..table.hints.len())
        .filter(|&i| {
            let m = table.masks[i];
            (discs & m) == discs && (cubes & m).is_empty()
        })
        .collect()
}

/// What a combination of hints must satisfy to count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inference {
    /// [B] `runner.py`: distinct hints naming exactly one space.
    Bot,
    /// Everything the generator guarantees (`clues::habitat`): each hint
    /// needed, at most two "two terrains" hints too.
    Puzzle,
}

/// What the combinations of the others' possible hints say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combos {
    /// Per seat, the hints found in some combination; empty for `me`.
    pub per_seat: Vec<Vec<usize>>,
    /// Candidate habitats and how many combinations name each.
    pub cells: Vec<(Cell, u32)>,
    /// `false` when the cap stopped the enumeration.
    pub complete: bool,
}

/// An enumeration of the others' hints, from `me`'s point of view.
#[derive(Clone, Copy, Debug)]
pub struct Query<'a> {
    pub table: &'a HintTable,
    pub tokens: &'a [Tokens],
    pub me: Seat,
    pub my_hint: usize,
    pub players: u8,
    pub inference: Inference,
    /// Nodes walked at most.
    pub cap: u64,
}

struct Walk<'a, 'v> {
    q: &'a Query<'a>,
    order: Vec<(Seat, Vec<usize>)>,
    by_seat: Vec<usize>,
    budget: u64,
    visit: &'v mut dyn FnMut(&[usize], Cell),
}

impl Walk<'_, '_> {
    fn terrains(&self, i: usize) -> usize {
        usize::from(matches!(self.q.table.hints[i].clue, Clue::Terrains(..)))
    }

    fn go(&mut self, depth: usize, inter: Mask, terrain_count: usize) -> bool {
        if self.budget == 0 {
            return false;
        }
        self.budget -= 1;
        if depth == self.order.len() {
            let home = match self.q.inference {
                Inference::Bot => inter.single(),
                Inference::Puzzle => habitat(self.q.table, &self.by_seat),
            };
            if let Some(home) = home {
                (self.visit)(&self.by_seat, home);
            }
            return true;
        }
        let seat = self.order[depth].0;
        for k in 0..self.order[depth].1.len() {
            let i = self.order[depth].1[k];
            if self.by_seat.contains(&i) {
                continue;
            }
            let t = terrain_count + self.terrains(i);
            if self.q.inference == Inference::Puzzle && t > MAX_TERRAIN_CLUES {
                continue;
            }
            let next = inter & self.q.table.masks[i];
            if next.is_empty() {
                continue;
            }
            self.by_seat[seat.index()] = i;
            let ok = self.go(depth + 1, next, t);
            self.by_seat[seat.index()] = usize::MAX;
            if !ok {
                return false;
            }
        }
        true
    }
}

impl Query<'_> {
    /// Calls `visit(hints by seat, habitat)` for every combination of the
    /// others' possible hints that, with `me`'s, satisfies the inference;
    /// `false` if the cap stopped the walk.
    pub fn walk(&self, visit: &mut dyn FnMut(&[usize], Cell)) -> bool {
        let (table, tokens) = (self.table, self.tokens);
        // The habitat bears no cube: nobody's hint excludes it.
        let no_cube: Mask = (0..CELLS)
            .map(Cell)
            .filter(|c| tokens[c.index()].cube.is_none())
            .collect();
        let start = table.masks[self.my_hint] & no_cube;
        // The most constrained seats first prune best.
        let mut order: Vec<(Seat, Vec<usize>)> = (0..self.players)
            .map(Seat)
            .filter(|&s| s != self.me)
            .map(|s| {
                let list = possible(table, tokens, s)
                    .into_iter()
                    .filter(|&i| i != self.my_hint && !(start & table.masks[i]).is_empty())
                    .collect();
                (s, list)
            })
            .collect();
        order.sort_by_key(|(_, l)| l.len());
        let mut by_seat = vec![usize::MAX; usize::from(self.players)];
        by_seat[self.me.index()] = self.my_hint;
        let mut w = Walk {
            q: self,
            order,
            by_seat,
            budget: self.cap,
            visit,
        };
        let t = w.terrains(self.my_hint);
        w.go(0, start, t)
    }

    /// [B] What the combinations say: hints per seat, candidate habitats.
    pub fn combos(&self) -> Combos {
        let me = self.me.index();
        let mut per_seat = vec![Vec::new(); usize::from(self.players)];
        let mut counts = vec![0u32; usize::from(CELLS)];
        let complete = self.walk(&mut |hints, home| {
            counts[home.index()] += 1;
            for (p, &i) in hints.iter().enumerate() {
                if p != me && !per_seat[p].contains(&i) {
                    per_seat[p].push(i);
                }
            }
        });
        for list in &mut per_seat {
            list.sort_unstable();
        }
        let cells = counts
            .iter()
            .enumerate()
            .filter(|&(_, &n)| n > 0)
            .map(|(i, &n)| (Cell(i as u8), n))
            .collect();
        Combos {
            per_seat,
            cells,
            complete,
        }
    }

    /// One combination drawn uniformly among those found (reservoir
    /// sampling): hints by seat and the habitat.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<(Vec<usize>, Cell)> {
        let mut seen = 0u32;
        let mut pick = None;
        self.walk(&mut |hints, home| {
            seen += 1;
            if rng.gen_range(0..seen) == 0 {
                pick = Some((hints.to_vec(), home));
            }
        });
        pick
    }
}

/// The query `seat` can make of a game: the tokens and its own hint.
pub fn query(s: &State, seat: Seat, inference: Inference, cap: u64) -> Option<Query<'_>> {
    let table = s.table();
    let my_hint = table.index_of(s.puzzle.hints.get(seat.index())?)?;
    Some(Query {
        table,
        tokens: &s.tokens,
        me: seat,
        my_hint,
        players: s.players,
        inference,
        cap,
    })
}
