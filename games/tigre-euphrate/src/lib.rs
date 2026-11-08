//! Tigre et Euphrate (Reiner Knizia, Hans im Glück 1997): leaders and
//! civilisation tiles build kingdoms between the two rivers; points come in
//! four colours and only the weakest one counts. Rules and their source:
//! RULES.md, where [R] is the French rulebook.

pub mod board;
mod heuristics;
mod view;

pub use view::{FinalScore, LastTurn, View};

use board::{
    is_corner, is_river, kingdom_count_next_to, kingdoms, kingdoms_next_to, neighbors, regions,
    square_cells, squares_with, starts_with_temple, CELLS,
};
use ludo_core::{
    Determinize, Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer,
};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub struct TigreEuphrate;

/// [R] The four colours: settlements and the king (black), temples and the
/// priest (red), farms and the farmer (blue), markets and the trader (green).
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Piece {
    /// `flipped`: turned face down under a monument.
    Tile {
        color: Color,
        flipped: bool,
    },
    Leader {
        seat: Seat,
        color: Color,
    },
    Catastrophe,
}

/// [R] Six monuments, each of two colours: every pair of colours.
pub const MONUMENTS: [[Color; 2]; 6] = [
    [Color::Black, Color::Red],
    [Color::Black, Color::Blue],
    [Color::Black, Color::Green],
    [Color::Red, Color::Blue],
    [Color::Red, Color::Green],
    [Color::Blue, Color::Green],
];

/// [R] 153 civilisation tiles. ❓ By colour (not in [R], from the box):
/// 30 black, 57 red, 36 blue, 30 green; ten temples start on the board.
pub const TILES: [u8; 4] = [30, 57, 36, 30];
/// [R] Six tiles behind each screen.
pub const HAND: u8 = 6;
/// [R] Two catastrophes each.
pub const CATASTROPHES: u8 = 2;
/// [R] Up to two actions a turn.
pub const ACTIONS: u8 = 2;
/// ludo-rs addition, not in [R]: the game ends (and is scored) after this
/// many rounds of the table without a tile laid, a catastrophe or a swap.
pub const QUIET_ROUNDS: u32 = 20;

/// How many tiles a player may add to a conflict.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reinforcements {
    /// [R] "Variante anglaise": the attacker only to get ahead, the
    /// defender exactly enough to draw level, or none.
    #[default]
    English,
    /// [R] Base rule: any number of the right colour.
    Free,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub reinforcements: Reinforcements,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// [R] Place a leader from one's supply, or move one already on the board.
    Leader { color: Color, to: u8 },
    /// [R] Take a leader back from the board.
    Withdraw { color: Color },
    /// [R] Place a civilisation tile.
    Tile { color: Color, to: u8 },
    /// [R] Lay a catastrophe tile.
    Catastrophe { at: u8 },
    /// [R] Discard these tiles (by colour) and draw as many.
    Swap { tiles: [u8; 4] },
    /// Stop before using both actions.
    EndTurn,
    /// [R] Add tiles from behind one's screen to a conflict.
    Reinforce { count: u8 },
    /// [R] Several conflicts: the player whose turn it is picks the next.
    Resolve { color: Color },
    /// [R] Build a monument on a completed square.
    Monument { square: u8, monument: u8 },
    /// Leave the completed square(s) without a monument, for good.
    NoMonument,
    /// [R] The trader takes every treasure of his kingdom but this one.
    LeaveTreasure { at: u8 },
}

/// [R] Internal: a leader placed where one of its colour already is, fought
/// with adjacent temples. External: two kingdoms joined by a tile, fought
/// with the tiles of the leaders' colour in their former kingdoms.
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
    pub attacker_at: u8,
    pub defender_at: u8,
    pub attacker_base: u8,
    pub defender_base: u8,
    /// `None` while the attacker decides.
    pub attacker_added: Option<u8>,
}

impl Conflict {
    /// The colour of the tiles that count and reinforce.
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
    /// The player whose turn it is chooses an action.
    Action,
    Reinforce(Conflict),
    /// Conflicts left after a junction, in the colours listed.
    ChooseConflict(Vec<Color>),
    /// Squares (top-left cells) the last tile completed.
    Monument(Vec<u8>),
    /// The trader's owner chooses which treasure stays.
    Treasure {
        seat: Seat,
        choices: Vec<u8>,
    },
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    /// [R] One or two treasures left on the board.
    Treasures,
    /// [R] The bag cannot bring a hand back to six.
    Bag,
    /// [`QUIET_ROUNDS`] rounds without progress.
    Quiet,
}

