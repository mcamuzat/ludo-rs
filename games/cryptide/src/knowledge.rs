//! What the tokens on the map say about each seat's hint [B], read from
//! public data and the observer's own hint only.

use ludo_core::Seat;
use rand::Rng;

use crate::clues::{habitat, HintTable, MAX_TERRAIN_CLUES};
use crate::map::{Cell, Mask, CELLS};
use crate::rules::Tokens;
use crate::Clue;

/// Nodes visited before an enumeration gives up. Once the setup cubes are
/// down a full walk takes about 1 ms natively even at five in advanced mode
/// (each cube rules out about half of a seat's hints); the cap only binds
/// before them, when nothing is known yet.
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

/// Walks every puzzle the others' possible hints could form with `me`'s
/// hint, calling `visit(hints by seat, habitat)`; `false` if `cap` nodes
/// were not enough.
fn walk(
    table: &HintTable,
    tokens: &[Tokens],
    me: Seat,
    my_hint: usize,
    players: u8,
    cap: u64,
    visit: &mut dyn FnMut(&[usize], Cell),
) -> bool {
    // The habitat bears no cube: nobody's hint excludes it.
    let no_cube: Mask = (0..CELLS)
        .map(Cell)
        .filter(|c| tokens[c.index()].cube.is_none())
        .collect();
    let start = table.masks[my_hint] & no_cube;
    // The most constrained seats first prune best.
    let mut order: Vec<(Seat, Vec<usize>)> = (0..players)
        .map(Seat)
        .filter(|&s| s != me)
        .map(|s| {
            let list = possible(table, tokens, s)
                .into_iter()
                .filter(|&i| i != my_hint && !(start & table.masks[i]).is_empty())
                .collect();
            (s, list)
        })
        .collect();
    order.sort_by_key(|(_, l): &(Seat, Vec<usize>)| l.len());
    let mut by_seat = vec![usize::MAX; usize::from(players)];
    by_seat[me.index()] = my_hint;
    let terrains = |i: usize| matches!(table.hints[i].clue, Clue::Terrains(..));
    let mut budget = cap;

    #[allow(clippy::too_many_arguments)]
    fn go(
        depth: usize,
        inter: Mask,
        terrain_count: usize,
        order: &[(Seat, Vec<usize>)],
        by_seat: &mut Vec<usize>,
        table: &HintTable,
        terrains: &dyn Fn(usize) -> bool,
        budget: &mut u64,
        visit: &mut dyn FnMut(&[usize], Cell),
    ) -> bool {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        let Some((seat, list)) = order.get(depth) else {
            if let Some(home) = habitat(table, by_seat) {
                visit(by_seat, home);
            }
            return true;
        };
        for &i in list {
            if by_seat.contains(&i) {
                continue;
            }
            let t = terrain_count + usize::from(terrains(i));
            if t > MAX_TERRAIN_CLUES {
                continue;
            }
            let next = inter & table.masks[i];
            if next.is_empty() {
                continue;
            }
            by_seat[seat.index()] = i;
            let ok = go(
                depth + 1,
                next,
                t,
                order,
                by_seat,
                table,
                terrains,
                budget,
                visit,
            );
            by_seat[seat.index()] = usize::MAX;
            if !ok {
                return false;
            }
        }
        true
    }

    go(
        0,
        start,
        usize::from(terrains(my_hint)),
        &order,
        &mut by_seat,
        table,
        &terrains,
        &mut budget,
        visit,
    )
}

/// [B] The combinations of the others' possible hints that, with `me`'s,
/// name exactly one space as a puzzle would.
pub fn combos(
    table: &HintTable,
    tokens: &[Tokens],
    me: Seat,
    my_hint: usize,
    players: u8,
    cap: u64,
) -> Combos {
    let mut per_seat = vec![Vec::new(); usize::from(players)];
    let mut counts = vec![0u32; usize::from(CELLS)];
    let complete = walk(
        table,
        tokens,
        me,
        my_hint,
        players,
        cap,
        &mut |hints, home| {
            counts[home.index()] += 1;
            for (p, &i) in hints.iter().enumerate() {
                if p != me.index() && !per_seat[p].contains(&i) {
                    per_seat[p].push(i);
                }
            }
        },
    );
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

/// One combination drawn uniformly among those found within `cap` nodes
/// (reservoir sampling): hints by seat and the habitat.
pub fn sample<R: Rng + ?Sized>(
    table: &HintTable,
    tokens: &[Tokens],
    me: Seat,
    my_hint: usize,
    players: u8,
    cap: u64,
    rng: &mut R,
) -> Option<(Vec<usize>, Cell)> {
    let mut seen = 0u32;
    let mut pick = None;
    walk(
        table,
        tokens,
        me,
        my_hint,
        players,
        cap,
        &mut |hints, home| {
            seen += 1;
            if rng.gen_range(0..seen) == 0 {
                pick = Some((hints.to_vec(), home));
            }
        },
    );
    pick
}
