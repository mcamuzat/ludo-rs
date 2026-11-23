//! The table: a grid without edges where dominoes are laid, and the laying
//! rules [R] (port of `evaluatePlacement` in the JS version).

use std::collections::{BTreeSet, HashMap};

use ludo_core::Illegal;
use serde::{Deserialize, Serialize};

use crate::deck::{is_double, values, DOMINOES};

/// Where the second half lies from the anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Dir {
    Right,
    Down,
}

/// A domino laid with its anchor at `(row, col)`. The anchor holds `a` (or
/// `b` if `flipped`), the other cell the other value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Placement {
    pub domino: u8,
    pub row: i8,
    pub col: i8,
    pub dir: Dir,
    pub flipped: bool,
}

impl Placement {
    /// The two cells, anchor first, `None` off the coordinate range.
    pub fn cells(&self) -> Option<[(i8, i8); 2]> {
        let other = match self.dir {
            Dir::Right => (self.row, self.col.checked_add(1)?),
            Dir::Down => (self.row.checked_add(1)?, self.col),
        };
        Some([(self.row, self.col), other])
    }

    /// The values on the anchor and on the other cell.
    pub fn sides(&self) -> (u8, u8) {
        let (a, b) = values(self.domino);
        if self.flipped {
            (b, a)
        } else {
            (a, b)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placed {
    pub at: Placement,
    /// `None` for the starting square.
    pub by: Option<u8>,
}

/// The laid dominoes, with an index of the cells rebuilt on load.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Vec<Placed>", into = "Vec<Placed>")]
pub struct Board {
    placed: Vec<Placed>,
    /// Cell → (value, index in `placed`).
    cells: HashMap<(i8, i8), (u8, usize)>,
}

impl From<Vec<Placed>> for Board {
    fn from(placed: Vec<Placed>) -> Self {
        let mut board = Board {
            placed: Vec::new(),
            cells: HashMap::new(),
        };
        for p in placed {
            board.lay(p);
        }
        board
    }
}

impl From<Board> for Vec<Placed> {
    fn from(board: Board) -> Self {
        board.placed
    }
}

/// [R] A group is a prefix of the repetition of p distinct values, p = 1 to 5.
pub fn valid_cycle(values: &[u8]) -> bool {
    let n = values.len();
    if n <= 1 {
        return true;
    }
    (1..=5).any(|p| {
        let unit = &values[..p.min(n)];
        let distinct = unit.iter().enumerate().all(|(i, v)| !unit[..i].contains(v));
        distinct && (p..n).all(|i| values[i] == values[i % p])
    })
}

impl Board {
    /// [R] The two starters side by side form a square: `starters[0]` on
    /// top, both horizontal, at (0,0).
    pub fn new(starters: [u8; 2]) -> Board {
        let mut board = Board::from(Vec::new());
        for (row, domino) in [0, 1].into_iter().zip(starters) {
            board.lay(Placed {
                at: Placement {
                    domino,
                    row,
                    col: 0,
                    dir: Dir::Right,
                    flipped: false,
                },
                by: None,
            });
        }
        board
    }

    pub fn placed(&self) -> &[Placed] {
        &self.placed
    }

    pub fn value_at(&self, row: i8, col: i8) -> Option<u8> {
        self.cells.get(&(row, col)).map(|&(v, _)| v)
    }

    /// Lays `p` for seat `by` without checking it.
    pub fn place(&mut self, p: Placement, by: u8) {
        self.lay(Placed {
            at: p,
            by: Some(by),
        });
    }

    fn lay(&mut self, p: Placed) {
        let Some(cells) = p.at.cells() else { return };
        let (first, second) = p.at.sides();
        let index = self.placed.len();
        self.cells.insert(cells[0], (first, index));
        self.cells.insert(cells[1], (second, index));
        self.placed.push(p);
    }

    /// The points `p` scores, or why it is refused.
    pub fn evaluate(&self, p: &Placement) -> Result<u32, Illegal> {
        if usize::from(p.domino) >= DOMINOES {
            return Err(Illegal::new("Ce domino n'existe pas."));
        }
        let Some(cells) = p.cells() else {
            return Err(Illegal::new("Ces cases sont hors du plateau."));
        };
        if cells.iter().any(|c| self.cells.contains_key(c)) {
            return Err(Illegal::new("Ces cases ne sont pas libres."));
        }
        let frame = self.frame(cells, p.dir);
        let (first, second) = p.sides();
        let mut buf = Vec::new();
        if !frame.cycles_hold(first, second, &mut buf) {
            return Err(Illegal::new("Ce placement casse un cycle de couleurs."));
        }
        let Some(product) = frame.product() else {
            return Err(Illegal::new(
                "Ce domino doit prolonger au moins deux groupes.",
            ));
        };
        if self.fourth_in_line(p) {
            return Err(Illegal::new(
                "Quatre dominos alignés par leur grand côté : interdit.",
            ));
        }
        Ok(if is_double(p.domino) {
            product * 2
        } else {
            product
        })
    }

    /// The values beside the free cells `cells` of a domino along `dir`.
    fn frame(&self, cells: [(i8, i8); 2], dir: Dir) -> Frame {
        let (own, across) = match dir {
            Dir::Right => ((0, 1), (1, 0)),
            Dir::Down => ((1, 0), (0, 1)),
        };
        Frame {
            own: (
                self.run(cells[0], (-own.0, -own.1)),
                self.run(cells[1], own),
            ),
            across: cells.map(|c| (self.run(c, (-across.0, -across.1)), self.run(c, across))),
        }
    }

    /// The values met from `from` (excluded) along `step` until an empty
    /// cell, nearest last when walking backwards (that is, in line order).
    fn run(&self, from: (i8, i8), step: (i8, i8)) -> Vec<u8> {
        let mut out = Vec::new();
        let mut cell = from;
        while let (Some(r), Some(c)) = (cell.0.checked_add(step.0), cell.1.checked_add(step.1)) {
            match self.value_at(r, c) {
                Some(v) => out.push(v),
                None => break,
            }
            cell = (r, c);
        }
        if step.0 < 0 || step.1 < 0 {
            out.reverse();
        }
        out
    }

    /// [R] Rule 2: `p` would be the fourth domino of a run of dominoes of
    /// its orientation, exactly aligned and touching by their long side.
    fn fourth_in_line(&self, p: &Placement) -> bool {
        let step = match p.dir {
            Dir::Right => (1, 0),
            Dir::Down => (0, 1),
        };
        let mut run = 1;
        for sign in [-1i8, 1] {
            let (mut row, mut col) = (p.row, p.col);
            while let (Some(r), Some(c)) = (
                row.checked_add(sign * step.0),
                col.checked_add(sign * step.1),
            ) {
                let aligned = self.cells.get(&(r, c)).is_some_and(|&(_, i)| {
                    let q = self.placed[i].at;
                    q.dir == p.dir && (q.row, q.col) == (r, c)
                });
                if !aligned {
                    break;
                }
                run += 1;
                (row, col) = (r, c);
            }
        }
        run >= 4
    }

    /// Every legal placement of the given dominoes with its points, ordered
    /// by domino, then anchor, direction and flip. A double is never flipped.
    pub fn moves(&self, dominoes: &[u8]) -> Vec<(Placement, u32)> {
        let mut spots = BTreeSet::new();
        for &(row, col) in self.cells.keys() {
            for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let (Some(r), Some(c)) = (row.checked_add(dr), col.checked_add(dc)) else {
                    continue;
                };
                if self.cells.contains_key(&(r, c)) {
                    continue;
                }
                spots.insert((r, c, Dir::Right));
                spots.insert((r, c, Dir::Down));
                if let Some(left) = c.checked_sub(1) {
                    spots.insert((r, left, Dir::Right));
                }
                if let Some(up) = r.checked_sub(1) {
                    spots.insert((up, c, Dir::Down));
                }
            }
        }
        // What does not depend on the values is settled once per spot.
        let frames: Vec<(i8, i8, Dir, Frame, u32)> = spots
            .into_iter()
            .filter_map(|(row, col, dir)| {
                let p = Placement {
                    domino: 0,
                    row,
                    col,
                    dir,
                    flipped: false,
                };
                let cells = p.cells()?;
                if cells.iter().any(|c| self.cells.contains_key(c)) {
                    return None;
                }
                let frame = self.frame(cells, dir);
                let product = frame.product()?;
                if self.fourth_in_line(&p) {
                    return None;
                }
                Some((row, col, dir, frame, product))
            })
            .collect();
        let mut moves = Vec::new();
        let mut buf = Vec::new();
        for &domino in dominoes {
            if usize::from(domino) >= DOMINOES {
                continue;
            }
            let double = is_double(domino);
            for (row, col, dir, frame, product) in &frames {
                for flipped in [false, true] {
                    if flipped && double {
                        continue;
                    }
                    let p = Placement {
                        domino,
                        row: *row,
                        col: *col,
                        dir: *dir,
                        flipped,
                    };
                    let (first, second) = p.sides();
                    if frame.cycles_hold(first, second, &mut buf) {
                        moves.push((p, if double { product * 2 } else { *product }));
                    }
                }
            }
        }
        moves
    }
}

/// The values beside a free footprint, on its own line and across each cell.
struct Frame {
    own: (Vec<u8>, Vec<u8>),
    across: [(Vec<u8>, Vec<u8>); 2],
}

impl Frame {
    /// Every line of 2 or more stays a valid cycle with `first` on the
    /// anchor and `second` on the other cell.
    fn cycles_hold(&self, first: u8, second: u8, buf: &mut Vec<u8>) -> bool {
        buf.clear();
        buf.extend_from_slice(&self.own.0);
        buf.extend([first, second]);
        buf.extend_from_slice(&self.own.1);
        if !valid_cycle(buf) {
            return false;
        }
        self.across
            .iter()
            .zip([first, second])
            .all(|((before, after), v)| {
                if before.is_empty() && after.is_empty() {
                    return true;
                }
                buf.clear();
                buf.extend_from_slice(before);
                buf.push(v);
                buf.extend_from_slice(after);
                valid_cycle(buf)
            })
    }

    /// [R] Rule 1: the product of the lengths of the groups extended, if
    /// there are two or more. The domino's own two cells only count when
    /// they extend a group (line longer than 2); a line across exists only
    /// with a neighbour.
    fn product(&self) -> Option<u32> {
        let own = self.own.0.len() + 2 + self.own.1.len();
        let mut groups: Vec<usize> = Vec::with_capacity(3);
        if own > 2 {
            groups.push(own);
        }
        for (before, after) in &self.across {
            let n = before.len() + 1 + after.len();
            if n >= 2 {
                groups.push(n);
            }
        }
        (groups.len() >= 2).then(|| groups.iter().map(|&n| n as u32).product())
    }
}