/// What happened, for the view (all of it is seen at the table).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all_fields = "camelCase")]
pub enum Event {
    Tile {
        seat: Seat,
        color: Color,
        at: u8,
        scorer: Option<Seat>,
    },
    Junction {
        seat: Seat,
        color: Color,
        at: u8,
    },
    Leader {
        seat: Seat,
        color: Color,
        from: Option<u8>,
        to: u8,
    },
    Withdraw {
        seat: Seat,
        color: Color,
    },
    Catastrophe {
        seat: Seat,
        at: u8,
        destroyed: Option<Color>,
    },
    Swap {
        seat: Seat,
        count: u8,
    },
    Conflict {
        kind: ConflictKind,
        color: Color,
        attacker: Seat,
        defender: Seat,
        attacker_total: u8,
        defender_total: u8,
        winner: Seat,
        removed: u8,
    },
    /// A leader left without a temple goes home.
    Returned {
        seat: Seat,
        color: Color,
    },
    Monument {
        seat: Seat,
        monument: u8,
        at: u8,
    },
    NoMonument {
        seat: Seat,
    },
    Treasures {
        seat: Seat,
        count: u8,
    },
    MonumentPoint {
        seat: Seat,
        color: Color,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub players: u8,
    pub cells: Vec<Option<Piece>>,
    pub treasure: Vec<bool>,
    /// Where each monument of [`MONUMENTS`] stands (top-left cell).
    pub monuments: Vec<Option<u8>>,
    /// [R] Squares left without a monument: none can be built there later.
    pub declined: Vec<u8>,
    /// Drawn from the end.
    pub bag: Vec<Color>,
    pub hands: Vec<[u8; 4]>,
    /// Tiles each seat discarded face down (for `determinize`).
    pub discards: Vec<[u8; 4]>,
    pub catastrophes: Vec<u8>,
    /// Points by colour, behind each screen.
    pub scores: Vec<[u32; 4]>,
    pub treasures: Vec<u32>,
    pub turn: Seat,
    pub actions_left: u8,
    pub phase: Phase,
    /// The tile laid by the action being resolved (for monuments).
    pub placed: Option<u8>,
    /// [R] The junction marker, while the conflicts it caused are resolved.
    pub junction: Option<u8>,
    /// Turns in a row without progress.
    pub quiet: u32,
    pub progress: bool,
    /// Decisions taken so far (lets the AI cut its simulations).
    pub moves: u32,
    pub steps: Vec<Event>,
    pub last_turn: Option<LastTurn>,
    pub end: Option<EndReason>,
}

fn hand_size(hand: &[u8; 4]) -> u8 {
    hand.iter().sum()
}

/// Every non-empty sub-multiset of `hand`.
fn sub_hands(hand: &[u8; 4]) -> Vec<[u8; 4]> {
    let mut out = Vec::new();
    for a in 0..=hand[0] {
        for b in 0..=hand[1] {
            for c in 0..=hand[2] {
                for d in 0..=hand[3] {
                    if a + b + c + d > 0 {
                        out.push([a, b, c, d]);
                    }
                }
            }
        }
    }
    out
}

/// [R] A player's score: points in his weakest colour, treasures as
/// jokers; ties broken by the next weakest. Each treasure goes to the
/// weakest colour, which maximises the sorted points.
pub fn final_points(scores: &[u32; 4], treasures: u32) -> [u32; 4] {
    let mut points = *scores;
    for _ in 0..treasures {
        let weakest = (0..4).min_by_key(|&i| points[i]).expect("four colours");
        points[weakest] += 1;
    }
    points
}

/// `final_points`, weakest first: compared lexicographically.
pub fn sorted_points(scores: &[u32; 4], treasures: u32) -> [u32; 4] {
    let mut points = final_points(scores, treasures);
    points.sort_unstable();
    points
}

impl State {
    fn hand(&self, seat: Seat) -> &[u8; 4] {
        &self.hands[seat.index()]
    }

    pub fn leader_at(&self, seat: Seat, color: Color) -> Option<u8> {
        self.cells
            .iter()
            .position(|p| *p == Some(Piece::Leader { seat, color }))
            .map(|c| c as u8)
    }

    fn is_temple(&self, cell: u8) -> bool {
        self.cells[usize::from(cell)]
            == Some(Piece::Tile {
                color: Color::Red,
                flipped: false,
            })
    }

    /// The temples next to `cell`.
    pub fn temples_next_to(&self, cell: u8) -> u8 {
        neighbors(cell).filter(|&n| self.is_temple(n)).count() as u8
    }

    pub fn treasures_left(&self) -> u8 {
        self.treasure.iter().filter(|&&t| t).count() as u8
    }

    /// Who decides now.
    pub fn deciding(&self) -> Option<Seat> {
        match &self.phase {
            Phase::Over => None,
            Phase::Reinforce(c) => Some(c.deciding()),
            Phase::Treasure { seat, .. } => Some(*seat),
            Phase::Action | Phase::ChooseConflict(_) | Phase::Monument(_) => Some(self.turn),
        }
    }

