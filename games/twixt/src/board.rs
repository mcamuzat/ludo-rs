//! The board, in twixtbot's representation ([TB] `twixt.py`): pegs and the
//! 8 link planes indexed by colour (Black = 0, Red = 1), so the network's
//! inputs are a straight copy.

use std::sync::OnceLock;

use ludo_core::Illegal;
use serde::{Deserialize, Serialize};

pub const SIZE: usize = 24;
const LAST: u8 = SIZE as u8 - 1;

/// A hole: `x` column (0 = left), `y` row (0 = top).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl Point {
    fn offset(self, (dx, dy): (i8, i8)) -> Option<Point> {
        let x = self.x as i8 + dx;
        let y = self.y as i8 + dy;
        ((0..SIZE as i8).contains(&x) && (0..SIZE as i8).contains(&y)).then_some(Point {
            x: x as u8,
            y: y as u8,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TwixtAction {
    Place(Point),
    /// The second player takes the first peg (reflected) instead of playing.
    Swap,
}

/// Red moves first and links top to bottom (twixtbot's WHITE); Black links
/// left to right (BLACK).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    Red,
    Black,
}

impl Color {
    /// twixtbot's index: BLACK = 0, WHITE = 1.
    pub fn index(self) -> usize {
        match self {
            Color::Black => 0,
            Color::Red => 1,
        }
    }

    pub fn other(self) -> Color {
        match self {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }

    /// Seat 0 is Red, seat 1 Black.
    pub fn seat(self) -> u8 {
        match self {
            Color::Red => 0,
            Color::Black => 1,
        }
    }

    /// May `c` put a peg in `p`? Not in the other colour's border rows,
    /// hence never in a corner.
    pub fn allowed(self, p: Point) -> bool {
        match self {
            Color::Red => (1..LAST).contains(&p.x),
            Color::Black => (1..LAST).contains(&p.y),
        }
    }

    fn at_start(self, p: Point) -> bool {
        match self {
            Color::Red => p.y == 0,
            Color::Black => p.x == 0,
        }
    }

    fn at_end(self, p: Point) -> bool {
        match self {
            Color::Red => p.y == LAST,
            Color::Black => p.x == LAST,
        }
    }
}

/// The 8 knight moves, in twixtbot's `DLINKS` order; the opposite of
/// direction `d` is `(d + 4) % 8`.
pub const DIRS: [(i8, i8); 8] = [
    (-2, -1),
    (-1, -2),
    (1, -2),
    (2, -1),
    (2, 1),
    (1, 2),
    (-1, 2),
    (-2, 1),
];

/// twixtbot's `get_link_index`: the plane and the centre cell of a link.
pub fn link_index(a: Point, b: Point, c: Color) -> (usize, usize, usize) {
    let mut plane = c.index();
    if !(a.x + b.x).is_multiple_of(2) {
        plane += 4; // LINK_LONGY
    }
    let dx = b.x as i8 - a.x as i8;
    let dy = b.y as i8 - a.y as i8;
    if dx * dy < 0 {
        plane += 2; // LINK_DIFFSIGN
    }
    (
        plane,
        usize::from((a.x + b.x) / 2),
        usize::from((a.y + b.y) / 2),
    )
}

fn cross(o: (i32, i32), a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

/// Do segments p1–p2 and q1–q2 cross strictly (no shared end)?
fn segments_cross(p1: (i32, i32), p2: (i32, i32), q1: (i32, i32), q2: (i32, i32)) -> bool {
    let d1 = cross(q1, q2, p1);
    let d2 = cross(q1, q2, p2);
    let d3 = cross(p1, p2, q1);
    let d4 = cross(p1, p2, q2);
    d1 * d2 < 0 && d3 * d4 < 0
}

/// Knight segments as (start offset, direction).
type Crossings = Vec<((i8, i8), usize)>;

/// For a link from (0,0) in direction `d`: every knight segment
/// (start offset, direction) it crosses.
fn crossings(d: usize) -> &'static [((i8, i8), usize)] {
    static TABLE: OnceLock<Vec<Crossings>> = OnceLock::new();
    &TABLE.get_or_init(|| {
        (0..8)
            .map(|d| {
                let (dx, dy) = DIRS[d];
                let a = (0, 0);
                let b = (i32::from(dx), i32::from(dy));
                let mut out = Vec::new();
                for ox in -3..=3i8 {
                    for oy in -3..=3i8 {
                        for (e, &(ex, ey)) in DIRS.iter().enumerate() {
                            let q1 = (i32::from(ox), i32::from(oy));
                            let q2 = (q1.0 + i32::from(ex), q1.1 + i32::from(ey));
                            if segments_cross(a, b, q1, q2) {
                                out.push(((ox, oy), e));
                            }
                        }
                    }
                }
                out
            })
            .collect()
    })[d]
}

#[derive(Clone, Debug)]
pub struct Board {
    /// [colour index][x][y]
    pub pegs: [[[bool; SIZE]; SIZE]; 2],
    /// twixtbot's link planes: [plane][centre x][centre y]
    pub links: [[[bool; SIZE]; SIZE]; 8],
    /// [colour index][x][y]: bit d set when an opposing link crosses the
    /// knight segment from (x,y) in direction d. Own links never block.
    blocked: [[[u8; SIZE]; SIZE]; 2],
    pub turn: Color,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            pegs: [[[false; SIZE]; SIZE]; 2],
            links: [[[false; SIZE]; SIZE]; 8],
            blocked: [[[0; SIZE]; SIZE]; 2],
            turn: Color::Red,
        }
    }

    pub fn peg(&self, p: Point) -> Option<Color> {
        [Color::Black, Color::Red]
            .into_iter()
            .find(|c| self.pegs[c.index()][usize::from(p.x)][usize::from(p.y)])
    }

    pub fn has_link(&self, c: Color, a: Point, b: Point) -> bool {
        let (plane, x, y) = link_index(a, b, c);
        self.links[plane][x][y]
    }

    fn is_blocked(&self, c: Color, p: Point, d: usize) -> bool {
        self.blocked[c.index()][usize::from(p.x)][usize::from(p.y)] & (1 << d) != 0
    }

    /// Can the colour to move put a peg in `p`?
    pub fn legal(&self, p: Point) -> bool {
        usize::from(p.x) < SIZE
            && usize::from(p.y) < SIZE
            && self.turn.allowed(p)
            && self.peg(p).is_none()
    }

    /// Plays `a` for the colour to move; `ply` is the number of moves
    /// already played (a swap is only allowed at ply 1).
    pub fn play(&mut self, a: &TwixtAction, ply: usize) -> Result<(), Illegal> {
        match *a {
            TwixtAction::Swap => {
                if ply != 1 || self.turn != Color::Black {
                    return Err(Illegal::new(
                        "Le swap n'est permis qu'au premier coup de Noir.",
                    ));
                }
                let first = self
                    .pegs_of(Color::Red)
                    .first()
                    .copied()
                    .ok_or_else(|| Illegal::new("Aucun pion à prendre."))?;
                // [TB] play_swap: the peg (x,y) becomes Black's (y,x).
                self.pegs[Color::Red.index()][usize::from(first.x)][usize::from(first.y)] = false;
                self.pegs[Color::Black.index()][usize::from(first.y)][usize::from(first.x)] = true;
                self.turn = Color::Red;
                Ok(())
            }
            TwixtAction::Place(p) => {
                if !self.legal(p) {
                    return Err(Illegal::new("Ce trou n'est pas jouable."));
                }
                let c = self.turn;
                self.pegs[c.index()][usize::from(p.x)][usize::from(p.y)] = true;
                for (d, &dir) in DIRS.iter().enumerate() {
                    let Some(q) = p.offset(dir) else { continue };
                    if self.peg(q) == Some(c) && !self.is_blocked(c, p, d) {
                        self.add_link(c, p, d);
                    }
                }
                self.turn = c.other();
                Ok(())
            }
        }
    }

    fn add_link(&mut self, c: Color, p: Point, d: usize) {
        let q = p.offset(DIRS[d]).expect("on the board");
        let (plane, x, y) = link_index(p, q, c);
        self.links[plane][x][y] = true;
        // Every knight segment it crosses is now blocked for the other colour.
        let them = c.other().index();
        for &(o, e) in crossings(d) {
            let Some(s) = p.offset(o) else { continue };
            let Some(t) = s.offset(DIRS[e]) else { continue };
            self.blocked[them][usize::from(s.x)][usize::from(s.y)] |= 1 << e;
            self.blocked[them][usize::from(t.x)][usize::from(t.y)] |= 1 << ((e + 4) % 8);
        }
    }

    pub fn pegs_of(&self, c: Color) -> Vec<Point> {
        all_points()
            .filter(|p| self.pegs[c.index()][usize::from(p.x)][usize::from(p.y)])
            .collect()
    }

    /// Every link, each once, as (colour, end, end).
    pub fn link_list(&self) -> Vec<(Color, Point, Point)> {
        let mut out = Vec::new();
        for c in [Color::Red, Color::Black] {
            for p in self.pegs_of(c) {
                // Directions 3..=6 point right or down: each link once.
                for &dir in &DIRS[3..=6] {
                    if let Some(q) = p.offset(dir) {
                        if self.peg(q) == Some(c) && self.has_link(c, p, q) {
                            out.push((c, p, q));
                        }
                    }
                }
            }
        }
        out
    }

    /// The linked group of `c`'s pegs containing `from`.
    fn group(&self, c: Color, from: Point) -> Vec<Point> {
        let mut seen = vec![from];
        let mut todo = vec![from];
        while let Some(p) = todo.pop() {
            for d in DIRS {
                let Some(q) = p.offset(d) else { continue };
                if self.peg(q) == Some(c) && !seen.contains(&q) && self.has_link(c, p, q) {
                    seen.push(q);
                    todo.push(q);
                }
            }
        }
        seen
    }

    /// The pegs of a group joining a colour's two borders.
    pub fn winning_chain(&self) -> Option<Vec<Point>> {
        [Color::Red, Color::Black].into_iter().find_map(|c| {
            self.pegs_of(c)
                .into_iter()
                .filter(|&p| c.at_start(p))
                .map(|p| self.group(c, p))
                .find(|g| g.iter().any(|&q| c.at_end(q)))
        })
    }

    pub fn winner(&self) -> Option<Color> {
        self.winning_chain()
            .and_then(|g| g.first().and_then(|&p| self.peg(p)))
    }

    /// Can `c` still join its borders: a path of knight steps through holes
    /// that are free or `c`'s, none crossed by an opposing link?
    pub fn can_still_connect(&self, c: Color) -> bool {
        let usable = |p: Point| c.allowed(p) && self.peg(p) != Some(c.other());
        let mut seen = [[false; SIZE]; SIZE];
        let mut todo: Vec<Point> = all_points()
            .filter(|&p| c.at_start(p) && usable(p))
            .collect();
        for p in &todo {
            seen[usize::from(p.x)][usize::from(p.y)] = true;
        }
        while let Some(p) = todo.pop() {
            if c.at_end(p) {
                return true;
            }
            for (d, &dir) in DIRS.iter().enumerate() {
                let Some(q) = p.offset(dir) else { continue };
                if seen[usize::from(q.x)][usize::from(q.y)]
                    || !usable(q)
                    || self.is_blocked(c, p, d)
                {
                    continue;
                }
                seen[usize::from(q.x)][usize::from(q.y)] = true;
                todo.push(q);
            }
        }
        false
    }
}

pub fn all_points() -> impl Iterator<Item = Point> {
    (0..SIZE as u8).flat_map(|x| (0..SIZE as u8).map(move |y| Point { x, y }))
}

/// The board after `moves`, Red first.
pub fn replay(moves: &[TwixtAction]) -> Result<Board, Illegal> {
    let mut b = Board::new();
    for (ply, m) in moves.iter().enumerate() {
        b.play(m, ply)?;
    }
    Ok(b)
}
