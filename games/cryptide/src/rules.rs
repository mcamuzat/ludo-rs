//! The game: setup, questions, searches and penalty cubes [R p. 6–7].

use std::sync::OnceLock;

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::clues::{generate, HintTable, Mode, Puzzle};
use crate::map::{Cell, Mask, CELLS};
use crate::view::View;

pub struct Cryptide;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub mode: Mode,
}

/// [R p. 4] A disc: the habitat may be here; a cube: it may not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Answer {
    Disc,
    Cube,
}

/// The tokens on one space.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tokens {
    /// [R p. 4] At most one cube per space.
    pub cube: Option<Seat>,
    /// In the order they were laid.
    pub discs: Vec<Seat>,
}

impl Tokens {
    pub fn has(&self, seat: Seat) -> bool {
        self.cube == Some(seat) || self.discs.contains(&seat)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Phase {
    /// [R p. 6] Two cubes each, in turn order.
    Setup,
    /// Ask or search.
    Turn,
    /// [R p. 7] A search on a space that already has one's disc: one's
    /// disc goes on another space.
    Relocate {
        search: Cell,
    },
    /// [R p. 7] One made another player lay a cube: lay one of one's own.
    Penalty,
    Over,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// A cube on a space outside one's clue (setup, penalty).
    Cube(Cell),
    /// "Could the creature be here?"
    Ask { cell: Cell, target: Seat },
    /// Search a space of one's clue.
    Search(Cell),
    /// After a search on one's own disc: the disc goes here.
    Disc(Cell),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Why {
    Setup,
    Answer,
    Search,
    Penalty,
}

/// What happened, for the board to show. All public.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Placed {
        seat: Seat,
        cell: Cell,
        answer: Answer,
        why: Why,
    },
    Asked {
        by: Seat,
        target: Seat,
        cell: Cell,
    },
    Searched {
        by: Seat,
        cell: Cell,
    },
    Won {
        seat: Seat,
    },
}

/// The hints of the mode on this game's map, rebuilt after a reload.
#[derive(Clone, Debug, Default)]
pub struct TableCache(OnceLock<HintTable>);

impl PartialEq for TableCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for TableCache {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub mode: Mode,
    /// Hidden: the hints of the seats and the habitat are never in a view
    /// before the end.
    pub puzzle: Puzzle,
    /// One per space.
    pub tokens: Vec<Tokens>,
    pub phase: Phase,
    pub turn: Seat,
    pub first: Seat,
    /// Setup cubes still to lay.
    pub setup_left: u8,
    pub log: Vec<Event>,
    pub winner: Option<Seat>,
    #[serde(skip)]
    pub cache: TableCache,
}

impl State {
    /// The hints of the mode on this map.
    pub fn table(&self) -> &HintTable {
        self.cache
            .0
            .get_or_init(|| HintTable::new(&self.puzzle.board(), self.mode))
    }

    /// The spaces `seat`'s hint allows.
    pub fn hint_mask(&self, seat: Seat) -> Mask {
        let table = self.table();
        self.puzzle
            .hints
            .get(seat.index())
            .and_then(|h| table.index_of(h))
            .map_or(Mask::empty(), |i| table.masks[i])
    }

    /// [R p. 7] What `seat` answers about `cell`, honestly.
    pub fn answer(&self, seat: Seat, cell: Cell) -> Answer {
        if self.hint_mask(seat).contains(cell) {
            Answer::Disc
        } else {
            Answer::Cube
        }
    }

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    fn at(&self, cell: Cell) -> &Tokens {
        &self.tokens[cell.index()]
    }

    /// [R p. 6–7] A cube of `seat`'s on `cell`: outside its clue, no cube
    /// there yet, none of its own tokens there.
    pub fn may_cube(&self, seat: Seat, cell: Cell) -> bool {
        cell.is_valid()
            && !self.hint_mask(seat).contains(cell)
            && self.at(cell).cube.is_none()
            && !self.at(cell).has(seat)
    }

