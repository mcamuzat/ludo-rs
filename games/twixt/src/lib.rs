//! TwixT (Alex Randolph), Little Golem "PP" rules: own links may cross,
//! links are never removed, swap on the second player's first move. Rules
//! and sources: RULES.md ([LG] the rules, [TB] twixtbot).

pub mod board;
mod view;

pub use board::{all_points, replay, Board, Color, Point, TwixtAction, SIZE};
pub use view::TwixtView;

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct Twixt;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {}

/// What is saved: the moves. The board is rebuilt from them.
#[derive(Serialize, Deserialize)]
struct Saved {
    moves: Vec<TwixtAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "Saved", into = "Saved")]
pub struct TwixtState {
    pub board: Board,
    pub moves: Vec<TwixtAction>,
    /// `Some(Some(c))`: `c` won; `Some(None)`: draw; `None`: still playing.
    pub result: Option<Option<Color>>,
}

impl From<TwixtState> for Saved {
    fn from(s: TwixtState) -> Saved {
        Saved { moves: s.moves }
    }
}

impl TryFrom<Saved> for TwixtState {
    type Error = Illegal;

    fn try_from(saved: Saved) -> Result<TwixtState, Illegal> {
        TwixtState::from_moves(&saved.moves)
    }
}

impl TwixtState {
    pub fn new() -> Self {
        TwixtState {
            board: Board::new(),
            moves: Vec::new(),
            result: None,
        }
    }

    pub fn from_moves(moves: &[TwixtAction]) -> Result<Self, Illegal> {
        let mut s = TwixtState::new();
        for m in moves {
            if s.result.is_some() {
                return Err(Illegal::new("La partie est finie."));
            }
            s.play(m)?;
        }
        Ok(s)
    }

    /// Plays `a` for the colour to move; on error nothing changes.
    pub fn play(&mut self, a: &TwixtAction) -> Result<(), Illegal> {
        let mut board = self.board.clone();
        board.play(a, self.moves.len())?;
        self.board = board;
        self.moves.push(a.clone());
        if let Some(c) = self.board.winner() {
            self.result = Some(Some(c));
        } else if !self.board.can_still_connect(Color::Red)
            && !self.board.can_still_connect(Color::Black)
        {
            // Site's choice, as [TB]: nobody can connect any more.
            self.result = Some(None);
        }
        Ok(())
    }

    pub fn to_move(&self) -> Option<Seat> {
        self.result.is_none().then(|| Seat(self.board.turn.seat()))
    }

    /// The legal actions of the colour to move.
    pub fn actions(&self) -> Vec<TwixtAction> {
        if self.result.is_some() {
            return Vec::new();
        }
        let mut out: Vec<TwixtAction> = all_points()
            .filter(|&p| self.board.legal(p))
            .map(TwixtAction::Place)
            .collect();
        if self.moves.len() == 1 {
            out.push(TwixtAction::Swap);
        }
        out
    }
}

impl Default for TwixtState {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for Twixt {
    type State = TwixtState;
    type Action = TwixtAction;
    type View = TwixtView;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "twixt",
            name: "TwixT",
            min_players: 2,
            max_players: 2,
            hidden_info: false,
            clock: false,
        }
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if players != 2 {
            return Err(Illegal::new("TwixT se joue à 2 joueurs."));
        }
        Ok(())
    }

    fn new(_config: &Config, _players: u8, _seed: u64) -> TwixtState {
        TwixtState::new()
    }

    fn pending(s: &TwixtState) -> Pending {
        s.to_move().map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &TwixtState, seat: Seat) -> Vec<TwixtAction> {
        if s.to_move() != Some(seat) {
            return Vec::new();
        }
        s.actions()
    }

    fn apply(s: &mut TwixtState, seat: Seat, action: &TwixtAction) -> Result<(), Illegal> {
        if s.to_move() != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        s.play(action)
    }

    fn view(s: &TwixtState, _viewer: Viewer) -> TwixtView {
        TwixtView::of(s)
    }

    fn outcome(s: &TwixtState) -> Option<Outcome> {
        s.result.map(|r| {
            Outcome::Ranked(match r {
                Some(Color::Red) => vec![0, 1],
                Some(Color::Black) => vec![1, 0],
                None => vec![0, 0],
            })
        })
    }
}

impl Determinize for Twixt {
    fn determinize<R: Rng + ?Sized>(s: &TwixtState, _observer: Seat, _rng: &mut R) -> TwixtState {
        s.clone()
    }
}
