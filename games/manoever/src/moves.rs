//! How far a soldier may go, where it may land, and who still has to move.

use ludo_core::Seat;

use crate::board::{disc, Disc, Square, GRAYS, LAST};
use crate::{Dest, Soldier, State};

/// The square of `soldier` and its index in the stack (0 at the bottom).
pub fn find(s: &State, soldier: Soldier) -> Option<(Square, usize)> {
    (1..=LAST).find_map(|sq| {
        s.stacks[usize::from(sq)]
            .iter()
            .position(|&x| x == soldier)
            .map(|i| (sq, i))
    })
}

/// [R][K] Rank in the race: 1 plus the squares holding a soldier ahead of
/// `sq`, whatever their colour or number. Empty squares, bonus pieces and
/// soldiers in the barracks do not count.
pub fn position(s: &State, sq: Square) -> u8 {
    1 + (sq + 1..=LAST)
        .filter(|&q| !s.stacks[usize::from(q)].is_empty())
        .count() as u8
}

/// [R] Level in the stack: 1 on the board; a white or black piece counts as
/// a level.
pub fn height(_s: &State, sq: Square, index: usize) -> u8 {
    index as u8 + 1 + u8::from(disc(sq).is_some())
}

/// [R] ×2 standing directly on a white piece, ×3 on a black one [U].
pub fn bonus(sq: Square, index: usize) -> u8 {
    match (disc(sq), index) {
        (Some(Disc::White), 0) => 2,
        (Some(Disc::Black), 0) => 3,
        _ => 1,
    }
}

/// [R] Position × height × bonus, counted as the soldier moves (❓); 0 for a
/// soldier off the track.
pub fn max_move(s: &State, soldier: Soldier) -> u32 {
    find(s, soldier).map_or(0, |(sq, i)| {
        u32::from(position(s, sq)) * u32::from(height(s, sq, i)) * u32::from(bonus(sq, i))
    })
}

/// [R] A stack never rises above 3 levels, white or black piece included.
pub fn room(s: &State, sq: Square) -> bool {
    s.stacks[usize::from(sq)].len() + usize::from(disc(sq).is_some()) < 3
}

fn is_free(s: &State, soldier: Soldier, sq: Square) -> bool {
    s.stacks[usize::from(sq)].last() == Some(&soldier)
}

/// Where `soldier` may go: at least 1 square, at most its maximum, onto a
/// square with room; the barracks once if its maximum goes past 84 (❓ no
/// exact count). Empty when it is covered or off the track.
pub fn destinations(s: &State, soldier: Soldier) -> Vec<Dest> {
    let Some((sq, _)) = find(s, soldier) else {
        return Vec::new();
    };
    if !is_free(s, soldier, sq) {
        return Vec::new();
    }
    let reach = u32::from(sq) + max_move(s, soldier);
    let mut out: Vec<Dest> = (sq + 1..=LAST)
        .filter(|&q| u32::from(q) <= reach && room(s, q))
        .map(Dest::Square)
        .collect();
    if reach > u32::from(LAST) {
        out.push(Dest::Barracks);
    }
    out
}

/// [R][K] The soldiers `seat` still has to move this turn: free, not moved
/// yet, with somewhere to go. Ascending numbers.
pub fn to_move(s: &State, seat: Seat) -> Vec<u8> {
    let mut out: Vec<u8> = (1..=LAST)
        .filter_map(|sq| s.stacks[usize::from(sq)].last())
        .filter(|x| x.seat == seat && !s.moved.contains(&x.number))
        .filter(|&&x| !destinations(s, x).is_empty())
        .map(|x| x.number)
        .collect();
    out.sort_unstable();
    out
}

/// [R] Barracks places plus gray pieces, per seat.
pub fn scores(s: &State) -> Vec<u32> {
    let mut out = vec![0u32; usize::from(s.players)];
    for (i, place) in s.barracks.iter().enumerate() {
        if let Some(x) = place {
            out[x.seat.index()] += i as u32 + 1;
        }
    }
    for (k, owner) in s.grays.iter().enumerate() {
        if let Some(seat) = owner {
            out[seat.index()] += u32::from(GRAYS[k].1);
        }
    }
    out
}