    /// [R p. 7] A disc of `seat`'s laid while searching: inside its clue,
    /// no cube, none of its own tokens there.
    fn may_disc(&self, seat: Seat, cell: Cell) -> bool {
        cell.is_valid()
            && self.hint_mask(seat).contains(cell)
            && self.at(cell).cube.is_none()
            && !self.at(cell).has(seat)
    }

    /// [R p. 7] No action on a space with a cube; [U] nor where the target
    /// already has a token.
    pub fn may_ask(&self, seat: Seat, cell: Cell, target: Seat) -> bool {
        cell.is_valid()
            && target != seat
            && target.0 < self.players
            && self.at(cell).cube.is_none()
            && !self.at(cell).has(target)
    }

    /// [R p. 7] A search: a space of one's clue without a cube.
    pub fn may_search(&self, seat: Seat, cell: Cell) -> bool {
        cell.is_valid() && self.hint_mask(seat).contains(cell) && self.at(cell).cube.is_none()
    }

    fn cubes_for(&self, seat: Seat) -> Vec<Cell> {
        (0..CELLS)
            .map(Cell)
            .filter(|&c| self.may_cube(seat, c))
            .collect()
    }

    fn lay(&mut self, seat: Seat, cell: Cell, answer: Answer, why: Why) {
        let t = &mut self.tokens[cell.index()];
        match answer {
            Answer::Disc => t.discs.push(seat),
            Answer::Cube => t.cube = Some(seat),
        }
        self.log.push(Event::Placed {
            seat,
            cell,
            answer,
            why,
        });
    }

    /// The turn passes to the left.
    fn end_turn(&mut self) {
        self.turn = self.next(self.turn);
        self.phase = Phase::Turn;
    }

    /// [R p. 7] After a cube laid because of the seat to move: its own
    /// cube, unless it has nowhere to put one (not foreseen by [R]).
    fn penalty_or_end(&mut self) {
        if self.cubes_for(self.turn).is_empty() {
            self.end_turn();
        } else {
            self.phase = Phase::Penalty;
        }
    }

    /// [R p. 6] The next setup cube, skipping a seat with nowhere to lay one.
    fn advance_setup(&mut self) {
        while self.setup_left > 0 {
            if !self.cubes_for(self.turn).is_empty() {
                self.phase = Phase::Setup;
                return;
            }
            self.setup_left -= 1;
            self.turn = self.next(self.turn);
        }
        self.turn = self.first;
        self.phase = Phase::Turn;
    }