    /// Whether a monument of `color` is still free.
    fn monument_free(&self, color: Color) -> bool {
        MONUMENTS
            .iter()
            .zip(&self.monuments)
            .any(|(colors, at)| at.is_none() && colors.contains(&color))
    }

    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.deciding() != Some(seat) {
            return Vec::new();
        }
        match &self.phase {
            Phase::Action => {
                let mut v = Vec::new();
                let open = self.tile_cells();
                for color in COLORS {
                    v.extend(self.tile_moves(color, &open));
                }
                for color in COLORS {
                    v.extend(self.leader_moves(color));
                }
                v.extend(self.catastrophe_moves());
                v.extend(
                    sub_hands(self.hand(seat))
                        .into_iter()
                        .map(|tiles| Action::Swap { tiles }),
                );
                v.push(Action::EndTurn);
                v
            }
            Phase::Reinforce(c) => self
                .reinforce_options(c)
                .into_iter()
                .map(|count| Action::Reinforce { count })
                .collect(),
            Phase::ChooseConflict(colors) => colors
                .iter()
                .map(|&color| Action::Resolve { color })
                .collect(),
            Phase::Monument(squares) => {
                let mut v = Vec::new();
                for &square in squares {
                    let color = self.tile_color(square).expect("a completed square");
                    for (m, colors) in MONUMENTS.iter().enumerate() {
                        if self.monuments[m].is_none() && colors.contains(&color) {
                            v.push(Action::Monument {
                                square,
                                monument: m as u8,
                            });
                        }
                    }
                }
                v.push(Action::NoMonument);
                v
            }
            Phase::Treasure { choices, .. } => choices
                .iter()
                .map(|&at| Action::LeaveTreasure { at })
                .collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn tile_color(&self, cell: u8) -> Option<Color> {
        match self.cells[usize::from(cell)] {
            Some(Piece::Tile {
                color,
                flipped: false,
            }) => Some(color),
            _ => None,
        }
    }

    /// [R] A tile goes on an empty square, blue ones only on the rivers and
    /// the others never; it may not join more than two kingdoms.
    fn tile_moves(&self, color: Color, open: &[u8]) -> Vec<Action> {
        if self.hand(self.turn)[color.index()] == 0 {
            return Vec::new();
        }
        open.iter()
            .filter(|&&c| is_river(c) == (color == Color::Blue))
            .map(|&to| Action::Tile { color, to })
            .collect()
    }

    /// The empty squares where a tile joins at most two kingdoms.
    fn tile_cells(&self) -> Vec<u8> {
        let label = regions(&self.cells, None);
        let kingdom = kingdoms(&self.cells, &label);
        (0..CELLS as u8)
            .filter(|&c| {
                self.cells[usize::from(c)].is_none()
                    && kingdom_count_next_to(c, &label, &kingdom) <= 2
            })
            .collect()
    }

    /// [R] A leader goes on an empty land square next to a temple, and may
    /// not join two kingdoms. A leader on the board may move (judged as if
    /// lifted first) or be taken back.
    fn leader_moves(&self, color: Color) -> Vec<Action> {
        let from = self.leader_at(self.turn, color);
        let label = regions(&self.cells, from);
        let kingdom = kingdoms(&self.cells, &label);
        let mut v: Vec<Action> = (0..CELLS as u8)
            .filter(|&c| {
                Some(c) != from
                    && self.cells[usize::from(c)].is_none()
                    && !is_river(c)
                    && self.temples_next_to(c) > 0
                    && kingdom_count_next_to(c, &label, &kingdom) <= 1
            })
            .map(|to| Action::Leader { color, to })
            .collect();
        if from.is_some() {
            v.push(Action::Withdraw { color });
        }
        v
    }

    /// [R] On an empty square or a tile, but not on a treasure, a monument
    /// or a leader (nor on another catastrophe).
    fn catastrophe_moves(&self) -> Vec<Action> {
        if self.catastrophes[self.turn.index()] == 0 {
            return Vec::new();
        }
        (0..CELLS as u8)
            .filter(|&c| {
                !self.treasure[usize::from(c)]
                    && matches!(
                        self.cells[usize::from(c)],
                        None | Some(Piece::Tile { flipped: false, .. })
                    )
            })
            .map(|at| Action::Catastrophe { at })
            .collect()
    }

    /// The counts the deciding seat may add.
    pub fn reinforce_options(&self, c: &Conflict) -> Vec<u8> {
        let have = self.hand(c.deciding())[c.support().index()];
        match (self.config.reinforcements, c.attacker_added) {
            (Reinforcements::Free, _) => (0..=have).collect(),
            (Reinforcements::English, None) => std::iter::once(0)
                .chain((1..=have).filter(|&n| c.attacker_base + n > c.defender_base))
                .collect(),
            (Reinforcements::English, Some(added)) => {
                let need = (c.attacker_base + added).saturating_sub(c.defender_base);
                if need > 0 && need <= have {
                    vec![0, need]
                } else {
                    vec![0]
                }
            }
        }
    }

    /// The legal moves of the same kind as `action`, for `apply`.
    fn moves_like(&self, action: &Action) -> Vec<Action> {
        match (*action, &self.phase) {
            (Action::Tile { color, .. }, Phase::Action) => {
                self.tile_moves(color, &self.tile_cells())
            }
            (Action::Leader { color, .. } | Action::Withdraw { color }, Phase::Action) => {
                self.leader_moves(color)
            }
            (Action::Catastrophe { .. }, Phase::Action) => self.catastrophe_moves(),
            (Action::Swap { tiles }, Phase::Action) => {
                let hand = self.hand(self.turn);
                let held = tiles.iter().zip(hand).all(|(t, h)| t <= h);
                if held && hand_size(&tiles) > 0 {
                    vec![*action]
                } else {
                    Vec::new()
                }
            }
            (Action::EndTurn, Phase::Action) => vec![Action::EndTurn],
            _ => self.moves(self.deciding().expect("checked by apply")),
        }
    }

    fn refusal(&self, action: &Action) -> &'static str {
        match (action, &self.phase) {
            (Action::Tile { .. }, Phase::Action) => {
                "Une tuile se pose sur une case vide (bleue sur une rivière, les autres hors des rivières), sans réunir plus de deux royaumes."
            }
            (Action::Leader { .. }, Phase::Action) => {
                "Un chef se pose sur une case vide hors des rivières, à côté d'un temple, sans réunir deux royaumes."
            }
            (Action::Withdraw { .. }, Phase::Action) => "Ce chef n'est pas sur le plateau.",
            (Action::Catastrophe { .. }, Phase::Action) => {
                "Une catastrophe ne va ni sur un trésor, ni sur un monument, ni sur un chef, et il t'en faut encore une."
            }
            (Action::Swap { .. }, Phase::Action) => "Échange des tuiles que tu as derrière ton écran.",
            (_, Phase::Reinforce(_)) => "Ce nombre de renforts n'est pas permis.",
            (_, Phase::ChooseConflict(_)) => "Choisis le conflit à résoudre d'abord.",
            (_, Phase::Monument(_)) => "Choisis un monument d'une couleur du carré, ou n'en bâtis pas.",
            (_, Phase::Treasure { .. }) => {
                "Laisse un trésor de ton royaume (pas un trésor de coin s'il en reste un autre)."
            }
            _ => "Ce coup n'est pas permis maintenant.",
        }
    }

