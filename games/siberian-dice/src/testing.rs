//! Hand-built positions for tests.

use ludo_core::{Game, Seat};

use crate::board::{at, cells};
use crate::{perfect_count, Config, Die, SiberianDice, Source, State, Step, DICE};

/// A game with the dice listed `(ab, owner, value)` on the board, the
/// reserves reduced to match, `turn` to choose a die, and the records set
/// to the perfect dice of this position.
pub fn position(dice: &[(u8, u8, u8)], turn: u8) -> State {
    let mut s = SiberianDice::new(&Config::default(), 2, 0);
    s.board = vec![None; cells().len()];
    s.reserve = [DICE; 2];
    for &(ab, owner, value) in dice {
        s.board[usize::from(at(ab))] = Some(Die { owner, value });
        s.reserve[usize::from(owner)] -= 1;
    }
    s.turn = Seat(turn);
    s.step = Step::Choose;
    s.records = [perfect_count(&s.board, 0), perfect_count(&s.board, 1)];
    s
}

/// As if the seat to play had picked the die from `from` and rolled `value`.
pub fn rolled(s: &mut State, from: Source, value: u8) {
    s.take(from, value);
}
