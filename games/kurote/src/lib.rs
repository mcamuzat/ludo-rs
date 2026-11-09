//! Kurote (Arvi Teikari): two bogs of wizards teleport around a 5×5 board;
//! a wizard with three enemies around it is captured, and the first to
//! capture three wins. Rules and their source: RULES.md, where [K] is the
//! rulebook.

pub mod board;
mod heuristics;
mod view;

pub use board::{cell, col, neighbours, row, CELLS, SIZE};
pub use view::View;

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct Kurote;

/// [K] 7 wizards each: 5 on the board, 2 on the side.
pub const RESERVE: u8 = 2;
/// [K] One may place a wizard from the side while fewer than 5 are on the board.
pub const FULL: usize = 5;
/// [K] A wizard with 3 or more opposing wizards around it is captured, and
/// no wizard may arrive where 3 or more would surround it.
pub const SURROUND: usize = 3;
/// [K] Capturing 3 of the opponent's wizards wins.
pub const TO_WIN: u8 = 3;
/// ❓ The site's limit: a draw after this many turns (both players) in a
/// row without a capture.
pub const QUIET_LIMIT: u16 = 80;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

/// A wizard on the board. Seat 0 is green (bottom), seat 1 yellow (top).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wizard {
    pub owner: u8,
    pub exhausted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// [K] Teleport a non-exhausted wizard.
    Move { from: u8, to: u8 },
    /// [K] Bring a wizard from the side onto one's own end of the board.
    Place { to: u8 },
    /// ❓ Only when nothing else is possible.
    Pass,
}

/// How the game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    /// [K] `winner` captured 3 wizards.
    Captures { winner: Seat },
    /// ❓ Both players in turn could do nothing: a draw.
    Blocked,
    /// ❓ `QUIET_LIMIT` turns without a capture: a draw.
    Limit,
}

/// The last action played, for the screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Last {
    pub seat: Seat,
    pub action: Action,
    /// The cells whose wizards it captured.
    pub captured: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub cells: Vec<Option<Wizard>>,
    /// Each seat's wizards still on the side.
    pub reserve: [u8; 2],
    /// The wizards each seat has captured.
    pub captured: [u8; 2],
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// Turns in a row without a capture.
    pub quiet: u16,
    /// Passes in a row.
    pub passes: u8,
    pub last: Option<Last>,
    pub end: Option<End>,
}

/// [K] Each player's end of the board: green the bottom row, yellow the top.
pub fn home_row(seat: Seat) -> u8 {
    if seat.0 == 0 {
        SIZE - 1
    } else {
        0
    }
}

fn other(seat: Seat) -> Seat {
    Seat(1 - seat.0)
}

impl State {
    /// [K] The starting position of the rulebook's picture.
    pub fn start() -> Self {
        let mut cells = vec![None; CELLS];
        let fresh = |owner| {
            Some(Wizard {
                owner,
                exhausted: false,
            })
        };
        for (owner, back, front) in [(1u8, 0u8, 1u8), (0, 4, 3)] {
            for c in 1..=3 {
                cells[usize::from(cell(back, c))] = fresh(owner);
            }
            for c in [0, 4] {
                cells[usize::from(cell(front, c))] = fresh(owner);
            }
        }
        Self {
            cells,
            reserve: [RESERVE; 2],
            captured: [0; 2],
            // ❓ The rulebook does not say who starts: seat 0 (green).
            turn: Some(Seat(0)),
            quiet: 0,
            passes: 0,
            last: None,
            end: None,
        }
    }

    pub fn owner(&self, c: u8) -> Option<u8> {
        self.cells[usize::from(c)].map(|w| w.owner)
    }

    /// The wizards of `seat` among the 8 cells around `c`.
    pub fn around(&self, c: u8, seat: Seat) -> usize {
        neighbours(c)
            .filter(|&n| self.owner(n) == Some(seat.0))
            .count()
    }

    pub fn on_board(&self, seat: Seat) -> usize {
        self.cells
            .iter()
            .filter(|w| w.is_some_and(|w| w.owner == seat.0))
            .count()
    }

    /// [K] A wizard of `seat` may arrive on `c` when it is empty and fewer
    /// than 3 opposing wizards surround it.
    fn open_to(&self, c: u8, seat: Seat) -> bool {
        self.cells[usize::from(c)].is_none() && self.around(c, other(seat)) < SURROUND
    }

    /// [K] Where the wizard on `from` may teleport: an open cell next to
    /// another wizard of its player.
    pub fn destinations(&self, from: u8) -> Vec<u8> {
        let Some(w) = self.cells[usize::from(from)] else {
            return Vec::new();
        };
        let seat = Seat(w.owner);
        (0..CELLS as u8)
            .filter(|&to| {
                self.open_to(to, seat)
                    && neighbours(to).any(|n| n != from && self.owner(n) == Some(w.owner))
            })
            .collect()
    }

