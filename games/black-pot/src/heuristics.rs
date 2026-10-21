use ludo_core::ai::{Budget, Level};
use ludo_core::{Heuristics, Seat};
use rand::Rng;

use crate::board::{windows, windows_of, CellId, Window};
use crate::lines::{new_alignments, points};
use crate::{plays, score, team_of, Action, BlackPot, State, UNPLAYED_POINTS};

/// Weight of an open window by the pawns a team has in it (0 to 5).
const WEIGHT: [f64; 6] = [0.0, 0.003, 0.015, 0.06, 0.3, 0.0];
/// Moves the search weighs at each node, besides the move placing nothing.
const SEARCH_WIDTH: usize = 12;
/// Moves shortlisted by their cells' gains before `worth` ranks them.
const SHORTLIST: usize = 36;
/// Moves a simulated player compares before choosing.
const ROLLOUT_SAMPLE: usize = 12;
/// Simulated turns after which a simulation is scored as it stands.
const HORIZON: u32 = 4;
/// Points that make the difference between a sure win and a sure loss.
const SCALE: f64 = 300.0;

/// What one window promises `team`: nothing once an opposing pawn is in
/// it or one of its cells already counts in its direction; else its points
/// weighted by how many of `team`'s pawns it holds.
fn promise(w: &Window, owner: &[Option<u8>], counted: &[[bool; 4]], team: u8) -> f64 {
    let mut mine = 0;
    for &c in &w.cells {
        let c = usize::from(c);
        if counted[c][w.dir.index()] {
            return 0.0;
        }
        match owner[c] {
            Some(t) if t == team => mine += 1,
            Some(_) => return 0.0,
            None => {}
        }
    }
    f64::from(points(&w.cells)) * WEIGHT[mine]
}

/// What `team` may still make: the promise of every window.
pub fn potential(owner: &[Option<u8>], counted: &[[bool; 4]], team: u8) -> f64 {
    windows()
        .iter()
        .map(|w| promise(w, owner, counted, team))
        .sum()
}

/// How good a move is for its team, up to a constant of the position: the
/// points it makes, the pawns it spends (each one would be worth 10 at the
/// end), and how the promise of the windows it touches changes for each
/// side.
fn worth(s: &State, seat: Seat, action: &Action) -> f64 {
    let Action::Play { groups } = action else {
        return f64::MIN;
    };
    let team = team_of(s.players, seat);
    let placed: Vec<CellId> = groups.iter().filter_map(|g| g.cell).collect();
    let mut touched: Vec<usize> = placed
        .iter()
        .flat_map(|&c| windows_of(c).iter().copied())
        .collect();
    touched.sort_unstable();
    touched.dedup();
    let mut owner = s.owner.clone();
    for &c in &placed {
        owner[usize::from(c)] = Some(team);
    }
    let mut change = 0.0;
    let mut completes = false;
    for &i in &touched {
        let w = &windows()[i];
        let before =
            promise(w, &s.owner, &s.counted, team) - promise(w, &s.owner, &s.counted, 1 - team);
        let after = promise(w, &owner, &s.counted, team) - promise(w, &owner, &s.counted, 1 - team);
        change += after - before;
        completes |= w.cells.iter().all(|&c| {
            owner[usize::from(c)] == Some(team) && !s.counted[usize::from(c)][w.dir.index()]
        });
    }
    let made: u32 = if completes {
        new_alignments(&owner, &s.counted, team)
            .iter()
            .map(|a| a.points)
            .sum()
    } else {
        0
    };
    f64::from(made) - f64::from(UNPLAYED_POINTS) * placed.len() as f64 + change
}

/// What placing a pawn of `team` on each free cell alone is worth (the
/// same terms as `worth`), to shortlist moves cheaply.
fn cell_gains(s: &State, team: u8) -> Vec<f64> {
    let mut owner = s.owner.clone();
    (0..s.owner.len())
        .map(|c| {
            if s.owner[c].is_some() {
                return f64::MIN;
            }
            owner[c] = Some(team);
            let mut gain = -f64::from(UNPLAYED_POINTS);
            for &i in windows_of(c as CellId) {
                let w = &windows()[i];
                gain += promise(w, &owner, &s.counted, team)
                    - promise(w, &s.owner, &s.counted, team)
                    - promise(w, &owner, &s.counted, 1 - team)
                    + promise(w, &s.owner, &s.counted, 1 - team);
                if w.cells.iter().all(|&x| {
                    owner[usize::from(x)] == Some(team) && !s.counted[usize::from(x)][w.dir.index()]
                }) {
                    gain += f64::from(points(&w.cells));
                }
            }
            owner[c] = None;
            gain
        })
        .collect()
}

