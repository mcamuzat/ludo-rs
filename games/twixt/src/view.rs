use serde::Serialize;

use crate::{Color, Point, TwixtAction, TwixtState};

/// How the game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Finish {
    Won(Color),
    /// Nobody can join their borders any more.
    Draw,
}

/// Everything is public in TwixT: the same view for everyone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwixtView {
    pub pegs: Vec<(Point, Color)>,
    pub links: Vec<(Color, Point, Point)>,
    pub turn: Color,
    pub can_swap: bool,
    /// The whole game, for the coach and the review.
    pub moves: Vec<TwixtAction>,
    /// The peg the last move put (after a swap: the reflected peg).
    pub last: Option<Point>,
    pub result: Option<Finish>,
    /// The winner's group joining its two borders.
    pub chain: Option<Vec<Point>>,
}

impl TwixtView {
    pub fn of(s: &TwixtState) -> TwixtView {
        let pegs = [Color::Red, Color::Black]
            .into_iter()
            .flat_map(|c| s.board.pegs_of(c).into_iter().map(move |p| (p, c)))
            .collect();
        let last = match s.moves.last() {
            Some(TwixtAction::Place(p)) => Some(*p),
            Some(TwixtAction::Swap) => s.board.pegs_of(Color::Black).first().copied(),
            None => None,
        };
        TwixtView {
            pegs,
            links: s.board.link_list(),
            turn: s.board.turn,
            can_swap: s.result.is_none() && s.moves.len() == 1,
            moves: s.moves.clone(),
            last,
            result: s.result.map(|r| r.map_or(Finish::Draw, Finish::Won)),
            chain: s.result.flatten().and_then(|_| s.board.winning_chain()),
        }
    }
}