    fn act(&mut self, seat: Seat, action: Action) {
        match action {
            Action::Tile { color, to } => self.place_tile(seat, color, to),
            Action::Leader { color, to } => self.place_leader(seat, color, to),
            Action::Withdraw { color } => {
                let from = self.leader_at(seat, color).expect("a legal withdrawal");
                self.cells[usize::from(from)] = None;
                self.steps.push(Event::Withdraw { seat, color });
                self.treasure_stage();
            }
            Action::Catastrophe { at } => self.catastrophe(seat, at),
            Action::Swap { tiles } => self.swap(seat, tiles),
            Action::EndTurn => self.end_turn(),
            Action::Reinforce { count } => self.reinforce(seat, count),
            Action::Resolve { color } => self.start_war(color),
            Action::Monument { square, monument } => {
                for c in square_cells(square) {
                    if let Some(Piece::Tile { flipped, .. }) = &mut self.cells[usize::from(c)] {
                        *flipped = true;
                    }
                }
                self.monuments[usize::from(monument)] = Some(square);
                self.steps.push(Event::Monument {
                    seat,
                    monument,
                    at: square,
                });
                // [R] A treasure stays on its flipped temple; a leader left
                // without a temple goes home.
                self.return_homeless();
                self.treasure_stage();
            }
            Action::NoMonument => {
                if let Phase::Monument(squares) = &self.phase {
                    self.declined.extend(squares.iter().copied());
                }
                self.steps.push(Event::NoMonument { seat });
                self.treasure_stage();
            }
            Action::LeaveTreasure { at } => {
                let label = regions(&self.cells, None);
                let kingdom = label[usize::from(at)];
                let taken: Vec<u8> = (0..CELLS as u8)
                    .filter(|&c| {
                        c != at && self.treasure[usize::from(c)] && label[usize::from(c)] == kingdom
                    })
                    .collect();
                self.take_treasures(seat, &taken);
                self.treasure_stage();
            }
        }
    }

