//! [TB] `swapmodel.py`: twixtbot does not search the swap. It played itself
//! and fitted the share of games Red wins after each first move; Black swaps
//! a first move worth more than one half, and the AI opens near one half.

use rand::Rng;

use crate::board::{Point, SIZE};

const BETAS: [f64; 4] = [0.494481, -0.00366079, 0.0225597, 0.00114293];
const HALFLIFE: f64 = 0.008;

/// Red's estimated share of wins after its first peg in `p`.
fn score(p: Point) -> f64 {
    let half = (SIZE / 2) as u8;
    let fold = |v: u8| if v >= half { 2 * half - v - 1 } else { v };
    let xres = f64::from(fold(p.x)) - 6.0;
    let yres = f64::from(fold(p.y)) - 5.5;
    BETAS[0] + BETAS[1] * xres + BETAS[2] * yres + BETAS[3] * xres * yres
}

/// Does Black take Red's first peg in `first`?
pub fn swap_wanted(first: Point) -> bool {
    score(first) > 0.5
}

/// A first move for Red, drawn with a weight halving every `HALFLIFE`
/// away from a fair game.
pub fn first_move<R: Rng + ?Sized>(rng: &mut R) -> Point {
    let points: Vec<(Point, f64)> = (1..SIZE as u8 - 1)
        .flat_map(|x| (0..SIZE as u8).map(move |y| Point { x, y }))
        .map(|p| (p, (0.5f64.ln() * (score(p) - 0.5).abs() / HALFLIFE).exp()))
        .collect();
    let total: f64 = points.iter().map(|(_, w)| w).sum();
    let mut z = rng.gen_range(0.0..total);
    for &(p, w) in &points {
        if z < w {
            return p;
        }
        z -= w;
    }
    points.last().expect("holes").0
}