fn best(s: &State, seat: Seat, moves: impl Iterator<Item = Action>) -> Option<Action> {
    moves
        .map(|a| (worth(s, seat, &a), a))
        .max_by(|(x, _), (y, _)| x.total_cmp(y))
        .map(|(_, a)| a)
}

fn places_nothing(action: &Action) -> bool {
    matches!(action, Action::Play { groups } if groups.iter().all(|g| g.cell.is_none()))
}

impl Heuristics for BlackPot {
    /// [U] The AI always finds its moves: the best of a sample, three
    /// times in four, else any move; never « Je ne trouve pas ».
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let mut moves = plays(s, seat);
        let pick = |rng: &mut R, n: usize| (rng.gen::<u32>() as usize) % n;
        if moves.len() > ROLLOUT_SAMPLE {
            for i in 0..ROLLOUT_SAMPLE {
                let j = i + pick(rng, moves.len() - i);
                moves.swap(i, j);
            }
            moves.truncate(ROLLOUT_SAMPLE);
        }
        if rng.gen_bool(0.25) {
            let i = pick(rng, moves.len());
            return moves.swap_remove(i);
        }
        best(s, seat, moves.into_iter()).expect("a pending seat has a move")
    }

    /// The best moves by `worth`, plus the move placing nothing. Moves
    /// are first shortlisted by the sum of their cells' gains alone.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let gains = cell_gains(s, team_of(s.players, seat));
        let mut zero = None;
        let mut shortlist: Vec<(f64, Action)> = Vec::new();
        for a in plays(s, seat) {
            if places_nothing(&a) {
                zero.get_or_insert(a);
                continue;
            }
            let Action::Play { groups } = &a else {
                continue;
            };
            let rough = groups
                .iter()
                .filter_map(|g| g.cell)
                .map(|c| gains[usize::from(c)])
                .sum();
            shortlist.push((rough, a));
        }
        shortlist.sort_by(|(x, _), (y, _)| y.total_cmp(x));
        shortlist.truncate(SHORTLIST);
        let mut scored: Vec<(f64, Action)> = shortlist
            .into_iter()
            .map(|(_, a)| (worth(s, seat, &a), a))
            .collect();
        scored.sort_by(|(x, _), (y, _)| y.total_cmp(x));
        let mut chosen: Vec<Action> = scored
            .into_iter()
            .take(SEARCH_WIDTH)
            .map(|(_, a)| a)
            .collect();
        chosen.extend(zero);
        chosen
    }

    fn random_actions(s: &State, seat: Seat) -> Vec<Action> {
        plays(s, seat)
    }

    /// Simulations stop after a few turns, scored on each team's points
    /// and what its open lines promise.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        if sim.end.is_some() || sim.turns < root.turns + HORIZON {
            return None;
        }
        let value =
            |team: u8| f64::from(score(sim, team)) + potential(&sim.owner, &sim.counted, team);
        let lead = (value(0) - value(1)) / SCALE;
        let team0 = (0.5 + lead / 2.0).clamp(0.0, 1.0);
        Some(
            (0..sim.players)
                .map(|seat| {
                    if team_of(sim.players, Seat(seat)) == 0 {
                        team0
                    } else {
                        1.0 - team0
                    }
                })
                .collect(),
        )
    }

    /// Each move costs a full enumeration: fewer iterations than the
    /// platform's default.
    fn budget(level: Level, timed: bool) -> Option<Budget> {
        Some(match (level, timed) {
            (_, true) => level.default_budget(true),
            (Level::Easy, false) => Budget::Iterations(60),
            (Level::Medium, false) => Budget::Iterations(300),
            (Level::Hard, false) => Budget::Iterations(1000),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::cells;

    #[test]
    fn an_empty_board_promises_the_same_to_both() {
        let owner = vec![None; cells().len()];
        let counted = vec![[false; 4]; cells().len()];
        assert_eq!(potential(&owner, &counted, 0), 0.0);
        assert_eq!(potential(&owner, &counted, 1), 0.0);
    }
}