    fn place_tile(&mut self, seat: Seat, color: Color, to: u8) {
        let label = regions(&self.cells, None);
        let kingdom = kingdoms(&self.cells, &label);
        let joined = kingdoms_next_to(to, &label, &kingdom);
        self.hands[seat.index()][color.index()] -= 1;
        self.cells[usize::from(to)] = Some(Piece::Tile {
            color,
            flipped: false,
        });
        self.progress = true;
        self.placed = Some(to);
        match joined[..] {
            [] => self.steps.push(Event::Tile {
                seat,
                color,
                at: to,
                scorer: None,
            }),
            [k] => {
                // [R] The leader of the tile's colour in the kingdom scores,
                // else its king; nobody otherwise.
                let leader_of = |want: Color| {
                    (0..CELLS).find_map(|c| match self.cells[c] {
                        Some(Piece::Leader { seat, color }) if color == want && label[c] == k => {
                            Some(seat)
                        }
                        _ => None,
                    })
                };
                let scorer = leader_of(color).or_else(|| leader_of(Color::Black));
                if let Some(s) = scorer {
                    self.scores[s.index()][color.index()] += 1;
                }
                self.steps.push(Event::Tile {
                    seat,
                    color,
                    at: to,
                    scorer,
                });
            }
            _ => {
                // [R] Joining two kingdoms scores nothing; the junction
                // marker goes on the tile until the conflicts are settled.
                self.junction = Some(to);
                self.steps.push(Event::Junction {
                    seat,
                    color,
                    at: to,
                });
            }
        }
        self.continue_war();
    }

    fn place_leader(&mut self, seat: Seat, color: Color, to: u8) {
        let from = self.leader_at(seat, color);
        if let Some(f) = from {
            self.cells[usize::from(f)] = None;
        }
        self.cells[usize::from(to)] = Some(Piece::Leader { seat, color });
        self.steps.push(Event::Leader {
            seat,
            color,
            from,
            to,
        });
        // [R] A leader placed in a kingdom where one of its colour already
        // stands: internal conflict, fought with adjacent temples.
        let label = regions(&self.cells, None);
        let k = label[usize::from(to)];
        let rival = (0..CELLS as u8).find_map(|c| match self.cells[usize::from(c)] {
            Some(Piece::Leader {
                seat: s,
                color: col,
            }) if col == color && s != seat && label[usize::from(c)] == k => Some((s, c)),
            _ => None,
        });
        match rival {
            Some((defender, at)) => {
                self.phase = Phase::Reinforce(Conflict {
                    kind: ConflictKind::Internal,
                    color,
                    attacker: seat,
                    defender,
                    attacker_at: to,
                    defender_at: at,
                    attacker_base: self.temples_next_to(to),
                    defender_base: self.temples_next_to(at),
                    attacker_added: None,
                });
                self.skip_forced_reinforcements();
            }
            None => self.treasure_stage(),
        }
    }

    fn catastrophe(&mut self, seat: Seat, at: u8) {
        let destroyed = self.tile_color(at);
        self.forget_square(at);
        self.cells[usize::from(at)] = Some(Piece::Catastrophe);
        self.catastrophes[seat.index()] -= 1;
        self.progress = true;
        self.steps.push(Event::Catastrophe {
            seat,
            at,
            destroyed,
        });
        // [R] A leader whose last temple is destroyed goes home.
        self.return_homeless();
        self.treasure_stage();
    }

    fn swap(&mut self, seat: Seat, tiles: [u8; 4]) {
        let count = hand_size(&tiles);
        self.progress = true;
        self.steps.push(Event::Swap { seat, count });
        // [R] The game ends when a player wanting to swap cannot draw
        // enough tiles.
        if self.bag.len() < usize::from(count) {
            return self.finish(EndReason::Bag);
        }
        for (i, &n) in tiles.iter().enumerate() {
            self.hands[seat.index()][i] -= n;
            self.discards[seat.index()][i] += n;
        }
        self.draw(seat, count);
        self.finish_action();
    }

    fn draw(&mut self, seat: Seat, count: u8) {
        for _ in 0..count {
            let color = self.bag.pop().expect("checked by the caller");
            self.hands[seat.index()][color.index()] += 1;
        }
    }

    /// A tile removed: a declined square it belonged to is free again.
    fn forget_square(&mut self, cell: u8) {
        self.declined
            .retain(|&sq| !square_cells(sq).contains(&cell));
    }

    /// [R] A leader left without an adjacent temple goes back to its owner.
    fn return_homeless(&mut self) {
        for c in 0..CELLS as u8 {
            if let Some(Piece::Leader { seat, color }) = self.cells[usize::from(c)] {
                if self.temples_next_to(c) == 0 {
                    self.cells[usize::from(c)] = None;
                    self.steps.push(Event::Returned { seat, color });
                }
            }
        }
    }

    /// The colours with two leaders in the kingdom joined at `junction`.
    fn war_colors(&self, junction: u8) -> Vec<Color> {
        let label = regions(&self.cells, None);
        let k = label[usize::from(junction)];
        let mut count = [0u8; 4];
        for (piece, &l) in self.cells.iter().zip(&label) {
            if let Some(Piece::Leader { color, .. }) = piece {
                if l == k {
                    count[color.index()] += 1;
                }
            }
        }
        COLORS
            .into_iter()
            .filter(|c| count[c.index()] >= 2)
            .collect()
    }

