use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::lines::Alignment;
use crate::{score, team_of, End, State, TurnRecord, QUIET_ROUNDS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Pawn {
    pub team: u8,
    /// [R] Turned over: it belongs to a line already paid.
    pub flipped: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamView {
    pub seats: Vec<Seat>,
    pub pawns_left: u8,
    /// Lines paid plus 10 points per pawn not played.
    pub score: u32,
}

/// Everything but the rolls to come is public. The moves are left out on
/// purpose [U]: the board shows no reachable cell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: u8,
    pub pawns: Vec<Option<Pawn>>,
    pub dice: [u8; 3],
    pub roller: Seat,
    /// Who plays now (`None` once the game is over).
    pub turn: Option<Seat>,
    /// Who still plays this roll, in order.
    pub queue: Vec<Seat>,
    pub teams: Vec<TeamView>,
    pub alignments: Vec<Alignment>,
    pub last: Vec<Option<TurnRecord>>,
    /// Rounds of the table without a pawn placed left before the game stops.
    pub quiet_left: u32,
    /// Turns played since the start (moves and passes): a new decision.
    pub turns: u32,
    pub end: Option<End>,
    /// The teams ranked first once the game is over (both on a tie).
    pub winners: Vec<u8>,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        let players = u32::from(s.players);
        let quiet_turns_left = (QUIET_ROUNDS * players).saturating_sub(s.quiet);
        View {
            players: s.players,
            pawns: s
                .owner
                .iter()
                .zip(&s.counted)
                .map(|(owner, counted)| {
                    owner.map(|team| Pawn {
                        team,
                        flipped: counted.iter().any(|&c| c),
                    })
                })
                .collect(),
            dice: s.dice,
            roller: s.roller,
            turn: s.end.is_none().then(|| s.queue.first().copied()).flatten(),
            queue: s.queue.clone(),
            teams: (0..2u8)
                .map(|team| TeamView {
                    seats: (0..s.players)
                        .map(Seat)
                        .filter(|&seat| team_of(s.players, seat) == team)
                        .collect(),
                    pawns_left: s.pawns_left[usize::from(team)],
                    score: score(s, team),
                })
                .collect(),
            alignments: s.alignments.clone(),
            last: s.last.clone(),
            quiet_left: quiet_turns_left.div_ceil(players),
            turns: s.turns,
            end: s.end,
            winners: if s.end.is_some() {
                let best = score(s, 0).max(score(s, 1));
                (0..2).filter(|&team| score(s, team) == best).collect()
            } else {
                Vec::new()
            },
        }
    }
}
