//! What a seat knows: the direct facts its sheet notes by itself, and a
//! backtracking search for the worlds consistent with everything it saw.

use ludo_core::ai::Level;
use ludo_core::Seat;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::material::{suspects, Suspect};
use crate::{all_trios, Phase, State};

/// A column of the investigation sheet [F]: a player, or Black Vienna.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Column {
    Seat(Seat),
    Trio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Mark {
    /// Holds it (or, in the Trio column, is a criminal).
    Plus,
    /// Does not.
    Minus,
}

/// One mark of the sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Note {
    pub suspect: Suspect,
    pub column: Column,
    pub mark: Mark,
}

/// Whether `seat` has seen the trio: after its own accusation (right or
/// wrong), or once the game is over.
pub fn knows_trio(s: &State, seat: Seat) -> bool {
    s.phase == Phase::Over
        || s.wrong.get(seat.index()).copied().unwrap_or(false)
        || s.accuser == Some(seat)
}

/// [U] The sheet's automatic notes, direct facts only: one's own cards (+
/// in one's column, − everywhere else), an answer of 0 (− for its three
/// suspects at the target), an answer of 3 (+ for the three), and the trio
/// once seen. Nothing else.
pub fn direct_facts(s: &State, seat: Seat) -> Vec<Note> {
    let mut notes = Vec::new();
    let columns: Vec<Column> = (0..s.players)
        .map(|p| Column::Seat(Seat(p)))
        .chain([Column::Trio])
        .collect();
    let mark_owned = |owner: Column, cards: &[Suspect], notes: &mut Vec<Note>| {
        for &suspect in cards {
            for &column in &columns {
                let mark = if column == owner {
                    Mark::Plus
                } else {
                    Mark::Minus
                };
                notes.push(Note {
                    suspect,
                    column,
                    mark,
                });
            }
        }
    };
    if let Some(hand) = s.hands.get(seat.index()) {
        mark_owned(Column::Seat(seat), hand, &mut notes);
    }
    if knows_trio(s, seat) {
        mark_owned(Column::Trio, &s.trio, &mut notes);
    }
    for a in &s.answers {
        let mark = match a.count {
            0 => Mark::Minus,
            3 => Mark::Plus,
            _ => continue,
        };
        for suspect in suspects(a.card) {
            notes.push(Note {
                suspect,
                column: Column::Seat(a.target),
                mark,
            });
        }
    }
    notes.sort();
    notes.dedup();
    notes
}

/// The hidden part of a deal, from one seat's point of view: who holds each
/// suspect it cannot place. Holders are the seats, then the trio.
#[derive(Clone)]
pub(crate) struct Problem {
    /// Suspects to place.
    cards: Vec<Suspect>,
    /// Free room of each holder (seats, then the trio).
    room: Vec<usize>,
    /// `allowed[card][holder]`.
    allowed: Vec<Vec<bool>>,
    /// (holder, cards of `cards` it concerns, how many it must hold among
    /// them, after what is already known).
    constraints: Vec<(usize, Vec<usize>, usize)>,
    /// For each card, the constraints it appears in.
    touching: Vec<Vec<usize>>,
}

impl Problem {
    /// Everything `seat` cannot place, with every answer as a constraint.
    pub(crate) fn of(s: &State, seat: Seat) -> Self {
        let n = usize::from(s.players);
        let trio_known = knows_trio(s, seat);
        let mine = &s.hands[seat.index()];
        let known = |x: &Suspect| mine.contains(x) || (trio_known && s.trio.contains(x));
        let cards: Vec<Suspect> = Suspect::all().filter(|x| !known(x)).collect();
        let mut room: Vec<usize> = s.hands.iter().map(Vec::len).collect();
        room[seat.index()] = 0;
        room.push(if trio_known { 0 } else { 3 });
        let allowed = vec![(0..=n).map(|h| room[h] > 0).collect::<Vec<bool>>(); cards.len()];
        let mut constraints = Vec::new();
        for a in &s.answers {
            let holder = a.target.index();
            if holder == seat.index() {
                continue;
            }
            let on_card = suspects(a.card);
            let idx: Vec<usize> = on_card
                .iter()
                .filter_map(|x| cards.iter().position(|c| c == x))
                .collect();
            constraints.push((holder, idx, usize::from(a.count)));
        }
        let mut touching = vec![Vec::new(); cards.len()];
        for (k, (_, idx, _)) in constraints.iter().enumerate() {
            for &i in idx {
                touching[i].push(k);
            }
        }
        Self {
            cards,
            room,
            allowed,
            constraints,
            touching,
        }
    }

