//! What a seat knows: the direct facts its sheet notes by itself, and a
//! backtracking search for the worlds consistent with everything it saw.

use ludo_core::Seat;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::material::{suspects, Suspect};
use crate::{Phase, State};

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

/// Whether `seat` has seen the trio: after its own wrong accusation, or
/// once the game is over.
pub fn knows_trio(s: &State, seat: Seat) -> bool {
    s.phase == Phase::Over || s.wrong.get(seat.index()).copied().unwrap_or(false)
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

    /// One placement of every card (holder per card) satisfying the room
    /// and every constraint, or `None`. With `rng`, holders are tried in
    /// random order, so repeated calls sample different worlds.
    pub(crate) fn solve<R: Rng + ?Sized>(&self, mut rng: Option<&mut R>) -> Option<Vec<usize>> {
        // Most constrained cards first.
        let mut order: Vec<usize> = (0..self.cards.len()).collect();
        if let Some(r) = rng.as_deref_mut() {
            order.shuffle(r);
        }
        order.sort_by_key(|&i| std::cmp::Reverse(self.touching[i].len()));
        let mut assign = vec![usize::MAX; self.cards.len()];
        let mut room = self.room.clone();
        // Per constraint: placed inside so far, and still unplaced.
        let mut inside: Vec<usize> = vec![0; self.constraints.len()];
        let mut open: Vec<usize> = self
            .constraints
            .iter()
            .map(|(_, idx, _)| idx.len())
            .collect();
        if self
            .constraints
            .iter()
            .enumerate()
            .any(|(k, (_, _, need))| open[k] < *need)
        {
            return None;
        }
        let mut budget: u32 = 2_000_000;
        if self.step(
            0,
            &order,
            &mut assign,
            &mut room,
            &mut inside,
            &mut open,
            &mut rng,
            &mut budget,
        ) {
            Some(assign)
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step<R: Rng + ?Sized>(
        &self,
        depth: usize,
        order: &[usize],
        assign: &mut Vec<usize>,
        room: &mut Vec<usize>,
        inside: &mut Vec<usize>,
        open: &mut Vec<usize>,
        rng: &mut Option<&mut R>,
        budget: &mut u32,
    ) -> bool {
        let Some(&card) = order.get(depth) else {
            return true;
        };
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        let mut holders: Vec<usize> = (0..room.len())
            .filter(|&h| room[h] > 0 && self.allowed[card][h])
            .collect();
        if let Some(r) = rng.as_deref_mut() {
            holders.shuffle(r);
        }
        for h in holders {
            // Place `card` at `h` and check the constraints it touches.
            let mut ok = true;
            for &k in &self.touching[card] {
                let (holder, _, need) = &self.constraints[k];
                open[k] -= 1;
                if *holder == h {
                    inside[k] += 1;
                }
                if inside[k] > *need || inside[k] + open[k] < *need {
                    ok = false;
                }
            }
            if ok {
                room[h] -= 1;
                assign[card] = h;
                if self.step(depth + 1, order, assign, room, inside, open, rng, budget) {
                    return true;
                }
                room[h] += 1;
                assign[card] = usize::MAX;
            }
            for &k in &self.touching[card] {
                open[k] += 1;
                if self.constraints[k].0 == h {
                    inside[k] -= 1;
                }
            }
        }
        false
    }
}

/// A world `observer` cannot tell from the real one: every hand (its own
/// unchanged) and the trio, consistent with the hand sizes and every
/// answer. Falls back to the real world if the search gives up.
pub fn sample_world<R: Rng + ?Sized>(
    s: &State,
    observer: Seat,
    rng: &mut R,
) -> (Vec<Vec<Suspect>>, [Suspect; 3]) {
    let problem = Problem::of(s, observer);
    let Some(assign) = problem.solve(Some(rng)) else {
        return (s.hands.clone(), s.trio);
    };
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
