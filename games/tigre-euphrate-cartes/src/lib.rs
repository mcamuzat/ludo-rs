//! Tigre et Euphrate, le jeu de cartes (« le combat des rois », Reiner
//! Knizia, 2005): eight treasures in a row, a column of cards under each,
//! kingdoms joined by a card laid between two treasures. Points come in four
//! colours and only the weakest counts. Rules and their sources: RULES.md,
//! where [R] is the French translation and [L] the English summary.

mod heuristics;
mod view;

pub use view::{FinalScore, PhaseView, View};

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub struct TigreEuphrateCartes;

/// [R] People and the king (black), temples and the priest (red), farms
/// and the farmer (blue), markets and the merchant (green).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
    Black,
    Red,
    Blue,
    Green,
}

pub const COLORS: [Color; 4] = [Color::Black, Color::Red, Color::Blue, Color::Green];

impl Color {
    pub fn index(self) -> usize {
        self as usize
    }
}

/// [R] 185 civilisation cards besides the treasures, by colour.
pub const DECK: [u8; 4] = [40, 65, 40, 40];
/// [R] Eight treasures, so seven junctions between them.
pub const COLUMNS: usize = 8;
pub const GAPS: usize = COLUMNS - 1;
/// [R] Up to eight cards under each treasure.
pub const COLUMN_MAX: usize = 8;
/// [R] A junction needs three cards under both treasures.
pub const JOIN_MIN: usize = 3;
/// [R] Hands of eight.
pub const HAND: u8 = 8;
/// [R] Two actions a turn.
pub const ACTIONS: u8 = 2;
/// [R] At two, thirty cards are set aside unseen.
pub const REMOVED_AT_TWO: usize = 30;
/// [L] Four cards of a colour in a row build a ship.
pub const SHIP_RUN: usize = 4;
/// [R] Three ships, each blue and one other colour: named by the other.
pub const SHIPS: [Color; 3] = [Color::Black, Color::Red, Color::Green];
/// Site addition, not in [R]: the game ends after this many rounds of the
/// table without a card drawn.
pub const QUIET_ROUNDS: u32 = 20;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

/// A place on the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pos {
    /// The treasure* heading a column.
    Head(u8),
    /// Between the treasures* of columns `i` and `i + 1`.
    Gap(u8),
    /// Column, then rank from the top.
    Card(u8, u8),
}