    pub(crate) fn cards(&self) -> &[Suspect] {
        &self.cards
    }

    /// Forbids a placement, or forces it by forbidding every other holder.
    pub(crate) fn set(&mut self, card: Suspect, holder: usize, holds: bool) {
        if let Some(i) = self.cards.iter().position(|&c| c == card) {
            if holds {
                for (h, ok) in self.allowed[i].iter_mut().enumerate() {
                    *ok = *ok && h == holder;
                }
            } else {
                self.allowed[i][holder] = false;
            }
        }
    }

    /// One placement of every card (holder per card) satisfying the room
    /// and every constraint. With `rng`, ties are broken at random, so
    /// repeated calls sample different worlds. The search gives up after
    /// `budget` nodes.
    pub(crate) fn solve<R: Rng + ?Sized>(&self, mut rng: Option<&mut R>, budget: u32) -> Solved {
        let mut st = Search {
            assign: vec![usize::MAX; self.cards.len()],
            room: self.room.clone(),
            inside: vec![0; self.constraints.len()],
            open: self
                .constraints
                .iter()
                .map(|(_, idx, _)| idx.len())
                .collect(),
            budget,
        };
        if self
            .constraints
            .iter()
            .enumerate()
            .any(|(k, (_, _, need))| st.open[k] < *need)
        {
            return Solved::No;
        }
        match self.step(&mut st, &mut rng) {
            Some(true) => Solved::Yes(st.assign),
            Some(false) => Solved::No,
            None => Solved::GaveUp,
        }
    }

    /// Whether `card` may go to `h` now, given the constraints it touches.
    fn fits(&self, st: &Search, card: usize, h: usize) -> bool {
        st.room[h] > 0
            && self.allowed[card][h]
            && self.touching[card].iter().all(|&k| {
                let (holder, _, need) = &self.constraints[k];
                let inside = st.inside[k] + usize::from(*holder == h);
                inside <= *need && inside + st.open[k] > *need
            })
    }

    fn place(&self, st: &mut Search, card: usize, h: usize, delta: isize) {
        for &k in &self.touching[card] {
            let held = usize::from(self.constraints[k].0 == h);
            if delta > 0 {
                st.open[k] -= 1;
                st.inside[k] += held;
            } else {
                st.open[k] += 1;
                st.inside[k] -= held;
            }
        }
        if delta > 0 {
            st.room[h] -= 1;
            st.assign[card] = h;
        } else {
            st.room[h] += 1;
            st.assign[card] = usize::MAX;
        }
    }

    /// `Some(true)`: solved; `Some(false)`: no solution below; `None`: out
    /// of budget. The next card is the one with the fewest holders left.
    fn step<R: Rng + ?Sized>(&self, st: &mut Search, rng: &mut Option<&mut R>) -> Option<bool> {
        if st.budget == 0 {
            return None;
        }
        st.budget -= 1;
        let mut best: Option<(usize, Vec<usize>)> = None;
        // How many unplaced cards could still go to each holder: every hand
        // must be filled exactly.
        let mut reachable = vec![0usize; st.room.len()];
        for card in (0..self.cards.len()).filter(|&c| st.assign[c] == usize::MAX) {
            let holders: Vec<usize> = (0..st.room.len())
                .filter(|&h| self.fits(st, card, h))
                .collect();
            if holders.is_empty() {
                return Some(false);
            }
            for &h in &holders {
                reachable[h] += 1;
            }
            if best.as_ref().is_none_or(|(_, b)| holders.len() < b.len()) {
                best = Some((card, holders));
            }
        }
        if reachable.iter().zip(&st.room).any(|(&r, &room)| r < room) {
            return Some(false);
        }
        let Some((card, mut holders)) = best else {
            return Some(true);
        };
        if let Some(r) = rng.as_deref_mut() {
            holders.shuffle(r);
        }
        let mut gave_up = false;
        for h in holders {
            self.place(st, card, h, 1);
            match self.step(st, rng) {
                Some(true) => return Some(true),
                Some(false) => {}
                None => gave_up = true,
            }
            self.place(st, card, h, -1);
            if gave_up {
                return None;
            }
        }
        Some(false)
    }
}

