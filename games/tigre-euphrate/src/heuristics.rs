//! The AI: generic ISMCTS over a dozen sensible moves, simulations played
//! by a greedy policy and cut after a couple of rounds.

use ludo_core::{Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::board::{
    kingdoms, kingdoms_next_to, neighbors, regions, square_cells, squares_with, CELLS, NONE,
};
use crate::{
    sorted_points, Action, Color, ConflictKind, Phase, Piece, State, TigreEuphrate, COLORS,
    MONUMENTS,
};

/// Moves the search considers in the action phase.
const WIDTH: usize = 12;
/// Simulated decisions per player before the estimate replaces the rest.
const HORIZON: u32 = 5;

/// How much a point of each colour is worth to `seat`: the weakest colour
/// most, since only it counts in the end.
fn weights(s: &State, seat: Seat) -> [f32; 4] {
    let mine = s.scores[seat.index()];
    let low = *mine.iter().min().expect("four colours");
    mine.map(|p| 1.0 + 2.0 / (1.0 + (p - low) as f32))
}

/// The regions of a board with, for each, its leaders and tiles by colour:
/// computed once per state, so that scoring a move is cheap.
struct Regions {
    label: Vec<u8>,
    kingdom: Vec<bool>,
    leader: Vec<[Option<(Seat, u8)>; 4]>,
    tiles: Vec<[u8; 4]>,
}

impl Regions {
    fn of(s: &State, skip: Option<u8>) -> Self {
        let label = regions(&s.cells, skip);
        let kingdom = kingdoms(&s.cells, &label);
        let mut leader = vec![[None; 4]; CELLS];
        let mut tiles = vec![[0u8; 4]; CELLS];
        for (c, piece) in s.cells.iter().enumerate() {
            let l = label[c];
            if l == NONE {
                continue;
            }
            match *piece {
                Some(Piece::Leader { seat, color }) => {
                    leader[usize::from(l)][color.index()] = Some((seat, c as u8));
                }
                Some(Piece::Tile {
                    color,
                    flipped: false,
                }) => tiles[usize::from(l)][color.index()] += 1,
                _ => {}
            }
        }
        Self {
            label,
            kingdom,
            leader,
            tiles,
        }
    }
}

/// The leader of `color` in the regions `labels` (owner and cell).
fn leader_in(r: &Regions, labels: &[u8], color: Color) -> Option<(Seat, u8)> {
    labels
        .iter()
        .find_map(|&l| r.leader[usize::from(l)][color.index()])
}

fn tiles_in(r: &Regions, labels: &[u8], color: Color) -> u8 {
    labels
        .iter()
        .map(|&l| r.tiles[usize::from(l)][color.index()])
        .sum()
}

/// The distinct regions next to `cell`, and how many.
fn regions_next_to(cell: u8, label: &[u8]) -> ([u8; 4], usize) {
    let mut found = [NONE; 4];
    let mut n = 0;
    for c in neighbors(cell) {
        let l = label[usize::from(c)];
        if l != NONE && !found[..n].contains(&l) {
            found[n] = l;
            n += 1;
        }
    }
    (found, n)
}

/// Whether the tile completes a square on which a monument can rise.
fn completes_square(s: &State, color: Color, to: u8) -> bool {
    let free = MONUMENTS
        .iter()
        .zip(&s.monuments)
        .any(|(cols, at)| at.is_none() && cols.contains(&color));
    free && squares_with(to).any(|sq| {
        !s.declined.contains(&sq)
            && square_cells(sq).iter().all(|&c| {
                c == to
                    || s.cells[usize::from(c)]
                        == Some(Piece::Tile {
                            color,
                            flipped: false,
                        })
            })
    })
}

/// The legal moves of `seat`, best first, scored on what it can see.
pub(crate) fn ranked(s: &State, seat: Seat) -> Vec<(f32, crate::Action)> {
    let legal = s.moves(seat);
    let mut scored: Vec<(f32, Action)> = match &s.phase {
        Phase::Action => {
            let here = Regions::of(s, None);
            let mine = COLORS.map(|c| s.leader_at(seat, c));
            let lifted: Vec<Option<Regions>> = mine
                .iter()
                .map(|from| from.map(|f| Regions::of(s, Some(f))))
                .collect();
            let w = weights(s, seat);
            let hand = s.hands[seat.index()];
            // The tiles no leader of mine can score (temples kept for
            // conflicts): swapping them is the one sensible swap.
            let useless: [u8; 4] = COLORS.map(|c| {
                if c == Color::Red || mine[c.index()].is_some() {
                    0
                } else {
                    hand[c.index()]
                }
            });
            legal
                .into_iter()
                .map(|a| {
                    let v = match a {
                        Action::Tile { color, to } => {
                            tile_value(s, seat, &here, &w, &hand, color, to)
                        }
                        Action::Leader { color, to } => {
                            let r = lifted[color.index()].as_ref().unwrap_or(&here);
                            let from = mine[color.index()];
                            leader_value(s, seat, r, &w, &hand, color, to, from)
                        }
                        Action::Withdraw { .. } => -2.0,
                        Action::Catastrophe { at } => catastrophe_value(s, seat, at),
                        Action::Swap { tiles } => swap_value(&useless, &tiles),
                        _ => 0.0,
                    };
                    (v, a)
                })
                .collect()
        }
        Phase::Reinforce(c) => {
            let base_mine = if c.attacker_added.is_none() {
                c.attacker_base
            } else {
                c.defender_base
            };
            let against = match c.attacker_added {
                None => c.defender_base,
                Some(added) => c.attacker_base + added,
            };
            let attacking = c.attacker_added.is_none();
            legal
                .into_iter()
                .map(|a| {
                    let Action::Reinforce { count } = a else {
                        return (0.0, a);
                    };
                    let total = base_mine + count;
                    let wins = if attacking {
                        total > against
                    } else {
                        total >= against
                    };
                    // The cheapest winning count, else nothing.
                    let v = if wins {
                        10.0 - f32::from(count)
                    } else {
                        -f32::from(count)
                    };
                    (v, a)
                })
                .collect()
        }
        Phase::Monument(_) => {
            let label = regions(&s.cells, None);
            legal
                .into_iter()
                .map(|a| {
                    let v = match a {
                        Action::Monument { square, monument } => {
                            let k = label[usize::from(square)];
                            1.0 + MONUMENTS[usize::from(monument)]
                                .iter()
                                .filter(|&&c| {
                                    s.leader_at(seat, c)
                                        .is_some_and(|at| label[usize::from(at)] == k)
                                })
                                .count() as f32
                        }
                        _ => 0.0,
                    };
                    (v, a)
                })
                .collect()
        }
        _ => legal.into_iter().map(|a| (0.0, a)).collect(),
    };
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    scored
}

fn tile_value(
    s: &State,
    seat: Seat,
    r: &Regions,
    w: &[f32; 4],
    hand: &[u8; 4],
    color: Color,
    to: u8,
) -> f32 {
    let joined = kingdoms_next_to(to, &r.label, &r.kingdom);
    let mut v = match joined[..] {
        [] => {
            if regions_next_to(to, &r.label).1 == 0 {
                -0.2
            } else {
                0.05
            }
        }
        [k] => {
            let scorer = leader_in(r, &[k], color)
                .or_else(|| leader_in(r, &[k], Color::Black))
                .map(|(owner, _)| owner);
            match scorer {
                Some(owner) if owner == seat => 2.0 * w[color.index()],
                Some(_) => -1.0,
                None => 0.1,
            }
        }
        _ => {
            // A war: worth it when it looks winnable.
            let mut v = -0.5;
            for col in COLORS {
                let (Some(a), Some(b)) = (
                    leader_in(r, &[joined[0]], col),
                    leader_in(r, &[joined[1]], col),
                ) else {
                    continue;
                };
                let (mine, theirs) = if a.0 == seat {
                    (joined[0], joined[1])
                } else if b.0 == seat {
                    (joined[1], joined[0])
                } else {
                    continue;
                };
                let spare = hand[col.index()] - u8::from(col == color);
                let my_side = tiles_in(r, &[mine], col) + spare;
                let their_side = tiles_in(r, &[theirs], col);
                v += if my_side > their_side + 1 {
                    (2.0 + f32::from(their_side)) * w[col.index()]
                } else {
                    -3.0
                };
            }
            v
        }
    };
    if completes_square(s, color, to) {
        v += 3.0;
    }
    v
}

#[allow(clippy::too_many_arguments)]
fn leader_value(
    s: &State,
    seat: Seat,
    r: &Regions,
    w: &[f32; 4],
    hand: &[u8; 4],
    color: Color,
    to: u8,
    from: Option<u8>,
) -> f32 {
    let (around, n) = regions_next_to(to, &r.label);
    let around = &around[..n];
    let tiles = f32::from(tiles_in(r, around, color));
    let temples = f32::from(s.temples_next_to(to));
    let mut v = 0.3 * tiles + 0.3 * temples;
    if color == Color::Black {
        let all: f32 = COLORS
            .iter()
            .map(|&c| f32::from(tiles_in(r, around, c)))
            .sum();
        v += 0.1 * all;
    }
    match leader_in(r, around, color) {
        Some((rival, at)) if rival != seat => {
            let mine = temples + f32::from(hand[Color::Red.index()]);
            let theirs = f32::from(s.temples_next_to(at)) + 1.0;
            v += if mine > theirs {
                3.0 * w[Color::Red.index()]
            } else {
                -3.0
            };
        }
        _ => {}
    }
    match from {
        None => v + 1.5,
        Some(old) => {
            let (before, n) = regions_next_to(old, &r.label);
            v - 0.3 * f32::from(tiles_in(r, &before[..n], color)) - 0.8
        }
    }
}

/// [R] A catastrophe is worth playing to send an opponent's leader home.
fn catastrophe_value(s: &State, seat: Seat, at: u8) -> f32 {
    if s.cells[usize::from(at)]
        != Some(Piece::Tile {
            color: Color::Red,
            flipped: false,
        })
    {
        return -1.5;
    }
    let mut v = -1.5;
    for n in neighbors(at) {
        if let Some(Piece::Leader { seat: owner, .. }) = s.cells[usize::from(n)] {
            if s.temples_next_to(n) == 1 {
                v += if owner == seat { -3.0 } else { 2.5 };
            }
        }
    }
    v
}

/// Swapping is worth it to get rid of tiles no leader of mine can score.
fn swap_value(useless: &[u8; 4], tiles: &[u8; 4]) -> f32 {
    if tiles == useless && useless.iter().sum::<u8>() >= 3 {
        0.4 * f32::from(useless.iter().sum::<u8>())
    } else {
        -5.0
    }
}

/// Each seat's standing: its weakest colours, then its leaders on the board.
fn standing(s: &State, seat: usize) -> f64 {
    let p = sorted_points(&s.scores[seat], s.treasures[seat]);
    let leaders = s
        .cells
        .iter()
        .filter(|c| matches!(c, Some(Piece::Leader { seat: o, .. }) if o.index() == seat))
        .count();
    f64::from(p[0]) + 0.3 * f64::from(p[1]) + 0.1 * f64::from(p[2]) + 0.5 * leaders as f64
}

impl Heuristics for TigreEuphrate {
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let ranked = ranked(s, seat);
        if rng.gen_bool(0.8) {
            let best = ranked[0].0;
            let ties: Vec<&(f32, Action)> = ranked.iter().take_while(|(v, _)| *v >= best).collect();
            ties.choose(rng).expect("a pending seat has a move").1
        } else {
            ranked[..ranked.len().min(5)]
                .choose(rng)
                .expect("a pending seat has a move")
                .1
        }
    }

    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let ranked = ranked(s, seat);
        if !matches!(s.phase, Phase::Action) {
            return ranked.into_iter().map(|(_, a)| a).collect();
        }
        let mut v: Vec<Action> = ranked.into_iter().take(WIDTH).map(|(_, a)| a).collect();
        if !v.contains(&Action::EndTurn) {
            v.push(Action::EndTurn);
        }
        v
    }

    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        if sim.moves < root.moves + HORIZON * u32::from(root.players) {
            return None;
        }
        // In the middle of a conflict the board is not settled yet.
        if matches!(sim.phase, Phase::Reinforce(c) if c.kind == ConflictKind::External) {
            return None;
        }
        let n = usize::from(sim.players);
        let v: Vec<f64> = (0..n).map(|i| standing(sim, i)).collect();
        Some(
            (0..n)
                .map(|i| {
                    let sum: f64 = (0..n)
                        .filter(|&j| j != i)
                        .map(|j| 1.0 / (1.0 + (-(v[i] - v[j]) / 2.0).exp()))
                        .sum();
                    sum / (n - 1) as f64
                })
                .collect(),
        )
    }
}
