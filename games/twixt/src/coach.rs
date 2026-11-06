//! The coach's numbers, all from twixtbot's network: win probabilities,
//! the grade of a move by the win probability it gave away, the policy's
//! favourite holes, and the review of a finished game.

use ludo_core::Illegal;
use serde::Serialize;

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
    /// The side to move's win probability.
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

pub fn advise(net: &Net, moves: &[TwixtAction]) -> Result<Advice, Illegal> {
    let s = state(moves)?;
    if s.result.is_some() {
        return Err(Illegal::new("La partie est finie."));
    }
    let me = s.board.turn;
    let heat = policy(net, &s.board);
    let mut ranked = heat.clone();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
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
    let last = match moves.len() {
        0 | 1 => None,
        n => {
            let ply = n - 2;
            let before = win_for(net, &state(&moves[..ply])?, me);
            let after = win_for(net, &state(&moves[..=ply])?, me);
            let loss = (before - after).max(0.0);
            Some(LastMove {
                ply,
                loss,
                grade: grade(loss),
            })
        }
    };
    Ok(Advice {
        win: win_for(net, &s, me),
        best,
        heat,
        last,
    })
}

/// Red's win probability after the first `ply` moves.
pub fn red_win_at(net: &Net, moves: &[TwixtAction], ply: usize) -> Result<f32, Illegal> {
    let upto = moves
        .get(..ply)
        .ok_or_else(|| Illegal::new("Coup hors de la partie."))?;
    Ok(win_for(net, &state(upto)?, Color::Red))
}

/// Grades every move from the curve of Red's win probability
/// (`red_win_at` for ply 0..=moves.len()).
pub fn review(moves: &[TwixtAction], curve: &[f32]) -> Result<Review, Illegal> {
    state(moves)?;
    if curve.len() != moves.len() + 1 {
        return Err(Illegal::new("Courbe incomplète."));
    }
    let win = |k: usize, c: Color| match c {
        Color::Red => curve[k],
        Color::Black => 1.0 - curve[k],
    };
    let losses: Vec<f32> = (1..curve.len())
        .map(|k| (win(k - 1, mover(k - 1)) - win(k, mover(k - 1))).max(0.0))
        .collect();
    let grades = std::iter::once(None)
        .chain(losses.iter().map(|&l| Some(grade(l))))
        .collect();
    let mut key: Vec<usize> = (1..curve.len())
        .filter(|&k| losses[k - 1] >= KEY_LOSS)
        .collect();
    key.sort_by(|&a, &b| losses[b - 1].total_cmp(&losses[a - 1]));
    key.truncate(KEY_MOMENTS);
    key.sort_unstable();
    Ok(Review { grades, key })
}