    /// Resolves the junction's conflicts one by one, then goes on.
    fn continue_war(&mut self) {
        if let Some(j) = self.junction {
            let colors = self.war_colors(j);
            match colors[..] {
                [] => self.junction = None,
                [only] => return self.start_war(only),
                _ => {
                    self.phase = Phase::ChooseConflict(colors);
                    return;
                }
            }
        }
        self.monument_stage();
    }

    /// [R] External conflict in `color`.
    fn start_war(&mut self, color: Color) {
        let j = self.junction.expect("a war has a junction");
        let label = regions(&self.cells, None);
        let k = label[usize::from(j)];
        let sides = regions(&self.cells, Some(j));
        let leaders: Vec<(Seat, u8)> = (0..CELLS as u8)
            .filter_map(|c| match self.cells[usize::from(c)] {
                Some(Piece::Leader { seat, color: col })
                    if col == color && label[usize::from(c)] == k =>
                {
                    Some((seat, c))
                }
                _ => None,
            })
            .collect();
        let [a, b] = leaders[..] else {
            unreachable!("a war opposes two leaders")
        };
        // [R] The player who laid the tile attacks if involved, else the
        // first involved player clockwise after him.
        let order = |s: Seat| (s.0 + self.players - self.turn.0) % self.players;
        let (att, def) = if order(a.0) <= order(b.0) {
            (a, b)
        } else {
            (b, a)
        };
        // [R] Supporters: the tiles of the colour in each leader's former
        // kingdom (❓ the junction tile counts for neither).
        let support = |at: u8| {
            let side = sides[usize::from(at)];
            (0..CELLS as u8)
                .filter(|&c| sides[usize::from(c)] == side && self.tile_color(c) == Some(color))
                .count() as u8
        };
        self.phase = Phase::Reinforce(Conflict {
            kind: ConflictKind::External,
            color,
            attacker: att.0,
            defender: def.0,
            attacker_at: att.1,
            defender_at: def.1,
            attacker_base: support(att.1),
            defender_base: support(def.1),
            attacker_added: None,
        });
        self.skip_forced_reinforcements();
    }

    /// A reinforcement decision with a single option is taken at once.
    fn skip_forced_reinforcements(&mut self) {
        while let Phase::Reinforce(c) = &self.phase {
            let options = self.reinforce_options(c);
            if options.len() != 1 {
                return;
            }
            let seat = c.deciding();
            self.reinforce(seat, options[0]);
        }
    }

    fn reinforce(&mut self, seat: Seat, count: u8) {
        let Phase::Reinforce(mut c) = self.phase.clone() else {
            unreachable!("reinforce in a conflict")
        };
        // [R] Reinforcements are set aside, then discarded.
        self.hands[seat.index()][c.support().index()] -= count;
        if c.attacker_added.is_none() {
            c.attacker_added = Some(count);
            self.phase = Phase::Reinforce(c);
            self.skip_forced_reinforcements();
        } else {
            self.resolve(c, count);
        }
    }

    fn resolve(&mut self, c: Conflict, defender_added: u8) {
        let attacker_total = c.attacker_base + c.attacker_added.unwrap_or(0);
        let defender_total = c.defender_base + defender_added;
        // [R] The higher total wins; the defender wins ties.
        let (winner, loser_at) = if attacker_total > defender_total {
            (c.attacker, c.defender_at)
        } else {
            (c.defender, c.attacker_at)
        };
        let mut removed = 0u8;
        match c.kind {
            ConflictKind::Internal => {
                // [R] The loser takes his leader back; the winner scores
                // one red point.
                self.cells[usize::from(loser_at)] = None;
                self.scores[winner.index()][Color::Red.index()] += 1;
            }
            ConflictKind::External => {
                let j = self.junction.expect("a war has a junction");
                let sides = regions(&self.cells, Some(j));
                let side = sides[usize::from(loser_at)];
                self.cells[usize::from(loser_at)] = None;
                // [R] The loser's supporters leave the board, except, in a
                // war of priests, temples holding a treasure or next to a
                // leader outside the conflict.
                for cell in 0..CELLS as u8 {
                    if sides[usize::from(cell)] != side || self.tile_color(cell) != Some(c.color) {
                        continue;
                    }
                    let kept = c.color == Color::Red
                        && (self.treasure[usize::from(cell)]
                            || neighbors(cell).any(|n| {
                                matches!(self.cells[usize::from(n)], Some(Piece::Leader { .. }))
                            }));
                    if !kept {
                        self.cells[usize::from(cell)] = None;
                        self.forget_square(cell);
                        removed += 1;
                    }
                }
                // [R] One point for the leader and one per supporter removed.
                self.scores[winner.index()][c.color.index()] += 1 + u32::from(removed);
            }
        }
        self.steps.push(Event::Conflict {
            kind: c.kind,
            color: c.color,
            attacker: c.attacker,
            defender: c.defender,
            attacker_total,
            defender_total,
            winner,
            removed,
        });
        self.return_homeless();
        match c.kind {
            ConflictKind::Internal => self.treasure_stage(),
            ConflictKind::External => {
                self.phase = Phase::Action;
                self.continue_war();
            }
        }
    }