impl Pos {
    pub fn column(self) -> u8 {
        match self {
            Pos::Head(c) | Pos::Gap(c) | Pos::Card(c, _) => c,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Face {
    Civ(Color),
    /// [R] A treasure card, still on the table.
    Treasure,
    /// [R] The red card that replaced a treasure taken by a merchant.
    Exchanged,
    /// A ship, by its colour other than blue.
    Ship(Color),
}

impl Face {
    /// The colour the card counts as: [L] treasures* are red; ships count
    /// for nothing.
    pub fn color(self) -> Option<Color> {
        match self {
            Face::Civ(c) => Some(c),
            Face::Treasure | Face::Exchanged => Some(Color::Red),
            Face::Ship(_) => None,
        }
    }

    pub fn is_head(self) -> bool {
        matches!(self, Face::Treasure | Face::Exchanged)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Leader {
    pub seat: Seat,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    pub face: Face,
    pub leader: Option<Leader>,
}

impl Slot {
    fn new(face: Face) -> Self {
        Self { face, leader: None }
    }
}

/// Columns `a..=b` and what joins them: a kingdom.
pub type Span = (u8, u8);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub heads: Vec<Slot>,
    pub gaps: Vec<Option<Slot>>,
    pub columns: Vec<Vec<Slot>>,
}

impl Table {
    fn new() -> Self {
        Self {
            heads: vec![Slot::new(Face::Treasure); COLUMNS],
            gaps: vec![None; GAPS],
            columns: vec![Vec::new(); COLUMNS],
        }
    }

    pub fn slot(&self, p: Pos) -> Option<&Slot> {
        match p {
            Pos::Head(c) => self.heads.get(usize::from(c)),
            Pos::Gap(g) => self.gaps.get(usize::from(g)).and_then(Option::as_ref),
            Pos::Card(c, r) => self
                .columns
                .get(usize::from(c))
                .and_then(|col| col.get(usize::from(r))),
        }
    }

    fn slot_mut(&mut self, p: Pos) -> Option<&mut Slot> {
        match p {
            Pos::Head(c) => self.heads.get_mut(usize::from(c)),
            Pos::Gap(g) => self.gaps.get_mut(usize::from(g)).and_then(Option::as_mut),
            Pos::Card(c, r) => self
                .columns
                .get_mut(usize::from(c))
                .and_then(|col| col.get_mut(usize::from(r))),
        }
    }

    /// [R] The kingdom of `column`: the columns joined to it by junction
    /// cards, `skip` (a face-down junction) not counted.
    pub fn span(&self, column: u8, skip: Option<u8>) -> Span {
        let linked = |g: u8| self.gaps[usize::from(g)].is_some() && Some(g) != skip;
        let mut a = column;
        while a > 0 && linked(a - 1) {
            a -= 1;
        }
        let mut b = column;
        while usize::from(b) < GAPS && linked(b) {
            b += 1;
        }
        (a, b)
    }

    /// Every card of a span, heads first in each column.
    pub fn positions(&self, (a, b): Span) -> Vec<Pos> {
        let mut out = Vec::new();
        for c in a..=b {
            out.push(Pos::Head(c));
            for r in 0..self.columns[usize::from(c)].len() {
                out.push(Pos::Card(c, r as u8));
            }
            if c < b && self.gaps[usize::from(c)].is_some() {
                out.push(Pos::Gap(c));
            }
        }
        out
    }

    pub fn all_positions(&self) -> Vec<Pos> {
        self.positions((0, GAPS as u8))
    }

    pub fn leaders_in(&self, span: Span) -> Vec<(Pos, Leader)> {
        self.positions(span)
            .into_iter()
            .filter_map(|p| self.slot(p).and_then(|s| s.leader).map(|l| (p, l)))
            .collect()
    }

    pub fn find_leader(&self, seat: Seat, color: Color) -> Option<Pos> {
        self.leaders_in((0, GAPS as u8))
            .into_iter()
            .find(|(_, l)| l.seat == seat && l.color == color)
            .map(|(p, _)| p)
    }

    /// Cards of `color` in a span (treasures* for red, ships never).
    pub fn count(&self, span: Span, color: Color) -> u8 {
        self.positions(span)
            .into_iter()
            .filter(|&p| self.slot(p).and_then(|s| s.face.color()) == Some(color))
            .count() as u8
    }

    /// Treasure cards still on the table in a span.
    pub fn treasures(&self, (a, b): Span) -> Vec<u8> {
        (a..=b)
            .filter(|&c| self.heads[usize::from(c)].face == Face::Treasure)
            .collect()
    }

    /// Removes a column card or a junction (the cards below move up).
    fn remove(&mut self, p: Pos) -> Option<Slot> {
        match p {
            Pos::Head(_) => None,
            Pos::Gap(g) => self.gaps.get_mut(usize::from(g)).and_then(Option::take),
            Pos::Card(c, r) => {
                let col = self.columns.get_mut(usize::from(c))?;
                (usize::from(r) < col.len()).then(|| col.remove(usize::from(r)))
            }
        }
    }

    /// Who scores a card of `color` laid in a span: [R] the owner of the
    /// leader of that colour, otherwise of the king.
    pub fn scorer(&self, span: Span, color: Color) -> Option<Seat> {
        let leaders = self.leaders_in(span);
        leaders
            .iter()
            .find(|(_, l)| l.color == color)
            .or_else(|| leaders.iter().find(|(_, l)| l.color == Color::Black))
            .map(|(_, l)| l.seat)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// [R] Place a leader from one's supply, or move one already laid.
    Leader {
        color: Color,
        to: Pos,
    },
    /// [R] Lay a card at the bottom of a column.
    Extend {
        color: Color,
        column: u8,
    },
    /// [R] Lay a card between two treasures*, joining their kingdoms.
    Join {
        color: Color,
        gap: u8,
    },
    /// [R] Remove a card with one's catastrophe.
    Catastrophe {
        at: Pos,
    },
    /// Stop before using both actions.
    EndTurn,
    /// [R] Add cards from one's hand to a conflict.
    Commit {
        count: u8,
    },
    /// [R] Several conflicts: the player whose turn it is picks the next.
    Resolve {
        color: Color,
    },
    /// [R] Build this ship on the four cards just completed.
    Ship {
        ship: Color,
    },
    NoShip,
}

/// [R] Internal: a leader placed where one of its colour already is, fought
/// with red cards. External: two kingdoms joined, fought with the cards of
/// the leaders' colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictKind {
    Internal,
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub kind: ConflictKind,
    pub color: Color,
    pub attacker: Seat,
    pub defender: Seat,
    pub attacker_base: u8,
    pub defender_base: u8,
    /// `None` while the attacker decides.
    pub attacker_added: Option<u8>,
}

impl Conflict {
    /// The colour of the cards that count and are committed.
    pub fn support(&self) -> Color {
        match self.kind {
            ConflictKind::Internal => Color::Red,
            ConflictKind::External => self.color,
        }
    }

    pub fn deciding(&self) -> Seat {
        if self.attacker_added.is_none() {
            self.attacker
        } else {
            self.defender
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Action,
    Conflict(Conflict),
    /// Conflicts left after a junction, in the colours listed.
    ChooseConflict(Vec<Color>),
    /// The column just completed and the ships that fit.
    Ship {
        column: u8,
        options: Vec<Color>,
    },
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    /// [R] One treasure card left on the table.
    Treasures,
    /// [R] Not enough cards to refill every hand.
    Deck,
    /// [`QUIET_ROUNDS`] rounds without a card drawn.
    Quiet,
}

/// What happened, for the view (all of it is seen at the table).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all_fields = "camelCase")]
pub enum Event {
    Leader {
        seat: Seat,
        color: Color,
        moved: bool,
        to: Pos,
    },
    Extend {
        seat: Seat,
        color: Color,
        column: u8,
        scorer: Option<Seat>,
    },
    Join {
        seat: Seat,
        color: Color,
        gap: u8,
    },
    Catastrophe {
        seat: Seat,
        at: Pos,
        color: Color,
    },
    Conflict {
        kind: ConflictKind,
        color: Color,
        attacker: Seat,
        defender: Seat,
        attacker_total: u8,
        defender_total: u8,
        winner: Seat,
        /// The winner scored a point for the cards committed.
        scored: bool,
        removed: u8,
    },
    Ship {
        seat: Seat,
        ship: Color,
        column: u8,
    },
    /// A leader on a card discarded for a ship goes home.
    Returned {
        seat: Seat,
        color: Color,
    },
    /// [R] The merchant's owner swapped a treasure for a red card.
    Treasure {
        seat: Seat,
        column: u8,
    },
    ShipPoint {
        seat: Seat,
        color: Color,
    },
}

/// The events of a finished turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastTurn {
    pub seat: Seat,
    pub steps: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub table: Table,
    /// [R] The junction card laid face down while its conflicts last.
    pub join: Option<u8>,
    /// Cards in hand by colour.
    pub hands: Vec<[u8; 4]>,
    /// Drawn from the end.
    pub deck: Vec<Color>,
    /// [R] At two, the cards set aside.
    pub removed: Vec<Color>,
    pub discards: u32,
    /// Still holds its catastrophe.
    pub catastrophes: Vec<bool>,
    /// Score piles by colour, treasures apart: [R] variant, visible to all.
    pub scores: Vec<[u8; 4]>,
    pub treasures: Vec<u8>,
    /// Ships still to build.
    pub ships: Vec<Color>,
    pub turn: Seat,
    pub actions_left: u8,
    pub phase: Phase,
    /// Turns played.
    pub turns: u32,
    /// Turns in a row without a card drawn.
    pub quiet: u32,
    pub steps: Vec<Event>,
    pub last_turn: Option<LastTurn>,
    pub end: Option<EndReason>,
}

/// [R] A player's score: points in his weakest colour, treasures as
/// jokers; ties broken by the next weakest. Each treasure goes to the
/// weakest colour, which maximises the sorted points.
pub fn final_points(scores: &[u8; 4], treasures: u8) -> [u32; 4] {
    let mut points = scores.map(u32::from);
    for _ in 0..treasures {
        let weakest = (0..4).min_by_key(|&i| points[i]).expect("four colours");
        points[weakest] += 1;
    }
    points
}

/// `final_points`, weakest first: compared lexicographically.
pub fn sorted_points(scores: &[u8; 4], treasures: u8) -> [u32; 4] {
    let mut points = final_points(scores, treasures);
    points.sort_unstable();
    points
}

fn hand_size(hand: &[u8; 4]) -> u8 {
    hand.iter().sum()
}

impl State {
    fn seat_count(&self) -> usize {
        usize::from(self.players)
    }

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// The seat to decide, if any.
    pub fn deciding(&self) -> Option<Seat> {
        match &self.phase {
            Phase::Over => None,
            Phase::Conflict(c) => Some(c.deciding()),
            _ => Some(self.turn),
        }
    }

    /// A leader's supply: those not on the table.
    pub fn supply(&self, seat: Seat) -> Vec<Color> {
        COLORS
            .into_iter()
            .filter(|&c| self.table.find_leader(seat, c).is_none())
            .collect()
    }

    pub fn treasures_left(&self) -> u8 {
        self.table.treasures((0, GAPS as u8)).len() as u8
    }

    /// The legal actions of `seat`; empty when it has nothing to decide.
    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.deciding() != Some(seat) {
            return Vec::new();
        }
        match &self.phase {
            Phase::Over => Vec::new(),
            Phase::Conflict(c) => (0..=self.hands[seat.index()][c.support().index()])
                .map(|count| Action::Commit { count })
                .collect(),
            Phase::ChooseConflict(colors) => colors
                .iter()
                .map(|&color| Action::Resolve { color })
                .collect(),
            Phase::Ship { options, .. } => options
                .iter()
                .map(|&ship| Action::Ship { ship })
                .chain([Action::NoShip])
                .collect(),
            Phase::Action => {
                let mut out = Vec::new();
                let free: Vec<Pos> = self
                    .table
                    .all_positions()
                    .into_iter()
                    .filter(|&p| self.leader_may_stand(p))
                    .collect();
                for color in COLORS {
                    out.extend(free.iter().map(|&to| Action::Leader { color, to }));
                }
                let hand = &self.hands[seat.index()];
                for color in COLORS.into_iter().filter(|c| hand[c.index()] > 0) {
                    for column in 0..COLUMNS as u8 {
                        if self.may_extend(column) {
                            out.push(Action::Extend { color, column });
                        }
                    }
                    for gap in 0..GAPS as u8 {
                        if self.may_join(gap) {
                            out.push(Action::Join { color, gap });
                        }
                    }
                }
                if self.catastrophes[seat.index()] {
                    out.extend(
                        self.table
                            .all_positions()
                            .into_iter()
                            .filter(|&at| self.may_destroy(at))
                            .map(|at| Action::Catastrophe { at }),
                    );
                }
                out.push(Action::EndTurn);
                out
            }
        }
    }

    /// [R] A leader goes on any free card, treasures* and junctions
    /// included; [L] never on a ship.
    pub fn leader_may_stand(&self, p: Pos) -> bool {
        self.table
            .slot(p)
            .is_some_and(|s| s.leader.is_none() && !matches!(s.face, Face::Ship(_)))
    }

    /// [R] Up to eight cards under a treasure* (ships count ❓).
    pub fn may_extend(&self, column: u8) -> bool {
        self.table.columns[usize::from(column)].len() < COLUMN_MAX
    }

    /// [R] A free place between two treasures* with three cards under each.
    pub fn may_join(&self, gap: u8) -> bool {
        let g = usize::from(gap);
        self.table.gaps[g].is_none()
            && self.table.columns[g].len() >= JOIN_MIN
            && self.table.columns[g + 1].len() >= JOIN_MIN
    }

    /// [R] Never a treasure*, a ship, nor a card a leader stands on.
    pub fn may_destroy(&self, at: Pos) -> bool {
        self.table
            .slot(at)
            .is_some_and(|s| s.leader.is_none() && matches!(s.face, Face::Civ(_)))
    }

    fn refusal(&self, seat: Seat, action: &Action) -> &'static str {
        if self.phase == Phase::Over {
            return "La partie est terminée.";
        }
        if self.deciding() != Some(seat) {
            return "Ce n'est pas ton tour.";
        }
        match (action, &self.phase) {
            (Action::Leader { .. }, Phase::Action) => {
                "Un chef se pose sur une carte libre, jamais sur un navire."
            }
            (Action::Extend { .. }, Phase::Action) => {
                "Il faut une carte de cette couleur en main et une colonne de moins de 8 cartes."
            }
            (Action::Join { .. }, Phase::Action) => {
                "Une jonction demande une place libre entre deux trésors, 3 cartes sous chacun et une carte de cette couleur en main."
            }
            (Action::Catastrophe { .. }, Phase::Action) => {
                "Catastrophe impossible : déjà jouée, ou carte protégée (trésor, navire, chef)."
            }
            (Action::Commit { .. }, Phase::Conflict(_)) => {
                "Tu n'as pas autant de cartes de cette couleur."
            }
            (Action::Resolve { .. }, Phase::ChooseConflict(_)) => "Ce conflit n'est pas en cours.",
            (Action::Ship { .. }, Phase::Ship { .. }) => "Ce navire ne convient pas ou est déjà bâti.",
            _ => "Ce n'est pas le moment.",
        }
    }

    /// [R] Scoring a point means laying a card of the colour from one's
    /// hand; the site lays it whenever it can (user's decision).
    fn score_from_hand(&mut self, seat: Seat, color: Color) -> bool {
        let held = &mut self.hands[seat.index()][color.index()];
        if *held == 0 {
            return false;
        }
        *held -= 1;
        self.scores[seat.index()][color.index()] += 1;
        true
    }

    fn act(&mut self, seat: Seat, action: Action) {
        match action {
            Action::Leader { color, to } => self.place_leader(seat, color, to),
            Action::Extend { color, column } => self.extend(seat, color, column),
            Action::Join { color, gap } => {
                self.hands[seat.index()][color.index()] -= 1;
                self.table.gaps[usize::from(gap)] = Some(Slot::new(Face::Civ(color)));
                self.join = Some(gap);
                self.steps.push(Event::Join { seat, color, gap });
                self.continue_war();
            }
            Action::Catastrophe { at } => {
                let slot = self.table.remove(at).expect("checked by moves");
                self.catastrophes[seat.index()] = false;
                self.discards += 1;
                self.steps.push(Event::Catastrophe {
                    seat,
                    at,
                    color: slot.face.color().unwrap_or(Color::Red),
                });
                self.finish_action();
            }
            Action::EndTurn => self.end_turn(),
            Action::Commit { count } => self.commit(seat, count),
            Action::Resolve { color } => self.start_war(color),
            Action::Ship { ship } => {
                let Phase::Ship { column, .. } = self.phase else {
                    unreachable!("checked by moves")
                };
                self.build_ship(seat, ship, column);
                self.finish_action();
            }
            Action::NoShip => self.finish_action(),
        }
    }

    fn place_leader(&mut self, seat: Seat, color: Color, to: Pos) {
        let from = self.table.find_leader(seat, color);
        if let Some(from) = from {
            self.table.slot_mut(from).expect("found").leader = None;
        }
        let leader = Leader { seat, color };
        self.table.slot_mut(to).expect("checked by moves").leader = Some(leader);
        self.steps.push(Event::Leader {
            seat,
            color,
            moved: from.is_some(),
            to,
        });
        let span = self.table.span(to.column(), None);
        let rival = self
            .table
            .leaders_in(span)
            .into_iter()
            .find(|(_, l)| l.color == color && l.seat != seat);
        match rival {
            // [R] Internal conflict: the player who placed attacks.
            Some((_, other)) => {
                let c = Conflict {
                    kind: ConflictKind::Internal,
                    color,
                    attacker: seat,
                    defender: other.seat,
                    attacker_base: self.king_bonus(seat, span),
                    defender_base: self.king_bonus(other.seat, span),
                    attacker_added: None,
                };
                self.phase = Phase::Conflict(c);
                self.skip_empty_commits();
            }
            None => self.finish_action(),
        }
    }

    /// [R] +1 in an internal conflict to a player whose king stands on a
    /// red card (temple or treasure*) of the kingdom.
    pub fn king_bonus(&self, seat: Seat, span: Span) -> u8 {
        self.table.leaders_in(span).into_iter().any(|(p, l)| {
            l.seat == seat
                && l.color == Color::Black
                && self.table.slot(p).and_then(|s| s.face.color()) == Some(Color::Red)
        }) as u8
    }

    fn extend(&mut self, seat: Seat, color: Color, column: u8) {
        self.hands[seat.index()][color.index()] -= 1;
        let col = usize::from(column);
        self.table.columns[col].push(Slot::new(Face::Civ(color)));
        let span = self.table.span(column, self.join);
        let scorer = self
            .table
            .scorer(span, color)
            .filter(|&owner| self.score_from_hand(owner, color));
        self.steps.push(Event::Extend {
            seat,
            color,
            column,
            scorer,
        });
        // [L] Four in a row of one colour, after the point: a ship.
        let cards = &self.table.columns[col];
        let run = cards.len() >= SHIP_RUN
            && cards[cards.len() - SHIP_RUN..]
                .iter()
                .all(|s| s.face == Face::Civ(color));
        let options: Vec<Color> = if !run {
            Vec::new()
        } else if color == Color::Blue {
            self.ships.clone()
        } else {
            self.ships.iter().copied().filter(|&s| s == color).collect()
        };
        if options.is_empty() {
            self.finish_action();
        } else {
            self.phase = Phase::Ship { column, options };
        }
    }

    fn build_ship(&mut self, seat: Seat, ship: Color, column: u8) {
        let cards = &mut self.table.columns[usize::from(column)];
        let gone = cards.split_off(cards.len() - SHIP_RUN);
        cards.push(Slot::new(Face::Ship(ship)));
        self.ships.retain(|&s| s != ship);
        self.discards += SHIP_RUN as u32;
        self.steps.push(Event::Ship { seat, ship, column });
        for l in gone.into_iter().filter_map(|s| s.leader) {
            self.steps.push(Event::Returned {
                seat: l.seat,
                color: l.color,
            });
        }
    }

    /// Colours with two leaders in the kingdom the face-down junction makes.
    pub fn war_colors(&self) -> Vec<Color> {
        let Some(gap) = self.join else {
            return Vec::new();
        };
        let leaders = self.table.leaders_in(self.table.span(gap, None));
        COLORS
            .into_iter()
            .filter(|&c| leaders.iter().filter(|(_, l)| l.color == c).count() >= 2)
            .collect()
    }

    fn continue_war(&mut self) {
        let colors = self.war_colors();
        match colors.as_slice() {
            [] => self.finish_join(),
            [only] => self.start_war(*only),
            _ => self.phase = Phase::ChooseConflict(colors),
        }
    }

    /// A leader's side of the junction: its former kingdom.
    fn side(&self, p: Pos) -> Span {
        self.table.span(p.column(), self.join)
    }

    fn start_war(&mut self, color: Color) {
        let gap = self.join.expect("a junction is pending");
        let both: Vec<(Pos, Leader)> = self
            .table
            .leaders_in(self.table.span(gap, None))
            .into_iter()
            .filter(|(_, l)| l.color == color)
            .collect();
        let involved = |s: Seat| both.iter().any(|(_, l)| l.seat == s);
        // [R] The player whose turn it is attacks if involved, otherwise the
        // first involved after him clockwise.
        let mut attacker = self.turn;
        while !involved(attacker) {
            attacker = self.next(attacker);
        }
        let (a_pos, _) = *both.iter().find(|(_, l)| l.seat == attacker).expect("two");
        let (d_pos, d) = *both.iter().find(|(_, l)| l.seat != attacker).expect("two");
        let c = Conflict {
            kind: ConflictKind::External,
            color,
            attacker,
            defender: d.seat,
            attacker_base: self.table.count(self.side(a_pos), color),
            defender_base: self.table.count(self.side(d_pos), color),
            attacker_added: None,
        };
        self.phase = Phase::Conflict(c);
        self.skip_empty_commits();
    }

    /// A seat with no card to commit commits none without being asked.
    fn skip_empty_commits(&mut self) {
        while let Phase::Conflict(c) = self.phase {
            if self.hands[c.deciding().index()][c.support().index()] > 0 {
                return;
            }
            self.commit(c.deciding(), 0);
        }
    }

    fn commit(&mut self, seat: Seat, count: u8) {
        let Phase::Conflict(mut c) = self.phase else {
            unreachable!("checked by moves")
        };
        self.hands[seat.index()][c.support().index()] -= count;
        match c.attacker_added {
            None => {
                c.attacker_added = Some(count);
                self.phase = Phase::Conflict(c);
                self.skip_empty_commits();
            }
            Some(_) => self.resolve(c, count),
        }
    }

    fn resolve(&mut self, c: Conflict, defender_added: u8) {
        let attacker_added = c.attacker_added.expect("attacker decided");
        let attacker_total = c.attacker_base + attacker_added;
        let defender_total = c.defender_base + defender_added;
        // [R] Ties go to the defender.
        let (winner, loser) = if attacker_total > defender_total {
            (c.attacker, c.defender)
        } else {
            (c.defender, c.attacker)
        };
        let support = c.support();
        let at = self.table.find_leader(loser, c.color).expect("in conflict");
        let side = self.side(at);
        // [L] The loser takes his leader back first.
        self.table.slot_mut(at).expect("found").leader = None;
        // [R] The winner scores one of the cards committed, otherwise one
        // from his hand; the other committed cards are discarded.
        let committed = u32::from(attacker_added + defender_added);
        let scored = if committed > 0 {
            self.scores[winner.index()][support.index()] += 1;
            self.discards += committed - 1;
            true
        } else {
            self.score_from_hand(winner, support)
        };
        // [R] External: the winner takes every card of the colour on the
        // loser's side, but treasures*, ships and cards with a leader.
        let mut removed = 0u8;
        if c.kind == ConflictKind::External {
            let doomed = |s: &Slot| s.leader.is_none() && s.face == Face::Civ(c.color);
            let (a, b) = side;
            for col in a..=b {
                let cards = &mut self.table.columns[usize::from(col)];
                let before = cards.len();
                cards.retain(|s| !doomed(s));
                removed += (before - cards.len()) as u8;
                if col < b {
                    let gap = &mut self.table.gaps[usize::from(col)];
                    if gap.as_ref().is_some_and(doomed) {
                        *gap = None;
                        removed += 1;
                    }
                }
            }
            self.scores[winner.index()][c.color.index()] += removed;
        }
        self.steps.push(Event::Conflict {
            kind: c.kind,
            color: c.color,
            attacker: c.attacker,
            defender: c.defender,
            attacker_total,
            defender_total,
            winner,
            scored,
            removed,
        });
        match c.kind {
            ConflictKind::Internal => self.finish_action(),
            ConflictKind::External => self.continue_war(),
        }
    }

    /// The junction turns face up; [R] then a merchant in the new kingdom
    /// with two treasures or more swaps one for a red card from its owner's
    /// hand (one only: user's decision; the leftmost ❓).
    fn finish_join(&mut self) {
        let gap = self.join.take().expect("a junction is pending");
        let span = self.table.span(gap, None);
        let merchant = self
            .table
            .leaders_in(span)
            .into_iter()
            .find(|(_, l)| l.color == Color::Green)
            .map(|(_, l)| l.seat);
        let treasures = self.table.treasures(span);
        if let Some(seat) = merchant {
            if treasures.len() >= 2 && self.hands[seat.index()][Color::Red.index()] > 0 {
                let column = treasures[0];
                self.hands[seat.index()][Color::Red.index()] -= 1;
                self.table.heads[usize::from(column)].face = Face::Exchanged;
                self.treasures[seat.index()] += 1;
                self.steps.push(Event::Treasure { seat, column });
            }
        }
        self.finish_action();
    }

    fn finish_action(&mut self) {
        self.phase = Phase::Action;
        self.actions_left -= 1;
        if self.actions_left == 0 {
            self.end_turn();
        }
    }

    fn end_turn(&mut self) {
        let seat = self.turn;
        // [R] Ships: a point per leader of the player of one of the ship's
        // colours in its kingdom ([L] the king is no joker here).
        let ships: Vec<(u8, Color)> = self
            .table
            .columns
            .iter()
            .enumerate()
            .flat_map(|(c, col)| {
                col.iter().filter_map(move |s| match s.face {
                    Face::Ship(ship) => Some((c as u8, ship)),
                    _ => None,
                })
            })
            .collect();
        for (column, ship) in ships {
            let leaders = self.table.leaders_in(self.table.span(column, None));
            for color in [Color::Blue, ship] {
                if leaders
                    .iter()
                    .any(|(_, l)| l.seat == seat && l.color == color)
                    && self.score_from_hand(seat, color)
                {
                    self.steps.push(Event::ShipPoint { seat, color });
                }
            }
        }
        self.turns += 1;
        // [R] The game ends when a single treasure is left, or when the
        // cards cannot refill every hand.
        if self.treasures_left() <= 1 {
            return self.finish(EndReason::Treasures);
        }
        let need: usize = self
            .hands
            .iter()
            .map(|h| usize::from(HAND - hand_size(h)))
            .sum();
        if need > self.deck.len() {
            return self.finish(EndReason::Deck);
        }
        // [R] Everyone refills to eight, the player first.
        for k in 0..self.players {
            let s = usize::from((seat.0 + k) % self.players);
            while hand_size(&self.hands[s]) < HAND {
                let card = self.deck.pop().expect("counted");
                self.hands[s][card.index()] += 1;
            }
        }
        self.quiet = if need > 0 { 0 } else { self.quiet + 1 };
        if self.quiet >= QUIET_ROUNDS * u32::from(self.players) {
            return self.finish(EndReason::Quiet);
        }
        self.last_turn = Some(LastTurn {
            seat,
            steps: std::mem::take(&mut self.steps),
        });
        self.turn = self.next(seat);
        self.actions_left = ACTIONS;
        self.phase = Phase::Action;
    }

    fn finish(&mut self, reason: EndReason) {
        self.end = Some(reason);
        self.phase = Phase::Over;
        self.last_turn = Some(LastTurn {
            seat: self.turn,
            steps: std::mem::take(&mut self.steps),
        });
    }

    /// Each seat's final rank: fewer seats strictly ahead is better.
    pub fn ranks(&self) -> Vec<u8> {
        let points: Vec<[u32; 4]> = (0..self.seat_count())
            .map(|i| sorted_points(&self.scores[i], self.treasures[i]))
            .collect();
        points
            .iter()
            .map(|mine| points.iter().filter(|other| *other > mine).count() as u8)
            .collect()
    }
}

impl Game for TigreEuphrateCartes {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "tigre-euphrate-cartes",
            name: "Tigre et Euphrate : le combat des rois",
            min_players: 2,
            max_players: 4,
            hidden_info: true,
            clock: true,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        // ChaCha, not SmallRng: a browser log must replay natively.
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut deck: Vec<Color> = COLORS
            .into_iter()
            .flat_map(|c| std::iter::repeat_n(c, usize::from(DECK[c.index()])))
            .collect();
        deck.shuffle(&mut rng);
        let removed = if players == 2 {
            deck.split_off(deck.len() - REMOVED_AT_TWO)
        } else {
            Vec::new()
        };
        let n = usize::from(players);
        let mut hands = vec![[0u8; 4]; n];
        for hand in &mut hands {
            for _ in 0..HAND {
                hand[deck.pop().expect("185 cards").index()] += 1;
            }
        }
        // [R] The first player is drawn at random.
        let turn = Seat((rng.gen::<u32>() % u32::from(players)) as u8);
        State {
            players,
            table: Table::new(),
            join: None,
            hands,
            deck,
            removed,
            discards: 0,
            catastrophes: vec![true; n],
            scores: vec![[0; 4]; n],
            treasures: vec![0; n],
            ships: SHIPS.to_vec(),
            turn,
            actions_left: ACTIONS,
            phase: Phase::Action,
            turns: 0,
            quiet: 0,
            steps: Vec::new(),
            last_turn: None,
            end: None,
        }
    }

    fn pending(s: &State) -> Pending {
        s.deciding().map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if !s.moves(seat).contains(action) {
            return Err(Illegal::new(s.refusal(seat, action)));
        }
        s.act(seat, *action);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        (s.phase == Phase::Over).then(|| Outcome::Ranked(s.ranks()))
    }

    /// End the turn, commit nothing, build no ship, or the first conflict.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        if s.deciding() != Some(seat) {
            return None;
        }
        match &s.phase {
            Phase::Over => None,
            Phase::Action => Some(Action::EndTurn),
            Phase::Conflict(_) => Some(Action::Commit { count: 0 }),
            Phase::ChooseConflict(colors) => Some(Action::Resolve { color: colors[0] }),
            Phase::Ship { .. } => Some(Action::NoShip),
        }
    }
}

impl Determinize for TigreEuphrateCartes {
    /// The other hands, the deck and the cards set aside at two are dealt
    /// again among themselves, sizes kept.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let mut hidden: Vec<Color> = std::mem::take(&mut d.deck);
        hidden.append(&mut d.removed);
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i != observer.index() {
                for c in COLORS {
                    hidden.extend(std::iter::repeat_n(c, usize::from(hand[c.index()])));
                }
            }
        }
        hidden.shuffle(rng);
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i == observer.index() {
                continue;
            }
            let size = hand_size(hand);
            *hand = [0; 4];
            for _ in 0..size {
                hand[hidden.pop().expect("as many").index()] += 1;
            }
        }
        d.removed = hidden.split_off(hidden.len() - s.removed.len());
        d.deck = hidden;
        d
    }
}