    /// Every action but `Pass`.
    fn plays(&self, seat: Seat) -> Vec<Action> {
        let mut out = Vec::new();
        for from in 0..CELLS as u8 {
            if self.cells[usize::from(from)].is_some_and(|w| w.owner == seat.0 && !w.exhausted) {
                out.extend(
                    self.destinations(from)
                        .into_iter()
                        .map(|to| Action::Move { from, to }),
                );
            }
        }
        // [K] Fewer than 5 on the board: place one from the side on an open
        // cell of one's own end.
        if self.reserve[seat.index()] > 0 && self.on_board(seat) < FULL {
            let r = home_row(seat);
            out.extend(
                (0..SIZE)
                    .map(|c| cell(r, c))
                    .filter(|&to| self.open_to(to, seat))
                    .map(|to| Action::Place { to }),
            );
        }
        out
    }

    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.turn != Some(seat) {
            return Vec::new();
        }
        let plays = self.plays(seat);
        if plays.is_empty() {
            vec![Action::Pass]
        } else {
            plays
        }
    }

    /// The opposing wizards `seat` would capture: those with 3 or more of
    /// its wizards around them.
    pub fn captures(&self, seat: Seat) -> Vec<u8> {
        (0..CELLS as u8)
            .filter(|&c| self.owner(c) == Some(other(seat).0) && self.around(c, seat) >= SURROUND)
            .collect()
    }

    /// Plays a legal `action` for the seat to move.
    fn play(&mut self, seat: Seat, action: Action) {
        let mut captured = Vec::new();
        match action {
            Action::Pass => self.passes += 1,
            Action::Move { from, to } => {
                self.cells[usize::from(from)] = None;
                // [K] Once a wizard has moved, it is exhausted.
                self.cells[usize::from(to)] = Some(Wizard {
                    owner: seat.0,
                    exhausted: true,
                });
            }
            Action::Place { to } => {
                self.reserve[seat.index()] -= 1;
                // [K] A wizard placed from the side arrives exhausted.
                self.cells[usize::from(to)] = Some(Wizard {
                    owner: seat.0,
                    exhausted: true,
                });
            }
        }
        if action != Action::Pass {
            self.passes = 0;
            // [K] Exhausted or not, 3 wizards around capture. Only the
            // opponent's wizards can be caught: an arrival never brings
            // a wizard of the mover next to 3 opponents, and nothing else
            // around the mover's wizards changes.
            captured = self.captures(seat);
            for &c in &captured {
                self.cells[usize::from(c)] = None;
            }
            self.captured[seat.index()] += captured.len() as u8;
        }
        self.quiet = if captured.is_empty() {
            self.quiet + 1
        } else {
            0
        };
        // [K] When all of a player's wizards are exhausted, they may
        // unexhaust them all. ❓ Done at once: staying exhausted only
        // takes moves away.
        for s in [Seat(0), Seat(1)] {
            let mine = || self.cells.iter().flatten().filter(|w| w.owner == s.0);
            if mine().count() > 0 && mine().all(|w| w.exhausted) {
                self.cells
                    .iter_mut()
                    .flatten()
                    .filter(|w| w.owner == s.0)
                    .for_each(|w| w.exhausted = false);
            }
        }
        self.last = Some(Last {
            seat,
            action,
            captured,
        });
        self.end = if self.captured[seat.index()] >= TO_WIN {
            Some(End::Captures { winner: seat })
        } else if self.passes >= 2 {
            Some(End::Blocked)
        } else if self.quiet >= QUIET_LIMIT {
            Some(End::Limit)
        } else {
            None
        };
        self.turn = self.end.is_none().then(|| other(seat));
    }
}

impl Game for Kurote {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "kurote",
            name: "Kurote",
            min_players: 2,
            max_players: 2,
            hidden_info: false,
            clock: true,
        }
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if players != 2 {
            return Err(Illegal::new("Kurote se joue à 2 joueurs."));
        }
        Ok(())
    }

    fn new(_config: &Config, _players: u8, _seed: u64) -> State {
        State::start()
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.turn.is_none() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        let in_board = |c: u8| usize::from(c) < CELLS;
        match *action {
            Action::Move { from, to } if !in_board(from) || !in_board(to) => {
                return Err(Illegal::new("Cette case n'existe pas."));
            }
            Action::Place { to } if !in_board(to) => {
                return Err(Illegal::new("Cette case n'existe pas."));
            }
            _ => {}
        }
        if !s.moves(seat).contains(action) {
            return Err(Illegal::new(match *action {
                Action::Move { from, .. }
                    if s.cells[usize::from(from)]
                        .is_none_or(|w| w.owner != seat.0) =>
                {
                    "Ce n'est pas un de tes sorciers."
                }
                Action::Move { from, .. }
                    if s.cells[usize::from(from)].is_some_and(|w| w.exhausted) =>
                {
                    "Ce sorcier est épuisé."
                }
                Action::Move { .. } => {
                    "Il faut arriver sur une case vide, voisine d'un autre de tes sorciers, avec moins de 3 sorciers adverses autour."
                }
                Action::Place { .. } => {
                    "On ne place un sorcier de réserve que sur une case libre de sa première rangée, avec moins de 5 sorciers en jeu."
                }
                Action::Pass => "On ne passe que si l'on ne peut rien faire.",
            }));
        }
        s.play(seat, *action);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        Some(Outcome::Ranked(match s.end? {
            End::Captures { winner } if winner.0 == 0 => vec![0, 1],
            End::Captures { .. } => vec![1, 0],
            End::Blocked | End::Limit => vec![0, 0],
        }))
    }

    /// A capture if there is one, else the first legal action.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let moves = s.moves(seat);
        heuristics::best_capture(s, seat, &moves).or_else(|| moves.first().copied())
    }
}

impl Determinize for Kurote {
    /// Perfect information: identity.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, _rng: &mut R) -> State {
        s.clone()
    }
}
