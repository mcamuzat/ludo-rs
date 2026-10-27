//! Hattrick: sixty cards, 1 to 20 in three colours. Each trick holds at
//! most two piles of different colours; win cards of one colour and avoid
//! the other two. Rules and their sources: RULES.md, where [H] is the rule
//! printed in *Jeux en boîte*.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{Pile, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::deal_evenly;
use crate::standings;

pub struct Hattrick;

pub const MIN_PLAYERS: u8 = 4;
pub const MAX_PLAYERS: u8 = 6;
/// [H] 1 to 20 in each colour.
pub const HIGHEST: u8 = 20;
/// [H] A card put down to pass costs 2 points, whatever its colour.
pub const PASS_PENALTY: i32 = 2;
/// No options. [H]'s variant with a target score is left out: it need not
/// end (scores can fall as well as rise).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
    Red,
    Blue,
    Green,
}

impl Color {
    pub const ALL: [Color; 3] = [Color::Red, Color::Blue, Color::Green];

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub color: Color,
    /// 1 to 20.
    pub value: u8,
}

/// [H] Sixty cards: 1 to 20 in three colours.
pub fn pack() -> Vec<Card> {
    Color::ALL
        .iter()
        .flat_map(|&color| (1..=HIGHEST).map(move |value| Card { color, value }))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Onto the pile of the card's colour, opening it if needed.
    Play { card: Card },
    /// [H] With two piles open: show a card of the third colour and put it
    /// down face down in front of oneself.
    Pass { card: Card },
}

impl Action {
    pub fn card(self) -> Card {
        match self {
            Action::Play { card } | Action::Pass { card } => card,
        }
    }
}

/// A pile of the last trick and who took it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TakenPile {
    pub color: Color,
    /// In the order played.
    pub cards: Vec<(Seat, Card)>,
    pub winner: Seat,
}

/// The last trick: every move in order, the piles taken and who leads next.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TakenTrick {
    pub moves: Vec<(Seat, Action)>,
    pub piles: Vec<TakenPile>,
    pub leader: Seat,
}

/// One player's score for a round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundLine {
    /// Cards won, by colour.
    pub won: [u8; 3],
    /// Cards put down to pass.
    pub passed: u8,
    pub points: i32,
    /// The total after the round.
    pub total: i32,
}

/// [H] The colour with the most cards counts one point a card, the other
/// two one point less a card, and each card put down to pass two less.
/// Which of two equal colours counts makes no difference.
pub fn round_points(won: [u8; 3], passed: u8) -> i32 {
    let best = i32::from(*won.iter().max().expect("three colours"));
    let all: i32 = won.iter().map(|&n| i32::from(n)).sum();
    best - (all - best) - PASS_PENALTY * i32::from(passed)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub players: u8,
    /// Every deal comes from it: never in a view.
    pub seed: u64,
    /// From 0.
    pub round: u32,
    pub dealer: Seat,
    pub hands: Vec<Vec<Card>>,
    /// The trick being played, in order.
    pub trick: Vec<(Seat, Action)>,
    pub last_trick: Option<TakenTrick>,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// Cards won this round, by seat and colour (laid out in view of all).
    pub won: Vec<[u8; 3]>,
    /// Cards put down to pass this round, by seat.
    pub passed: Vec<u8>,
    pub scores: Vec<i32>,
    /// The previous round's scoring.
    pub last_round: Option<Vec<RoundLine>>,
}

impl State {
    /// The colours of the piles open in this trick, in the order opened.
    pub fn pile_colors(&self) -> Vec<Color> {
        pile_colors(&self.trick)
    }

