use ludo_core::{Game, Heuristics, Seat};
use rand::Rng;

use crate::board::CellId;
use crate::{is_perfect, perfect_count, Action, Die, SiberianDice, Source, State, Step};

/// Simulated rolls after which a simulation is scored as it stands.
const HORIZON: u32 = 12;
/// Chance that a simulated player places on its best cell.
const GREEDY: f64 = 0.85;
/// Chance that a simulated player takes from its reserve while it can.
const FROM_RESERVE: f64 = 0.7;
/// Reward share per perfect die of difference, around one half.
const PER_DIE: f64 = 0.08;

fn margin(s: &State, seat: Seat) -> i32 {
    i32::from(perfect_count(&s.board, seat.0)) - i32::from(perfect_count(&s.board, 1 - seat.0))
}

/// How much placing the rolled die on `to` changes `seat`'s lead in perfect
/// dice. Only meaningful while a die waits to be placed.
pub fn gain(s: &State, seat: Seat, to: CellId) -> i32 {
    let Step::Place { value, .. } = s.step else {
        return 0;
    };
    let before = margin(s, seat);
    let mut board = s.board.clone();
    board[usize::from(to)] = Some(Die {
        owner: seat.0,
        value,
    });
    let after =
        i32::from(perfect_count(&board, seat.0)) - i32::from(perfect_count(&board, 1 - seat.0));
    after - before
}

fn pick<T: Copy, R: Rng + ?Sized>(items: &[T], rng: &mut R) -> T {
    items[rng.gen::<u32>() as usize % items.len()]
}

impl Heuristics for SiberianDice {
    /// Mostly the best cell for the rolled die; picks from the reserve
    /// while it lasts, else a die that is not perfect when there is one.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        match s.step {
            Step::Choose => {
                if s.reserve[seat.index()] > 0 && rng.gen_bool(FROM_RESERVE) {
                    return Action::Pick(Source::Reserve);
                }
                let loose: Vec<Action> = legal
                    .iter()
                    .copied()
                    .filter(
                        |a| matches!(a, Action::Pick(Source::Cell(c)) if !is_perfect(&s.board, *c)),
                    )
                    .collect();
                pick(if loose.is_empty() { &legal } else { &loose }, rng)
            }
            Step::Place { .. } => {
                if !rng.gen_bool(GREEDY) {
                    return pick(&legal, rng);
                }
                let scored: Vec<(Action, i32)> = legal
                    .iter()
                    .map(|&a| match a {
                        Action::Place(to) => (a, gain(s, seat, to)),
                        Action::Pick(_) => (a, i32::MIN),
                    })
                    .collect();
                let best = scored.iter().map(|&(_, g)| g).max().unwrap_or(0);
                let top: Vec<Action> = scored
                    .into_iter()
                    .filter(|&(_, g)| g == best)
                    .map(|(a, _)| a)
                    .collect();
                pick(&top, rng)
            }
        }
    }

    /// After `HORIZON` rolls, the lead in perfect dice.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        if sim.rolls < root.rolls + HORIZON {
            return None;
        }
        let lead = f64::from(margin(sim, Seat(0)));
        let blue = (0.5 + PER_DIE * lead).clamp(0.05, 0.95);
        Some(vec![blue, 1.0 - blue])
    }
}
