//! The coach's numbers, all from twixtbot's network: win probabilities,
//! the grade of a move by the win probability it gave away, the policy's
//! favourite holes, and the review of a finished game.

use ludo_core::Illegal;
use serde::{Deserialize, Serialize};

use crate::encode::{encode, win_probability};
use crate::mcts::policy;
use crate::net::Net;
use crate::{Color, Point, TwixtAction, TwixtState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Grade {
    /// Meilleur
    Best,
    /// Bon
    Good,
    /// Imprécision
    Inaccuracy,
    /// Erreur
    Mistake,
    /// Gaffe
    Blunder,
}

/// The grade of a move that gave away `loss` of the mover's win probability.
pub fn grade(loss: f32) -> Grade {
    match loss {
        l if l <= 0.01 => Grade::Best,
        l if l <= 0.05 => Grade::Good,
        l if l <= 0.10 => Grade::Inaccuracy,
        l if l <= 0.20 => Grade::Mistake,
        _ => Grade::Blunder,
    }
}

/// Losses from this much up are key moments of the review.
const KEY_LOSS: f32 = 0.10;
const KEY_MOMENTS: usize = 3;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub at: Point,
    /// The mover's win probability after playing it.
    pub win: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastMove {
    /// The move's index in the game.
    pub ply: usize,
    pub loss: f32,
    pub grade: Grade,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advice {
    /// The side to move's win probability with the best of `best`.
    pub win: f32,
    /// The policy's three favourite holes, best first by `win`.
    pub best: Vec<Candidate>,
    /// The policy's probability of every legal hole.
    pub heat: Vec<(Point, f32)>,
    /// The side to move's previous move.
    pub last: Option<LastMove>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    /// For each position after k moves, the grade of the k-th move
    /// (`None` for the starting position).
    pub grades: Vec<Option<Grade>>,
    /// The plies (positions) of the largest losses, in game order.
    pub key: Vec<usize>,
}

/// The colour that plays move number `ply`: moves alternate, the swap
/// included.
fn mover(ply: usize) -> Color {
    if ply.is_multiple_of(2) {
        Color::Red
    } else {
        Color::Black
    }
}

/// `c`'s win probability in `s`: the result if the game is over.
fn win_for(net: &Net, s: &TwixtState, c: Color) -> f32 {
    match s.result {
        Some(Some(w)) => f32::from(u8::from(w == c)),
        Some(None) => 0.5,
        None => {
            let p = win_probability(&net.eval(&encode(&s.board)).value);
            if s.board.turn == c {
                p
            } else {
                1.0 - p
            }
        }
    }
}

fn state(moves: &[TwixtAction]) -> Result<TwixtState, Illegal> {
    TwixtState::from_moves(moves)
}

/// `me`'s win after each of the policy's three favourite holes in `s`
/// (`me` to move), best first.
fn top_children(net: &Net, s: &TwixtState, ranked: &[(Point, f32)]) -> Vec<Candidate> {
    let me = s.board.turn;
    let mut best: Vec<Candidate> = ranked
        .iter()
        .take(3)
        .map(|&(at, _)| {
            let mut child = s.clone();
            child.play(&TwixtAction::Place(at)).expect("a legal hole");
            Candidate {
                at,
                win: win_for(net, &child, me),
            }
        })
        .collect();
    best.sort_by(|a, b| b.win.total_cmp(&a.win));
    best
}

fn ranked_policy(net: &Net, s: &TwixtState) -> Vec<(Point, f32)> {
    let mut ranked = policy(net, &s.board);
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked
}

/// The best win the side to move can get in `s` among the policy's three
/// favourite holes: what a move is graded against. Every value compared is
/// measured the same way, on the position after the move, so the edge of
/// having the move never counts as a loss.
fn best_win(net: &Net, s: &TwixtState) -> f32 {
    top_children(net, s, &ranked_policy(net, s))
        .first()
        .map_or(0.0, |c| c.win)
}

pub fn advise(net: &Net, moves: &[TwixtAction]) -> Result<Advice, Illegal> {
    let s = state(moves)?;
    if s.result.is_some() {
        return Err(Illegal::new("La partie est finie."));
    }
    let me = s.board.turn;
    let heat = policy(net, &s.board);
    let mut ranked = heat.clone();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let best = top_children(net, &s, &ranked);
    let last = match moves.len() {
        0 | 1 => None,
        n => {
            let ply = n - 2;
            let before = state(&moves[..ply])?;
            let played = win_for(net, &state(&moves[..=ply])?, me);
            let loss = (best_win(net, &before) - played).max(0.0);
            Some(LastMove {
                ply,
                loss,
                grade: grade(loss),
            })
        }
    };
    Ok(Advice {
        win: best.first().map_or(0.5, |c| c.win),
        best,
        heat,
        last,
    })
}

/// What the review needs of the position after `ply` moves.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPoint {
    /// Red's win probability here (the curve).
    pub red_win: f32,
    /// The side to move's best win among the policy's three favourite
    /// holes; `None` once the game is over.
    pub best: Option<f32>,
}

/// The review's numbers for the position after the first `ply` moves.
pub fn review_point(net: &Net, moves: &[TwixtAction], ply: usize) -> Result<ReviewPoint, Illegal> {
    let upto = moves
        .get(..ply)
        .ok_or_else(|| Illegal::new("Coup hors de la partie."))?;
    let s = state(upto)?;
    Ok(ReviewPoint {
        red_win: win_for(net, &s, Color::Red),
        best: s.result.is_none().then(|| best_win(net, &s)),
    })
}

/// Grades every move: the mover's best win before it, against the win
/// the move gave (`review_point` for ply 0..=moves.len()).
pub fn review(moves: &[TwixtAction], points: &[ReviewPoint]) -> Result<Review, Illegal> {
    state(moves)?;
    if points.len() != moves.len() + 1 {
        return Err(Illegal::new("Courbe incomplète."));
    }
    let win = |k: usize, c: Color| match c {
        Color::Red => points[k].red_win,
        Color::Black => 1.0 - points[k].red_win,
    };
    let losses: Vec<f32> = (1..points.len())
        .map(|k| {
            let m = mover(k - 1);
            let best = points[k - 1].best.unwrap_or(win(k - 1, m));
            (best - win(k, m)).max(0.0)
        })
        .collect();
    let grades = std::iter::once(None)
        .chain(losses.iter().map(|&l| Some(grade(l))))
        .collect();
    let mut key: Vec<usize> = (1..points.len())
        .filter(|&k| losses[k - 1] >= KEY_LOSS)
        .collect();
    key.sort_by(|&a, &b| losses[b - 1].total_cmp(&losses[a - 1]));
    key.truncate(KEY_MOMENTS);
    key.sort_unstable();
    Ok(Review { grades, key })
}