    /// [H] Any card while there is one pile; with two, a card of either
    /// colour, or a pass with a card of the third.
    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.turn != Some(seat) {
            return Vec::new();
        }
        let piles = self.pile_colors();
        self.hands[seat.index()]
            .iter()
            .map(|&card| {
                if piles.len() < 2 || piles.contains(&card.color) {
                    Action::Play { card }
                } else {
                    Action::Pass { card }
                }
            })
            .collect()
    }

    /// [H] The game lasts twice as many rounds as players.
    pub fn rounds(&self) -> u32 {
        2 * u32::from(self.players)
    }

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// A fresh round from the round's own stream of the seed, so that a log
    /// replays identically. [H] The player at the dealer's left leads.
    fn start_round(&mut self) {
        let stream = self.seed ^ u64::from(self.round + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.hands = deal_evenly(pack(), self.players, stream).hands;
        self.trick.clear();
        self.won = vec![[0; 3]; usize::from(self.players)];
        self.passed = vec![0; usize::from(self.players)];
        self.turn = Some(self.next(self.dealer));
    }

    fn act(&mut self, seat: Seat, action: Action) {
        let card = action.card();
        self.hands[seat.index()].retain(|&c| c != card);
        if let Action::Pass { .. } = action {
            self.passed[seat.index()] += 1;
        }
        self.trick.push((seat, action));
        if self.trick.len() < usize::from(self.players) {
            self.turn = Some(self.next(seat));
            return;
        }
        self.end_trick();
    }

    /// [H] The highest card of each pile takes it; the player of the
    /// strongest card leads the next trick.
    fn end_trick(&mut self) {
        let moves = std::mem::take(&mut self.trick);
        let piles: Vec<TakenPile> = pile_colors(&moves)
            .into_iter()
            .map(|color| {
                let cards = pile_cards(&moves, color);
                let winner = cards
                    .iter()
                    .max_by_key(|(_, c)| c.value)
                    .map(|&(s, _)| s)
                    .expect("an open pile holds a card");
                TakenPile {
                    color,
                    cards,
                    winner,
                }
            })
            .collect();
        for pile in &piles {
            self.won[pile.winner.index()][pile.color.index()] += pile.cards.len() as u8;
        }
        let leader = strongest(&piles);
        self.last_trick = Some(TakenTrick {
            moves,
            piles,
            leader,
        });
        // [H] Each player's last card is not played.
        if self.hands.iter().all(|h| h.len() <= 1) {
            self.end_round();
        } else {
            self.turn = Some(leader);
        }
    }

    fn end_round(&mut self) {
        let lines: Vec<RoundLine> = (0..usize::from(self.players))
            .map(|i| {
                let points = round_points(self.won[i], self.passed[i]);
                self.scores[i] += points;
                RoundLine {
                    won: self.won[i],
                    passed: self.passed[i],
                    points,
                    total: self.scores[i],
                }
            })
            .collect();
        self.last_round = Some(lines);
        if self.round + 1 >= self.rounds() {
            self.turn = None;
            return;
        }
        self.round += 1;
        // [H] The deal passes to the left.
        self.dealer = self.next(self.dealer);
        self.start_round();
    }
}

/// The colours of the piles `moves` opened, in the order opened.
pub fn pile_colors(moves: &[(Seat, Action)]) -> Vec<Color> {
    let mut colors = Vec::new();
    for &(_, action) in moves {
        if let Action::Play { card } = action {
            if !colors.contains(&card.color) {
                colors.push(card.color);
            }
        }
    }
    colors
}

/// The cards played onto the pile of `color`, in order.
pub fn pile_cards(moves: &[(Seat, Action)], color: Color) -> Vec<(Seat, Card)> {
    moves
        .iter()
        .filter_map(|&(s, a)| match a {
            Action::Play { card } if card.color == color => Some((s, card)),
            _ => None,
        })
        .collect()
}

/// The player of the trick's strongest card. [H] When the two piles' highest
/// cards are equal, their second, then third highest decide; ❓ a pile that
/// runs out first loses, and identical piles go to the one opened first.
fn strongest(piles: &[TakenPile]) -> Seat {
    let values = |p: &TakenPile| {
        let mut v: Vec<u8> = p.cards.iter().map(|(_, c)| c.value).collect();
        v.sort_unstable_by(|a, b| b.cmp(a));
        v
    };
    piles
        .iter()
        .rev()
        .max_by(|a, b| values(a).cmp(&values(b)))
        .map(|p| p.winner)
        .expect("a trick opens at least one pile")
}

impl Game for Hattrick {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cards-hattrick",
            name: "Hattrick",
            min_players: MIN_PLAYERS,
            max_players: MAX_PLAYERS,
            hidden_info: true,
            clock: true,
        }
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&players) {
            return Err(Illegal::new(format!(
                "Hattrick se joue de {MIN_PLAYERS} à {MAX_PLAYERS} joueurs."
            )));
        }
        Ok(())
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            config: config.clone(),
            players,
            seed,
            round: 0,
            // ❓ [H] does not say who deals first: seat 0.
            dealer: Seat(0),
            hands: vec![Vec::new(); n],
            trick: Vec::new(),
            last_trick: None,
            turn: None,
            won: vec![[0; 3]; n],
            passed: vec![0; n],
            scores: vec![0; n],
            last_round: None,
        };
        s.start_round();
        s
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
        if !s.moves(seat).contains(action) {
            let card = action.card();
            return Err(Illegal::new(if !s.hands[seat.index()].contains(&card) {
                "Cette carte n'est pas dans ta main."
            } else {
                match action {
                    Action::Play { .. } => {
                        "Deux couleurs sont déjà posées : passe avec la troisième."
                    }
                    Action::Pass { .. } if s.pile_colors().len() < 2 => {
                        "On ne passe que lorsque deux piles sont ouvertes."
                    }
                    Action::Pass { .. } => "On passe avec une carte de la troisième couleur.",
                }
            }));
        }
        s.act(seat, *action);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.turn
            .is_none()
            .then(|| Outcome::Ranked(standings(&s.scores)))
    }

    /// The lowest card that can go onto a pile; failing that, a pass with
    /// the lowest card.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let moves = s.moves(seat);
        moves
            .iter()
            .filter(|a| matches!(a, Action::Play { .. }))
            .min_by_key(|a| a.card().value)
            .or_else(|| moves.iter().min_by_key(|a| a.card().value))
            .copied()
    }
}
