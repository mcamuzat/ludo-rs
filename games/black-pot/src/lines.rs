//! [R] Lines of five pawns and what they pay.

use serde::{Deserialize, Serialize};

use crate::board::{cell_at, cells, CellId, Dir, Kind, DIRS};

/// [R] What five aligned cells pay.
pub fn points(line: &[CellId; 5]) -> u32 {
    let count = |kind: Kind| {
        line.iter()
            .filter(|&&c| cells()[usize::from(c)].kind == kind)
            .count()
    };
    match count(Kind::BlackPot) {
        4.. => 400,
        3 => 250,
        2 => 100,
        _ if count(Kind::Cherry) == 5 => 100,
        _ if count(Kind::Lemon) == 5 => 50,
        _ => 30,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alignment {
    pub team: u8,
    pub dir: Dir,
    pub cells: [CellId; 5],
    pub points: u32,
}

/// The cells of each full line of the board in direction `dir`, in order.
fn lines(dir: Dir) -> Vec<Vec<CellId>> {
    let (dr, dc) = dir.step();
    cells()
        .iter()
        .filter(|c| cell_at(c.row as i8 - dr, c.col as i8 - dc).is_none())
        .map(|c| {
            let (r, col) = (c.row as i8, c.col as i8);
            (0..)
                .map_while(|k| cell_at(r + dr * k, col + dc * k))
                .collect()
        })
        .collect()
}

/// The non-overlapping windows of `run` that pay the most in total; on a
/// tie, the windows taken earliest in the run.
fn best_windows(run: &[CellId]) -> Vec<[CellId; 5]> {
    let n = run.len();
    let window = |i: usize| -> [CellId; 5] { run[i..i + 5].try_into().expect("five cells") };
    // best[i]: the most the cells from i onwards pay.
    let mut best = vec![0u32; n + 1];
    for i in (0..n).rev() {
        let skip = best[i + 1];
        let take = if i + 5 <= n {
            points(&window(i)) + best[i + 5]
        } else {
            0
        };
        best[i] = skip.max(take);
    }
    let mut chosen = Vec::new();
    let mut i = 0;
    while i + 5 <= n {
        if points(&window(i)) + best[i + 5] >= best[i + 1] {
            chosen.push(window(i));
            i += 5;
        } else {
            i += 1;
        }
    }
    chosen
}

/// The lines of five `team` has just completed. `owner[cell]` is the team
/// of the pawn on it; `counted[cell][dir]` tells that pawn already counts
/// in a line of that direction. [U] A pawn counts once per direction, so a
/// run of pawns not yet counted in a direction gives its best windows
/// there (❓ a run longer than five keeps the windows paying the most).
pub fn new_alignments(owner: &[Option<u8>], counted: &[[bool; 4]], team: u8) -> Vec<Alignment> {
    let mut found = Vec::new();
    for dir in DIRS {
        for line in lines(dir) {
            let free = |c: &CellId| {
                owner[usize::from(*c)] == Some(team) && !counted[usize::from(*c)][dir.index()]
            };
            for run in line.split(|c| !free(c)).filter(|run| run.len() >= 5) {
                found.extend(best_windows(run).into_iter().map(|cells| Alignment {
                    team,
                    dir,
                    cells,
                    points: points(&cells),
                }));
            }
        }
    }
    found.sort_by_key(|a| (a.dir.index(), a.cells[0]));
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::windows;

    fn empty() -> (Vec<Option<u8>>, Vec<[bool; 4]>) {
        (vec![None; cells().len()], vec![[false; 4]; cells().len()])
    }

    fn kinds(w: &[CellId; 5], kind: Kind) -> usize {
        w.iter()
            .filter(|&&c| cells()[usize::from(c)].kind == kind)
            .count()
    }

    /// Cells of row 5 (the middle row), by column.
    fn middle(col: usize) -> CellId {
        cell_at(5, col as i8).unwrap()
    }

    #[test]
    fn each_line_of_the_scale_pays() {
        let find = |pred: &dyn Fn(&[CellId; 5]) -> bool| {
            windows()
                .iter()
                .find(|w| pred(&w.cells))
                .expect("such a window")
                .cells
        };
        let bp = |n| move |w: &[CellId; 5]| kinds(w, Kind::BlackPot) == n;
        assert_eq!(points(&find(&bp(4))), 400);
        assert_eq!(points(&find(&bp(3))), 250);
        assert_eq!(points(&find(&bp(2))), 100);
        assert_eq!(points(&find(&|w| kinds(w, Kind::Cherry) == 5)), 100);
        assert_eq!(points(&find(&|w| kinds(w, Kind::Lemon) == 5)), 50);
        assert_eq!(points(&find(&bp(1))), 30);
        assert_eq!(
            points(&find(&|w| kinds(w, Kind::BlackPot) == 0
                && kinds(w, Kind::Lemon) < 5
                && kinds(w, Kind::Cherry) < 5)),
            30
        );
    }

    #[test]
    fn five_in_a_row_make_a_line_four_do_not() {
        let (mut owner, counted) = empty();
        for col in 0..4 {
            owner[usize::from(middle(col))] = Some(0);
        }
        assert!(new_alignments(&owner, &counted, 0).is_empty());
        owner[usize::from(middle(4))] = Some(0);
        let found = new_alignments(&owner, &counted, 0);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].dir, Dir::Row);
        assert_eq!(found[0].cells, [0, 1, 2, 3, 4].map(middle));
        assert_eq!(found[0].points, 30);
        assert!(new_alignments(&owner, &counted, 1).is_empty());
    }

    #[test]
    fn a_run_of_six_gives_its_best_line_only() {
        // Row 5: 1 2 10 22 24 23 — columns 1 to 5 hold two Black-Pots.
        let (mut owner, counted) = empty();
        for col in 0..6 {
            owner[usize::from(middle(col))] = Some(0);
        }
        let found = new_alignments(&owner, &counted, 0);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].cells, [1, 2, 3, 4, 5].map(middle));
        assert_eq!(found[0].points, 100);
    }

    #[test]
    fn a_run_of_ten_gives_two_lines_when_they_pay_more() {
        // Row 2: 19 11 13 10 3 | 20 4 9 14 12 pay 100 + 30; no single
        // window of these ten pays more than 100.
        let row2 = |col: usize| cell_at(2, col as i8).unwrap();
        let (mut owner, counted) = empty();
        for col in 0..10 {
            owner[usize::from(row2(col))] = Some(0);
        }
        let found = new_alignments(&owner, &counted, 0);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].cells, [0, 1, 2, 3, 4].map(row2));
        assert_eq!(found[1].cells, [5, 6, 7, 8, 9].map(row2));
        assert_eq!(found[0].points + found[1].points, 130);
    }

    #[test]
    fn a_run_of_ten_keeps_one_line_when_it_pays_more() {
        // Row 5: 1 2 10 22 24 23 24 22 10 2 — columns 3 to 7 hold three
        // Black-Pots (250), more than the two halves together (30 + 100).
        let (mut owner, counted) = empty();
        for col in 0..10 {
            owner[usize::from(middle(col))] = Some(0);
        }
        let found = new_alignments(&owner, &counted, 0);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].cells, [3, 4, 5, 6, 7].map(middle));
        assert_eq!(found[0].points, 250);
    }

    #[test]
    fn a_counted_pawn_crosses_into_another_direction_only() {
        let (mut owner, mut counted) = empty();
        let centre = middle(5);
        for col in 0..5 {
            owner[usize::from(middle(col))] = Some(0);
        }
        owner[usize::from(centre)] = Some(0);
        counted[usize::from(centre)][Dir::Row.index()] = true;
        for row in [3, 4, 6, 7] {
            owner[usize::from(cell_at(row, 5).unwrap())] = Some(0);
        }
        let found = new_alignments(&owner, &counted, 0);
        // The column through the centre counts; the row through it does
        // not reuse it, but columns 0 to 4 of the row are a line of their own.
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].dir, Dir::Row);
        assert_eq!(found[0].cells, [0, 1, 2, 3, 4].map(middle));
        assert_eq!(found[1].dir, Dir::Column);
        assert!(found[1].cells.contains(&centre));
        // Counted in the row, the five row cells give nothing more.
        for col in 0..5 {
            counted[usize::from(middle(col))][Dir::Row.index()] = true;
        }
        let found = new_alignments(&owner, &counted, 0);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].dir, Dir::Column);
    }

    #[test]
    fn an_opposing_pawn_cuts_a_run() {
        let (mut owner, counted) = empty();
        for col in 0..6 {
            owner[usize::from(middle(col))] = Some(0);
        }
        owner[usize::from(middle(2))] = Some(1);
        assert!(new_alignments(&owner, &counted, 0).is_empty());
    }
}
