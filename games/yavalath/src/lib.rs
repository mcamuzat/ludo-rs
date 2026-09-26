//! Yavalath (Cameron Browne / Ludi, 2007). Rules and their sources: RULES.md.

mod board;
mod heuristics;
mod view;

pub use board::{all_cells, Board, Cell, Color, Effect, CELL_COUNT, DIRECTIONS, RADIUS};
pub use view::{CellView, View};

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct Yavalath;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Place(Cell),
    /// [R] Black's first move, 2 players only: take White's stone and colour.
    Swap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub board: Board,
    pub players: u8,
    /// Colour of each seat (index = seat). A swap exchanges seats 0 and 1.
    pub seat_colors: Vec<Color>,
    /// Colour to move; `None` once the game is over.
    pub current: Option<Color>,
    /// Eliminated seats, in elimination order.
    pub eliminated: Vec<Seat>,
    pub winner: Option<Seat>,
    pub swapped: bool,
    pub stones: u16,
    pub last_move: Option<Cell>,
    pub lines: Vec<Vec<Cell>>,
}

impl State {
    pub fn seat_of(&self, color: Color) -> Seat {
        let i = self
            .seat_colors
            .iter()
            .position(|&c| c == color)
            .expect("colour in play");
        Seat(i as u8)
    }

    pub fn is_alive(&self, seat: Seat) -> bool {
        !self.eliminated.contains(&seat)
    }

    pub fn alive(&self) -> Vec<Seat> {
        (0..self.players)
            .map(Seat)
            .filter(|&s| self.is_alive(s))
            .collect()
    }

    /// ❓4: the next colour in turn order whose seat is still in the game.
    pub fn next_color(&self, color: Color) -> Color {
        let in_play = &Color::TURN_ORDER[..usize::from(self.players)];
        let start = in_play
            .iter()
            .position(|&c| c == color)
            .expect("colour in play");
        (1..=in_play.len())
            .map(|k| in_play[(start + k) % in_play.len()])
            .find(|&c| self.is_alive(self.seat_of(c)))
            .unwrap_or(color)
    }

    /// [R] 3 players: cells where the next player would win at once, which
    /// the player to move must block. Empty when the rule does not apply.
    pub fn forced_blocks(&self) -> Vec<Cell> {
        let Some(me) = self.current else {
            return Vec::new();
        };
        if self.players != 3 {
            return Vec::new();
        }
        let next = self.next_color(me);
        if next == me {
            return Vec::new();
        }
        self.board
            .empty_cells()
            .filter(|&c| self.board.effect(c, next) == Effect::Wins)
            .collect()
    }

    /// Cells the colour to move may play.
    pub fn legal_cells(&self) -> Vec<Cell> {
        let Some(me) = self.current else {
            return Vec::new();
        };
        let forced = self.forced_blocks();
        let empty = self.board.empty_cells();
        if forced.is_empty() {
            return empty.collect();
        }
        // ❓2: an immediate win is allowed even when it does not block.
        empty
            .filter(|&c| forced.contains(&c) || self.board.effect(c, me) == Effect::Wins)
            .collect()
    }

    /// [R] + ❓1: Black's first move, 2 players only.
    pub fn can_swap(&self) -> bool {
        self.players == 2 && self.stones == 1 && !self.swapped && self.current == Some(Color::Black)
    }

    fn place(&mut self, seat: Seat, color: Color, cell: Cell) {
        let effect = self.board.effect(cell, color);
        if effect != Effect::Neutral {
            self.lines.extend(self.board.deciding_lines(cell, color));
        }
        self.board.set(cell, Some(color));
        self.stones += 1;
        self.last_move = Some(cell);
        match effect {
            // [R] 4 or more in a row wins, even if the same move makes a 3.
            Effect::Wins => {
                self.winner = Some(seat);
                self.current = None;
                return;
            }
            // [R] 3 without 4: eliminated, the stones stay on the board.
            Effect::Loses => self.eliminated.push(seat),
            Effect::Neutral => {}
        }
        let alive = self.alive();
        if alive.len() == 1 {
            // [R] the last surviving player wins.
            self.winner = Some(alive[0]);
            self.current = None;
        } else if self.board.is_full() {
            // [R] full board: draw.
            self.current = None;
        } else {
            self.current = Some(self.next_color(color));
        }
    }
}

impl Game for Yavalath {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "yavalath",
            name: "Yavalath",
            min_players: 2,
            max_players: 3,
            hidden_info: false,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, _seed: u64) -> State {
        State {
            board: Board::default(),
            players,
            seat_colors: Color::TURN_ORDER[..usize::from(players)].to_vec(),
            current: Some(Color::White),
            eliminated: Vec::new(),
            winner: None,
            swapped: false,
            stones: 0,
            last_move: None,
            lines: Vec::new(),
        }
    }

    fn pending(s: &State) -> Pending {
        match s.current {
            Some(color) => Pending::one(s.seat_of(color)),
            None => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        let Some(color) = s.current else {
            return Vec::new();
        };
        if s.seat_of(color) != seat {
            return Vec::new();
        }
        let mut actions: Vec<Action> = s.legal_cells().into_iter().map(Action::Place).collect();
        if s.can_swap() {
            actions.push(Action::Swap);
        }
        actions
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        let Some(color) = s.current else {
            return Err(Illegal::new("La partie est terminée."));
        };
        if s.seat_of(color) != seat {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match *action {
            Action::Swap => {
                if !s.can_swap() {
                    return Err(Illegal::new(
                        "L'échange n'est possible qu'au premier coup de Noir, à 2 joueurs.",
                    ));
                }
                // The swapper takes White and its stone; the other seat becomes Black and moves.
                s.seat_colors.swap(0, 1);
                s.swapped = true;
            }
            Action::Place(cell) => {
                if !s.board.is_empty(cell) {
                    return Err(Illegal::new("Cette case est occupée ou hors du plateau."));
                }
                if !s.legal_cells().contains(&cell) {
                    return Err(Illegal::new(
                        "Tu dois bloquer la victoire du joueur suivant.",
                    ));
                }
                s.place(seat, color, cell);
            }
        }
        Ok(())
    }

    fn view(s: &State, _viewer: Viewer) -> View {
        View::of(s)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.current.is_some() {
            return None;
        }
        let mut ranks = vec![0u8; usize::from(s.players)];
        let alive = s.alive();
        // ❓6: winner first, other survivors tied next, then the eliminated, last eliminated first.
        let mut next = match s.winner {
            Some(winner) => {
                let others: Vec<Seat> = alive.into_iter().filter(|&x| x != winner).collect();
                for seat in &others {
                    ranks[seat.index()] = 1;
                }
                1 + others.len() as u8
            }
            None => alive.len() as u8,
        };
        for seat in s.eliminated.iter().rev() {
            ranks[seat.index()] = next;
            next += 1;
        }
        Some(Outcome::Ranked(ranks))
    }
}

impl Determinize for Yavalath {
    /// Perfect information: nothing to redraw.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, _rng: &mut R) -> State {
        s.clone()
    }
}