    /// [R p. 7] The others answer in turn order from the searcher's left,
    /// those with a disc there already skipped; the first cube stops it.
    fn resolve_search(&mut self, cell: Cell) {
        let by = self.turn;
        let mut seat = self.next(by);
        while seat != by {
            if !self.at(cell).discs.contains(&seat) {
                let answer = self.answer(seat, cell);
                self.lay(seat, cell, answer, Why::Search);
                if answer == Answer::Cube {
                    self.penalty_or_end();
                    return;
                }
            }
            seat = self.next(seat);
        }
        self.winner = Some(by);
        self.phase = Phase::Over;
        self.log.push(Event::Won { seat: by });
    }
}

impl Game for Cryptide {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cryptide",
            name: "Cryptide",
            min_players: 3,
            max_players: 5,
            hidden_info: true,
            clock: false,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "mode",
            kind: OptionKind::Choice {
                values: &["Normal", "Advanced"],
                default: "Normal",
            },
        }]
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let puzzle = generate(config.mode, players, seed);
        // The first player, from the seed too.
        let first = Seat((seed.rotate_left(17) % u64::from(players)) as u8);
        let mut s = State {
            players,
            mode: config.mode,
            puzzle,
            tokens: vec![Tokens::default(); usize::from(CELLS)],
            phase: Phase::Setup,
            turn: first,
            first,
            setup_left: 2 * players,
            log: Vec::new(),
            winner: None,
            cache: TableCache::default(),
        };
        s.advance_setup();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Over => Pending::none(),
            _ => Pending::one(s.turn),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat != s.turn || s.phase == Phase::Over {
            return Vec::new();
        }
        let cells = (0..CELLS).map(Cell);
        match s.phase {
            Phase::Setup | Phase::Penalty => {
                s.cubes_for(seat).into_iter().map(Action::Cube).collect()
            }
            Phase::Relocate { .. } => cells
                .filter(|&c| s.may_disc(seat, c))
                .map(Action::Disc)
                .collect(),
            Phase::Turn => {
                let mut out = Vec::new();
                for cell in cells {
                    for target in (0..s.players).map(Seat) {
                        if s.may_ask(seat, cell, target) {
                            out.push(Action::Ask { cell, target });
                        }
                    }
                    if s.may_search(seat, cell) {
                        out.push(Action::Search(cell));
                    }
                }
                out
            }
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est finie."));
        }
        if seat != s.turn {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match (s.phase.clone(), action) {
            (Phase::Setup, &Action::Cube(cell)) => {
                if !s.may_cube(seat, cell) {
                    return Err(Illegal::new(
                        "Un cube va sur une case libre de cube où l'habitat ne peut pas être selon ton indice.",
                    ));
                }
                s.lay(seat, cell, Answer::Cube, Why::Setup);
                s.setup_left -= 1;
                s.turn = s.next(seat);
                s.advance_setup();
            }
            (Phase::Penalty, &Action::Cube(cell)) => {
                if !s.may_cube(seat, cell) {
                    return Err(Illegal::new(
                        "Un cube va sur une case libre de cube où l'habitat ne peut pas être selon ton indice.",
                    ));
                }
                s.lay(seat, cell, Answer::Cube, Why::Penalty);
                s.end_turn();
            }
            (Phase::Turn, &Action::Ask { cell, target }) => {
                if !s.may_ask(seat, cell, target) {
                    return Err(Illegal::new(
                        "On n'interroge pas sur une case à cube, ni là où l'interrogé a déjà un jeton.",
                    ));
                }
                s.log.push(Event::Asked {
                    by: seat,
                    target,
                    cell,
                });
                let answer = s.answer(target, cell);
                s.lay(target, cell, answer, Why::Answer);
                match answer {
                    Answer::Disc => s.end_turn(),
                    Answer::Cube => s.penalty_or_end(),
                }
            }
            (Phase::Turn, &Action::Search(cell)) => {
                if !s.may_search(seat, cell) {
                    return Err(Illegal::new(
                        "On fouille une case sans cube où l'habitat peut être selon son indice.",
                    ));
                }
                s.log.push(Event::Searched { by: seat, cell });
                if s.at(cell).discs.contains(&seat) {
                    if (0..CELLS).map(Cell).any(|c| s.may_disc(seat, c)) {
                        s.phase = Phase::Relocate { search: cell };
                    } else {
                        s.resolve_search(cell);
                    }
                } else {
                    s.lay(seat, cell, Answer::Disc, Why::Search);
                    s.resolve_search(cell);
                }
            }
            (Phase::Relocate { search }, &Action::Disc(cell)) => {
                if !s.may_disc(seat, cell) {
                    return Err(Illegal::new(
                        "Ton disque va sur une autre case possible selon ton indice, sans jeton à toi.",
                    ));
                }
                s.lay(seat, cell, Answer::Disc, Why::Search);
                s.resolve_search(search);
            }
            _ => return Err(Illegal::new("Ce coup n'est pas possible maintenant.")),
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        let winner = s.winner.filter(|_| s.phase == Phase::Over)?;
        Some(Outcome::Ranked(
            (0..s.players).map(|p| u8::from(p != winner.0)).collect(),
        ))
    }
}