    /// [R] A tile completing a square of four of a colour may raise a
    /// monument of that colour, once its conflicts are settled.
    fn monument_stage(&mut self) {
        let squares: Vec<u8> = match self.placed {
            Some(p) => squares_with(p)
                .filter(|&sq| {
                    let color = self.tile_color(sq);
                    color.is_some_and(|col| self.monument_free(col))
                        && square_cells(sq)
                            .iter()
                            .all(|&c| self.tile_color(c) == color)
                        && !self.declined.contains(&sq)
                })
                .collect(),
            None => Vec::new(),
        };
        if squares.is_empty() {
            self.treasure_stage();
        } else {
            self.phase = Phase::Monument(squares);
        }
    }

    /// [R] At the end of an action, the trader of a kingdom holding two
    /// treasures or more takes all but one, corner treasures first.
    fn treasure_stage(&mut self) {
        loop {
            let label = regions(&self.cells, None);
            let mut found = None;
            for c in 0..CELLS {
                if let Some(Piece::Leader {
                    seat,
                    color: Color::Green,
                }) = self.cells[c]
                {
                    let k = label[c];
                    let held: Vec<u8> = (0..CELLS as u8)
                        .filter(|&t| self.treasure[usize::from(t)] && label[usize::from(t)] == k)
                        .collect();
                    if held.len() >= 2 {
                        found = Some((seat, held));
                        break;
                    }
                }
            }
            let Some((seat, held)) = found else {
                break;
            };
            let plain: Vec<u8> = held.iter().copied().filter(|&t| !is_corner(t)).collect();
            let choices = if plain.is_empty() {
                held.clone()
            } else {
                plain
            };
            if let [leave] = choices[..] {
                let taken: Vec<u8> = held.into_iter().filter(|&t| t != leave).collect();
                self.take_treasures(seat, &taken);
            } else {
                self.phase = Phase::Treasure { seat, choices };
                return;
            }
        }
        self.finish_action();
    }

    fn take_treasures(&mut self, seat: Seat, taken: &[u8]) {
        for &t in taken {
            self.treasure[usize::from(t)] = false;
        }
        self.treasures[seat.index()] += taken.len() as u32;
        self.steps.push(Event::Treasures {
            seat,
            count: taken.len() as u8,
        });
    }

    fn finish_action(&mut self) {
        self.placed = None;
        self.actions_left -= 1;
        if self.actions_left == 0 {
            self.end_turn();
        } else {
            self.phase = Phase::Action;
        }
    }

    fn end_turn(&mut self) {
        let seat = self.turn;
        // [R] One point per monument in the kingdom of one of the player's
        // leaders of its colour (the king scores only black monuments).
        let label = regions(&self.cells, None);
        for color in COLORS {
            let Some(at) = self.leader_at(seat, color) else {
                continue;
            };
            for (m, place) in self.monuments.iter().enumerate() {
                if let Some(sq) = *place {
                    if MONUMENTS[m].contains(&color)
                        && label[usize::from(sq)] == label[usize::from(at)]
                    {
                        self.scores[seat.index()][color.index()] += 1;
                        self.steps.push(Event::MonumentPoint { seat, color });
                    }
                }
            }
        }
        // [R] Everyone short of six tiles draws, the player first; the game
        // ends when the bag cannot bring a hand back to six.
        for k in 0..self.players {
            let s = Seat((seat.0 + k) % self.players);
            let need = HAND - hand_size(self.hand(s));
            if self.bag.len() < usize::from(need) {
                return self.finish(EndReason::Bag);
            }
            self.draw(s, need);
        }
        // [R] One or two treasures left: the game ends.
        if self.treasures_left() <= 2 {
            return self.finish(EndReason::Treasures);
        }
        self.quiet = if self.progress { 0 } else { self.quiet + 1 };
        if self.quiet >= QUIET_ROUNDS * u32::from(self.players) {
            return self.finish(EndReason::Quiet);
        }
        self.last_turn = Some(LastTurn {
            seat,
            steps: std::mem::take(&mut self.steps),
        });
        self.progress = false;
        self.turn = Seat((seat.0 + 1) % self.players);
        self.actions_left = ACTIONS;
        self.phase = Phase::Action;
    }

