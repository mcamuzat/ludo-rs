use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::board::distance_to_finish;
use crate::{ranks, Action, Bantu, Dest, Spot, State};

/// Simulated moves after which a simulation is scored as it stands.
const HORIZON: u32 = 150;

impl State {
    /// Steps a move saves towards the finish (entering from home counts as
    /// progress too), and whether it takes a piece.
    fn gain(&self, seat: Seat, action: &Action) -> (bool, i32) {
        let Action::Move { piece, to } = *action;
        let before = match self.pieces[seat.index()][usize::from(piece) - 1] {
            Spot::On { cell } => i32::from(distance_to_finish(cell)),
            _ => i32::from(u8::MAX),
        };
        match to {
            Dest::Finish => (false, before),
            Dest::Cell(cell) => (
                self.occupant(cell).is_some(),
                before - i32::from(distance_to_finish(cell)),
            ),
        }
    }
}

impl Heuristics for Bantu {
    /// Take a piece if possible, else go in, else the move that gains most,
    /// with a random move one time in four.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if let Some(take) = legal.iter().find(|a| s.gain(seat, a).0) {
            return *take;
        }
        if let Some(home) = legal.iter().find(|a| {
            matches!(
                a,
                Action::Move {
                    to: Dest::Finish,
                    ..
                }
            )
        }) {
            return *home;
        }
        if rng.gen_bool(0.25) {
            return *legal.choose(rng).expect("a pending seat has a move");
        }
        *legal
            .iter()
            .max_by_key(|a| s.gain(seat, a).1)
            .expect("a pending seat has a move")
    }

    /// Long simulations are scored on the ranking as it stands.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.moves >= root.moves + HORIZON).then(|| Outcome::Ranked(ranks(sim)).rewards())
    }
}