impl Problem {
    /// A placement that only respects the room of each holder.
    fn fill<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<usize> {
        let mut room = self.room.clone();
        let mut assign = vec![usize::MAX; self.cards.len()];
        let mut order: Vec<usize> = (0..self.cards.len()).collect();
        order.shuffle(rng);
        for card in order {
            let open: Vec<usize> = (0..room.len()).filter(|&h| room[h] > 0).collect();
            if let Some(&h) = open.choose(rng) {
                room[h] -= 1;
                assign[card] = h;
            }
        }
        assign
    }
}

/// What a search found.
pub(crate) enum Solved {
    Yes(Vec<usize>),
    No,
    GaveUp,
}

/// The mutable part of a search.
struct Search {
    assign: Vec<usize>,
    room: Vec<usize>,
    inside: Vec<usize>,
    open: Vec<usize>,
    budget: u32,
}

/// Nodes a search may visit.
const BUDGET: u32 = 20_000;

/// A world `observer` cannot tell from the real one: every hand (its own
/// unchanged) and the trio, consistent with the hand sizes and every
/// answer (only the hand sizes if the search keeps giving up).
pub fn sample_world<R: Rng + ?Sized>(
    s: &State,
    observer: Seat,
    rng: &mut R,
) -> (Vec<Vec<Suspect>>, [Suspect; 3]) {
    let problem = Problem::of(s, observer);
    let mut found = None;
    for _ in 0..4 {
        if let Solved::Yes(assign) = problem.solve(Some(&mut *rng), BUDGET) {
            found = Some(assign);
            break;
        }
    }
    // Never the real world: out of luck, a world that only fits the hand
    // sizes (the answers are then off, but nothing leaks).
    let assign = found.unwrap_or_else(|| problem.fill(rng));
    let n = usize::from(s.players);
    let mut hands: Vec<Vec<Suspect>> = (0..n)
        .map(|i| {
            if i == observer.index() {
                s.hands[i].clone()
            } else {
                Vec::new()
            }
        })
        .collect();
    let mut trio: Vec<Suspect> = if knows_trio(s, observer) {
        s.trio.to_vec()
    } else {
        Vec::new()
    };
    for (&card, &holder) in problem.cards().iter().zip(&assign) {
        if holder == n {
            trio.push(card);
        } else {
            hands[holder].push(card);
        }
    }
    for hand in &mut hands {
        hand.sort();
    }
    trio.sort();
    (hands, [trio[0], trio[1], trio[2]])
}

/// What a seat holds for certain, cell by cell: `Some(true)` "+",
/// `Some(false)` "−", `None` unknown. One row per column (the seats, then
/// the trio), one cell per suspect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Knowledge {
    cells: Vec<[Option<bool>; 27]>,
}

impl Knowledge {
    fn row(&self, column: Column) -> usize {
        match column {
            Column::Seat(seat) => seat.index(),
            Column::Trio => self.cells.len() - 1,
        }
    }

    pub fn get(&self, column: Column, suspect: Suspect) -> Option<bool> {
        self.cells[self.row(column)][suspect.index()]
    }

    /// `true` when the cell changed.
    fn put(&mut self, row: usize, suspect: usize, value: bool) -> bool {
        let cell = &mut self.cells[row][suspect];
        if cell.is_some() {
            return false;
        }
        *cell = Some(value);
        true
    }
}

/// [U] The deductions of each level, from what `seat` saw: Easy keeps the
/// direct facts; Medium adds a good player's simple rules until nothing
/// moves; Hard is Medium's sheet (the full search lives in
/// `possible_trios`).
pub fn knowledge(s: &State, seat: Seat, level: Level) -> Knowledge {
    let n = usize::from(s.players);
    let mut k = Knowledge {
        cells: vec![[None; 27]; n + 1],
    };
    for note in direct_facts(s, seat) {
        let row = k.row(note.column);
        k.put(row, note.suspect.index(), note.mark == Mark::Plus);
    }
    if level != Level::Easy {
        simple_rules(&mut k, s);
    }
    k
}

