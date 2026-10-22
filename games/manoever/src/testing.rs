//! Hand-built positions for tests.

use ludo_core::{Game, Seat};

use crate::{soldiers_per_seat, Config, Manoever, Soldier, State};

/// A game of `players` past its setup, `turn` to move, nothing moved yet.
/// The listed soldiers (seat, number, square) are stacked in the order
/// given (first at the bottom); every other soldier is already in the
/// barracks, seat by seat and number by number, from place XII downwards.
pub fn position(players: u8, soldiers: &[(u8, u8, u8)], turn: u8) -> State {
    let mut s = Manoever::new(&Config::default(), players, 0);
    s.waiting = vec![0; usize::from(players)];
    for &(seat, number, square) in soldiers {
        s.stacks[usize::from(square)].push(Soldier {
            seat: Seat(seat),
            number,
        });
    }
    let mut place = 12;
    for seat in 0..players {
        for number in 1..=soldiers_per_seat(players) {
            if !soldiers.iter().any(|&(s, n, _)| s == seat && n == number) {
                place -= 1;
                s.barracks[place] = Some(Soldier {
                    seat: Seat(seat),
                    number,
                });
            }
        }
    }
    s.turn = Some(Seat(turn));
    s
}
