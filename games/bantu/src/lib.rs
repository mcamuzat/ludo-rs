//! Bantu (Parker Brothers, 1955): a race without dice, where a piece moves
//! as far as the number it bears, or as all the numbers in its row together.
//! Rules and their sources: RULES.md.

pub mod board;
mod heuristics;
mod moves;
pub mod testing;
mod view;

pub use moves::{destinations, is_protected, move_length, progress};
pub use view::View;

use ludo_core::{
    Determinize, Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use board::Cell;

pub struct Bantu;

/// ludo-rs addition, not in [R]: the game stops after this many rounds of
/// the table without progress (a piece taken, finished, or nearer the
/// finish than it has been since it last left its start square).
pub const QUIET_ROUNDS: u32 = 30;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// [R] Short game: pieces 1 to 3 only.
    pub short: bool,
}

impl Config {
    pub fn pieces(&self) -> u8 {
        if self.short {
            3
        } else {
            4
        }
    }
}

/// Where a piece is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Spot {
    /// On its start square, in the centre.
    Home,
    On {
        cell: Cell,
    },
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dest {
    Cell(Cell),
    Finish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Piece `piece` (its number) of the player to move goes to `to`.
    Move { piece: u8, to: Dest },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    /// [R] All its pieces in the finish first.
    Won(Seat),
    /// No piece taken nor finished for `QUIET_ROUNDS` rounds.
    QuietLimit,
    /// Nobody can move.
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastMove {
    pub seat: Seat,
    pub piece: u8,
    pub from: Spot,
    pub to: Spot,
    /// The piece sent back to its start square, if any.
    pub captured: Option<(Seat, u8)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub players: u8,
    /// `pieces[seat][number - 1]`.
    pub pieces: Vec<Vec<Spot>>,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// Turns (moves and passes) since the last progress.
    pub quiet: u32,
    /// `best[seat][number - 1]`: the fewest steps to the finish the piece
    /// has stood at since it last left its start square (`u8::MAX` at home).
    pub best: Vec<Vec<u8>>,
    /// Moves played since the start.
    pub moves: u32,
    pub last: Option<LastMove>,
    pub end: Option<End>,
}

impl State {
    fn has_move(&self, seat: Seat) -> bool {
        (1..=self.config.pieces()).any(|p| !destinations(self, seat, p).is_empty())
    }

    /// The next seat that can move after `from`, each seat passed counting
    /// as a quiet turn; ends the game on the quiet limit or when nobody can
    /// move.
    fn pass_turn(&mut self, from: Seat) {
        for k in 1..=self.players {
            if self.quiet >= QUIET_ROUNDS * u32::from(self.players) {
                self.end_game(End::QuietLimit);
                return;
            }
            let seat = Seat((from.0 + k) % self.players);
            if self.has_move(seat) {
                self.turn = Some(seat);
                return;
            }
            // [R] A player who cannot move loses the turn.
            self.quiet += 1;
        }
        self.end_game(End::Blocked);
    }

    fn end_game(&mut self, end: End) {
        self.end = Some(end);
        self.turn = None;
    }
}

/// Ranks: most pieces finished first, then fewest steps left; ties share.
pub(crate) fn ranks(s: &State) -> Vec<u8> {
    let keys: Vec<(u8, u32)> = (0..s.players).map(|i| progress(s, Seat(i))).collect();
    keys.iter()
        .map(|&(done, left)| {
            keys.iter()
                .filter(|&&(d, l)| d > done || (d == done && l < left))
                .count() as u8
        })
        .collect()
}

impl Game for Bantu {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "bantu",
            name: "Bantu",
            min_players: 2,
            max_players: 4,
            hidden_info: false,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "short",
            kind: OptionKind::Toggle { default: false },
        }]
    }

    fn new(config: &Config, players: u8, _seed: u64) -> State {
        State {
            config: config.clone(),
            players,
            pieces: vec![vec![Spot::Home; usize::from(config.pieces())]; usize::from(players)],
            best: vec![vec![u8::MAX; usize::from(config.pieces())]; usize::from(players)],
            // [R] The first player is chosen: seat 0.
            turn: Some(Seat(0)),
            quiet: 0,
            moves: 0,
            last: None,
            end: None,
        }
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != Some(seat) {
            return Vec::new();
        }
        (1..=s.config.pieces())
            .flat_map(|piece| {
                destinations(s, seat, piece)
                    .into_iter()
                    .map(move |to| Action::Move { piece, to })
            })
            .collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.turn.is_none() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Ce pion ne peut pas aller là."));
        }
        let Action::Move { piece, to } = *action;
        let from = s.pieces[seat.index()][usize::from(piece) - 1];
        let mut captured = None;
        let best = &mut s.best[seat.index()][usize::from(piece) - 1];
        let mut progress = false;
        let spot = match to {
            Dest::Finish => {
                progress = true;
                Spot::Finished
            }
            Dest::Cell(cell) => {
                let d = board::distance_to_finish(cell);
                if d < *best {
                    *best = d;
                    progress = true;
                }
                // [R] The piece taken goes back to its start square.
                if let Some((owner, number)) = s.occupant(cell) {
                    s.pieces[owner.index()][usize::from(number) - 1] = Spot::Home;
                    s.best[owner.index()][usize::from(number) - 1] = u8::MAX;
                    captured = Some((owner, number));
                    progress = true;
                }
                Spot::On { cell }
            }
        };
        s.pieces[seat.index()][usize::from(piece) - 1] = spot;
        s.last = Some(LastMove {
            seat,
            piece,
            from,
            to: spot,
            captured,
        });
        s.moves += 1;
        s.quiet = if progress { 0 } else { s.quiet + 1 };
        if s.pieces[seat.index()].iter().all(|&p| p == Spot::Finished) {
            s.end_game(End::Won(seat));
        } else {
            s.pass_turn(seat);
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.end.map(|_| Outcome::Ranked(ranks(s)))
    }

    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        Self::legal_actions(s, seat).into_iter().next()
    }
}

impl Determinize for Bantu {
    /// Perfect information: nothing to redraw.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, _rng: &mut R) -> State {
        s.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No real position seems to block everyone (a search over millions of
    /// random ones found none: an arrow cell always lets a piece in), so the
    /// guard is checked on a state where no seat has a piece left to move.
    #[test]
    fn nobody_can_move_ends_the_game() {
        let mut s = testing::position(2, &[], 0);
        s.pass_turn(Seat(0));
        assert_eq!(s.end, Some(End::Blocked));
        assert_eq!(s.turn, None);
        assert_eq!(s.quiet, 2);
    }
}