/// Medium's rules, to a fixed point: a suspect has one column; an answer
/// whose count is met (or can only be met) settles its unknowns; a column
/// whose size is met (or can only be met) settles its unknowns.
fn simple_rules(k: &mut Knowledge, s: &State) {
    let rows = k.cells.len();
    let sizes: Vec<usize> = s.hands.iter().map(Vec::len).chain([3]).collect();
    loop {
        let mut changed = false;
        // One column per suspect.
        for x in 0..Suspect::COUNT {
            let column: Vec<Option<bool>> = (0..rows).map(|r| k.cells[r][x]).collect();
            if let Some(owner) = column.iter().position(|c| *c == Some(true)) {
                for r in (0..rows).filter(|&r| r != owner) {
                    changed |= k.put(r, x, false);
                }
            } else if column.iter().filter(|c| **c != Some(false)).count() == 1 {
                if let Some(r) = column.iter().position(|c| c.is_none()) {
                    changed |= k.put(r, x, true);
                }
            }
        }
        // Answers.
        for a in &s.answers {
            let r = a.target.index();
            let on_card = suspects(a.card);
            let plus = on_card
                .iter()
                .filter(|x| k.cells[r][x.index()] == Some(true))
                .count();
            let unknown: Vec<usize> = on_card
                .iter()
                .map(|x| x.index())
                .filter(|&x| k.cells[r][x].is_none())
                .collect();
            let count = usize::from(a.count);
            if plus == count {
                for &x in &unknown {
                    changed |= k.put(r, x, false);
                }
            } else if plus + unknown.len() == count {
                for &x in &unknown {
                    changed |= k.put(r, x, true);
                }
            }
        }
        // Column sizes.
        for (r, &size) in sizes.iter().enumerate() {
            let plus = k.cells[r].iter().filter(|c| **c == Some(true)).count();
            let unknown: Vec<usize> = (0..Suspect::COUNT)
                .filter(|&x| k.cells[r][x].is_none())
                .collect();
            if plus == size {
                for &x in &unknown {
                    changed |= k.put(r, x, false);
                }
            } else if plus + unknown.len() == size {
                for &x in &unknown {
                    changed |= k.put(r, x, true);
                }
            }
        }
        if !changed {
            return;
        }
    }
}

/// The trios `seat` cannot rule out at `level`, by its sheet; at Hard, a
/// suspect also stays a possible criminal unless no whole deal fits every
/// answer with it in the trio (a search that gives up keeps it). Never
/// rules out the true trio; exact in practice when three suspects are left.
pub fn possible_trios(s: &State, seat: Seat, level: Level) -> Vec<[Suspect; 3]> {
    let sheet = knowledge(
        s,
        seat,
        if level == Level::Hard {
            Level::Medium
        } else {
            level
        },
    );
    let seats = usize::from(s.players);
    let mut criminal: Vec<bool> = Suspect::all()
        .map(|x| {
            sheet.get(Column::Trio, x) != Some(false)
                && (0..seats).all(|r| sheet.cells[r][x.index()] != Some(true))
        })
        .collect();
    if level == Level::Hard {
        criminal = feasible_criminals(s, seat, &sheet, &criminal);
    }
    let sure: Vec<Suspect> = Suspect::all()
        .filter(|&x| sheet.get(Column::Trio, x) == Some(true))
        .collect();
    all_trios()
        .filter(|t| t.iter().all(|x| criminal[x.index()]))
        .filter(|t| sure.iter().all(|x| t.contains(x)))
        .collect()
}

/// Among the `candidates`, the suspects some whole deal puts in the trio:
/// one search per suspect at most, each deal found clearing its three.
fn feasible_criminals(s: &State, seat: Seat, sheet: &Knowledge, candidates: &[bool]) -> Vec<bool> {
    let seats = usize::from(s.players);
    let mut base = Problem::of(s, seat);
    for x in Suspect::all() {
        for r in 0..=seats {
            if let Some(v) = sheet.cells[r][x.index()] {
                base.set(x, r, v);
            }
        }
    }
    let mut known_trio: Vec<Suspect> = if knows_trio(s, seat) {
        s.trio.to_vec()
    } else {
        Vec::new()
    };
    known_trio.retain(|x| candidates[x.index()]);
    let mut feasible = vec![false; Suspect::COUNT];
    for x in known_trio {
        feasible[x.index()] = true;
    }
    for x in Suspect::all() {
        if !candidates[x.index()] || feasible[x.index()] {
            continue;
        }
        let mut p = base.clone();
        p.set(x, seats, true);
        match p.solve::<rand::rngs::SmallRng>(None, BUDGET) {
            Solved::Yes(assign) => {
                for (&card, &holder) in p.cards().iter().zip(&assign) {
                    if holder == seats {
                        feasible[card.index()] = true;
                    }
                }
            }
            // Unsure: keep it, so the true trio is never ruled out.
            Solved::GaveUp => feasible[x.index()] = true,
            Solved::No => {}
        }
    }
    feasible
}