    fn finish(&mut self, reason: EndReason) {
        self.end = Some(reason);
        self.phase = Phase::Over;
        self.junction = None;
        self.last_turn = Some(LastTurn {
            seat: self.turn,
            steps: std::mem::take(&mut self.steps),
        });
    }

    /// Each seat's final rank: fewer seats strictly ahead is better.
    pub fn ranks(&self) -> Vec<u8> {
        let points: Vec<[u32; 4]> = (0..usize::from(self.players))
            .map(|i| sorted_points(&self.scores[i], self.treasures[i]))
            .collect();
        points
            .iter()
            .map(|mine| points.iter().filter(|other| *other > mine).count() as u8)
            .collect()
    }
}

impl Game for TigreEuphrate {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "tigre-euphrate",
            name: "Tigre et Euphrate",
            min_players: 2,
            max_players: 4,
            hidden_info: true,
            clock: false,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "reinforcements",
            kind: OptionKind::Choice {
                values: &["English", "Free"],
                default: "English",
            },
        }]
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if !(2..=4).contains(&players) {
            return Err(Illegal::new("Tigre et Euphrate se joue de 2 à 4 joueurs."));
        }
        Ok(())
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let mut cells = vec![None; CELLS];
        let mut treasure = vec![false; CELLS];
        let mut left = TILES;
        // [R] A temple and a treasure on each of the ten sphinx squares.
        for c in (0..CELLS as u8).filter(|&c| starts_with_temple(c)) {
            cells[usize::from(c)] = Some(Piece::Tile {
                color: Color::Red,
                flipped: false,
            });
            treasure[usize::from(c)] = true;
            left[Color::Red.index()] -= 1;
        }
        let mut bag: Vec<Color> = COLORS
            .iter()
            .flat_map(|&c| std::iter::repeat_n(c, usize::from(left[c.index()])))
            .collect();
        // ChaCha, not SmallRng: a browser log must replay natively.
        bag.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
        let n = usize::from(players);
        let mut s = State {
            config: config.clone(),
            players,
            cells,
            treasure,
            monuments: vec![None; MONUMENTS.len()],
            declined: Vec::new(),
            bag,
            hands: vec![[0; 4]; n],
            discards: vec![[0; 4]; n],
            catastrophes: vec![CATASTROPHES; n],
            scores: vec![[0; 4]; n],
            treasures: vec![0; n],
            // ❓ Seat 0 starts.
            turn: Seat(0),
            actions_left: ACTIONS,
            phase: Phase::Action,
            placed: None,
            junction: None,
            quiet: 0,
            progress: false,
            moves: 0,
            steps: Vec::new(),
            last_turn: None,
            end: None,
        };
        for i in 0..players {
            s.draw(Seat(i), HAND);
        }
        s
    }

    fn pending(s: &State) -> Pending {
        s.deciding().map(Pending::one).unwrap_or_else(Pending::none)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        match s.deciding() {
            None => return Err(Illegal::new("La partie est terminée.")),
            Some(d) if d != seat => return Err(Illegal::new("Ce n'est pas à toi de décider.")),
            Some(_) => {}
        }
        if !s.moves_like(action).contains(action) {
            return Err(Illegal::new(s.refusal(action)));
        }
        s.moves += 1;
        s.act(seat, *action);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.end.map(|_| Outcome::Ranked(s.ranks()))
    }
}

/// How far the AI's memory of another player's points may be off, by
/// colour (the points are behind the screens).
const MEMORY_ERROR: f64 = 0.2;

impl Determinize for TigreEuphrate {
    /// Redraws the bag, the other hands and the tiles the others discarded
    /// face down; blurs the others' points, which the observer saw scored
    /// but cannot remember exactly. Nothing is hidden once the game is over.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        if s.end.is_some() {
            return d;
        }
        let others: Vec<usize> = (0..usize::from(s.players))
            .filter(|&i| i != observer.index())
            .collect();
        let mut pool = d.bag.clone();
        for &i in &others {
            for color in COLORS {
                let n = usize::from(d.hands[i][color.index()] + d.discards[i][color.index()]);
                pool.extend(std::iter::repeat_n(color, n));
            }
        }
        pool.shuffle(rng);
        for &i in &others {
            let (hand, discarded) = (hand_size(&d.hands[i]), hand_size(&d.discards[i]));
            d.hands[i] = [0; 4];
            d.discards[i] = [0; 4];
            for _ in 0..hand {
                d.hands[i][pool.pop().expect("as many as taken").index()] += 1;
            }
            for _ in 0..discarded {
                d.discards[i][pool.pop().expect("as many as taken").index()] += 1;
            }
            for points in &mut d.scores[i] {
                let factor = rng.gen_range(1.0 - MEMORY_ERROR..=1.0 + MEMORY_ERROR);
                *points = (f64::from(*points) * factor).round() as u32;
            }
        }
        d.bag = pool;
        d
    }
}
