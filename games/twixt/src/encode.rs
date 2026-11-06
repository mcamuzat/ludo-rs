//! The network's inputs, exactly as [TB] `naf.py` (`NetInputs` and
//! `to_input_arrays`): always seen from the side to move, Black's view
//! being the board transposed with its colours swapped.

use crate::board::{Board, Color, Point, SIZE};

const N: usize = SIZE * SIZE;
/// Policy outputs: rows x = 1..=22 of the side to move's frame, 22 × 24.
pub const POLICY: usize = (SIZE - 2) * SIZE;

/// Channel-major planes: `pegs` 2 × 24 × 24, `links` 8 × 24 × 24, indexed
/// [plane][x][y] (x is the network's height axis).
#[derive(Clone, Debug, PartialEq)]
pub struct Inputs {
    pub pegs: Vec<f32>,
    pub links: Vec<f32>,
}

impl Inputs {
    /// From twixtbot's NHWC arrays ([x][y][plane]).
    pub fn from_hwc(pegs: &[f32], links: &[f32]) -> Inputs {
        Inputs {
            pegs: to_chw(pegs, 2),
            links: to_chw(links, 8),
        }
    }

    pub fn pegs_hwc(&self) -> Vec<f32> {
        to_hwc(&self.pegs, 2)
    }

    pub fn links_hwc(&self) -> Vec<f32> {
        to_hwc(&self.links, 8)
    }
}

fn to_chw(hwc: &[f32], planes: usize) -> Vec<f32> {
    let mut out = vec![0.0; planes * N];
    for i in 0..N {
        for p in 0..planes {
            out[p * N + i] = hwc[i * planes + p];
        }
    }
    out
}

fn to_hwc(chw: &[f32], planes: usize) -> Vec<f32> {
    let mut out = vec![0.0; planes * N];
    for i in 0..N {
        for p in 0..planes {
            out[i * planes + p] = chw[p * N + i];
        }
    }
    out
}

fn at(x: usize, y: usize) -> usize {
    x * SIZE + y
}

pub fn encode(board: &Board) -> Inputs {
    // naf planes [plane][x][y]: links 0..8, pegs 8..10.
    let mut naf = vec![0.0f32; 10 * N];
    let red = board.turn == Color::Red;
    for x in 0..SIZE {
        for y in 0..SIZE {
            if red {
                for i in 0..8 {
                    naf[i * N + at(x, y)] = f32::from(u8::from(board.links[i][x][y]));
                }
                for c in 0..2 {
                    naf[(8 + c) * N + at(x, y)] = f32::from(u8::from(board.pegs[c][x][y]));
                }
            } else {
                // init_from_game_black: transposed, colours swapped, and the
                // LONGY/DIFFSIGN planes exchanged by the transposition.
                for c in 0..2 {
                    let a = 1 - c;
                    let link = |plane: usize| f32::from(u8::from(board.links[plane][y][x]));
                    naf[(4 + c) * N + at(x, y)] = link(a);
                    naf[(6 + c) * N + at(x, y)] = link(2 + a);
                    naf[c * N + at(x, y)] = link(4 + a);
                    naf[(2 + c) * N + at(x, y)] = link(6 + a);
                    naf[(8 + c) * N + at(x, y)] = f32::from(u8::from(board.pegs[a][y][x]));
                }
            }
        }
    }
    let pegs = naf[8 * N..10 * N].to_vec();
    // to_input_arrays: some link planes move by one along x, some along y.
    let mut links = vec![0.0f32; 8 * N];
    for index in 0..8 {
        let vertical = index & 4 != 0;
        let diffsign = index & 2 != 0;
        let dx = usize::from(vertical || diffsign);
        let dy = usize::from(!vertical || diffsign);
        for x in dx..SIZE {
            for y in dy..SIZE {
                links[index * N + at(x, y)] = naf[index * N + at(x - dx, y - dy)];
            }
        }
    }
    Inputs { pegs, links }
}

/// naf.py `policy_index_point`.
pub fn policy_index_point(turn: Color, index: usize) -> Point {
    let (major, minor) = (index / SIZE, index % SIZE);
    let (a, b) = ((major + 1) as u8, minor as u8);
    match turn {
        Color::Red => Point { x: a, y: b },
        Color::Black => Point { x: b, y: a },
    }
}

/// naf.py `policy_point_index`.
pub fn policy_point_index(turn: Color, p: Point) -> usize {
    let (major, minor) = match turn {
        Color::Red => (p.x, p.y),
        Color::Black => (p.y, p.x),
    };
    (usize::from(major) - 1) * SIZE + usize::from(minor)
}

/// Probability that the side to move wins, from the value logits
/// [loss, draw, win]: p(win) + p(draw) / 2.
pub fn win_probability(value: &[f32; 3]) -> f32 {
    let m = value.iter().copied().fold(f32::MIN, f32::max);
    let e = value.map(|v| (v - m).exp());
    let sum: f32 = e.iter().sum();
    (e[2] + e[1] / 2.0) / sum
}

/// [TB] `three_to_one`: p(win) − p(loss), in [−1, 1].
pub fn score(value: &[f32; 3]) -> f32 {
    let m = value.iter().copied().fold(f32::MIN, f32::max);
    let e = value.map(|v| (v - m).exp());
    let sum: f32 = e.iter().sum();
    (e[2] - e[0]) / sum
}
