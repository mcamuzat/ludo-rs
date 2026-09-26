use ludo_core::{Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{Action, Cell, Effect, State, Yavalath};

impl Heuristics for Yavalath {
    /// Win if possible; otherwise block the next player's win with a safe
    /// move; otherwise any move that does not make 3; otherwise anything.
    /// Never swaps: swapping is left to the tree search.
    fn rollout_action<R: Rng + ?Sized>(s: &State, _seat: Seat, rng: &mut R) -> Action {
        let me = s.current.expect("rollout on a finished game");
        let cells = s.legal_cells();
        if let Some(&win) = cells
            .iter()
            .find(|&&c| s.board.effect(c, me) == Effect::Wins)
        {
            return Action::Place(win);
        }
        let safe: Vec<Cell> = cells
            .iter()
            .copied()
            .filter(|&c| s.board.effect(c, me) != Effect::Loses)
            .collect();
        let next = s.next_color(me);
        let blocks: Vec<Cell> = if next == me {
            Vec::new()
        } else {
            safe.iter()
                .copied()
                .filter(|&c| s.board.effect(c, next) == Effect::Wins)
                .collect()
        };
        let pool = if !blocks.is_empty() {
            &blocks
        } else if !safe.is_empty() {
            &safe
        } else {
            &cells
        };
        Action::Place(
            *pool
                .choose(rng)
                .expect("an unfinished game has an empty cell"),
        )
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::{Action, Cell, Color, Config, State, Yavalath};

    fn with(stones: &[(Color, i8, i8)]) -> State {
        let mut s = Yavalath::new(&Config::default(), 2, 0);
        for &(color, q, r) in stones {
            s.board.set(Cell::new(q, r), Some(color));
        }
        s.stones = 2;
        s
    }

    #[test]
    fn takes_an_immediate_win() {
        let s = with(&[
            (Color::White, 0, 0),
            (Color::White, 1, 0),
            (Color::White, 3, 0),
        ]);
        for seed in 0..20 {
            let a = Yavalath::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(a, Action::Place(Cell::new(2, 0)));
        }
    }

    #[test]
    fn blocks_the_next_players_win() {
        let s = with(&[
            (Color::Black, 0, 0),
            (Color::Black, 1, 0),
            (Color::Black, 3, 0),
        ]);
        for seed in 0..20 {
            let a = Yavalath::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(a, Action::Place(Cell::new(2, 0)));
        }
    }

    #[test]
    fn avoids_making_three() {
        let s = with(&[(Color::White, 0, 0), (Color::White, 1, 0)]);
        for seed in 0..50 {
            let a = Yavalath::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(
                a != Action::Place(Cell::new(2, 0)) && a != Action::Place(Cell::new(-1, 0)),
                "{a:?}"
            );
        }
    }
}
