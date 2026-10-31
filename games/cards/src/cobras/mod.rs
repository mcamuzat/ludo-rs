//! Cobras (Chris & Suzanne Zinsli, Cardboard Edison): the losers of a trick
//! share the cobras on its cards; the winner trades its basket for points on
//! a scale that peaks at seven cobras. Rules and their sources: RULES.md,
//! where [K] is the rulebook.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{DummyView, View};

use std::fmt;

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::deal_evenly;
use crate::standings;

pub struct Cobras;

pub const MIN_PLAYERS: u8 = 2;
pub const MAX_PLAYERS: u8 = 5;
/// [K] The round ends once this many triple-cobra cards are in the line.
pub const TRIPLES_TO_END: u32 = 10;

/// [K] Two players: the dummy's pile stays face down ("high surprise") or is
/// flipped face up once the baskets are started ("high strategy").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DummyPile {
    Hidden,
    /// ❓ The default: the site's choice.
    #[default]
    Shown,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Only used at two players.
    pub dummy: DummyPile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Suit {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Red, Suit::Green, Suit::Blue, Suit::Yellow];

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    /// 1 to 13.
    pub value: u8,
}

impl Card {
    /// [K] 1–4 show one cobra, 5–9 two, 10–13 three.
    pub fn cobras(self) -> u32 {
        match self.value {
            0..=4 => 1,
            5..=9 => 2,
            _ => 3,
        }
    }

    pub fn is_triple(self) -> bool {
        self.cobras() == 3
    }
}

impl fmt::Display for Card {
    /// `g9`: the suit's initial (r, g, b, y) and the value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letter = ['r', 'g', 'b', 'y'][self.suit.index()];
        write!(f, "{letter}{}", self.value)
    }
}

/// [K] Four suits of 1 to 13.
pub fn pack() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| (1..=13).map(move |value| Card { suit, value }))
        .collect()
}

/// [K] The basket scale: points for the cobras turned in.
pub const SCALE: [i32; 11] = [0, 1, 3, 5, 8, 11, 15, 21, 15, 8, 3];

/// [K] What a basket of `cobras` scores; 10 and more score 3.
pub fn basket_points(cobras: u32) -> i32 {
    SCALE[cobras.min(10) as usize]
}

/// [K] What a King Cobra's player declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Share {
    Win,
    Loss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Start one's basket with this card (in secret).
    Seed {
        card: Card,
    },
    Play {
        card: Card,
    },
    King {
        share: Share,
    },
}

/// A place in the playing order: a player, or the two-player dummy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Who {
    Seat(Seat),
    Dummy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Played {
    Card(Card),
    King(Share),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Play {
    pub who: Who,
    pub played: Played,
}

/// A winner who turned its basket in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scored {
    pub seat: Seat,
    pub cobras: u32,
    pub points: i32,
}

/// How the last trick ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrickResult {
    pub plays: Vec<Play>,
    /// Who won it with a card; `None` when only King Cobras were played.
    pub winner: Option<Who>,
    pub scored: Vec<Scored>,
    /// Who received cobras, and how many each.
    pub losers: Vec<Who>,
    pub share: u32,
    /// Cobras left in the centre for the next trick.
    pub center: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundLine {
    /// Won from tricks this round.
    pub gained: i32,
    /// [K] One point per cobra left in the basket.
    pub penalty: u32,
    /// The total after the round.
    pub total: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone picks the card that starts its basket.
    Seed,
    Trick,
    Over,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub players: u8,
    /// Every deal comes from it: never in a view.
    pub seed: u64,
    /// From 0.
    pub round: u32,
    /// Matters at two players only: the dummy sits at its left.
    pub dealer: Seat,
    pub phase: Phase,
    pub hands: Vec<Vec<Card>>,
    /// The dummy's pile, top first (empty unless two players).
    pub dummy: Vec<Card>,
    /// [K] The cards left over, face down.
    pub set_aside: Vec<Card>,
    /// Secret basket choices, still in hand until all are revealed.
    pub chosen: Vec<Option<Card>>,
    /// The cards that started the baskets this round, once revealed.
    pub revealed: Vec<Option<Card>>,
    pub baskets: Vec<u32>,
    /// Cobras left over from the last division.
    pub center: u32,
    /// Triple-cobra cards in the line this round.
    pub triples: u32,
    /// The seat still holds its King Cobra this round.
    pub kings: Vec<bool>,
    /// Who leads the round's first trick.
    pub first_leader: Seat,
    /// Who opened the trick in progress.
    pub leader: Who,
    pub trick: Vec<Play>,
    pub turn: Option<Who>,
    pub last_trick: Option<TrickResult>,
    /// `void[seat][suit]`: the seat did not follow this suit this round, so
    /// it holds none (public).
    pub void: Vec<[bool; 4]>,
    /// Points won from tricks this round.
    pub gained: Vec<i32>,
    pub scores: Vec<i32>,
    pub last_round: Option<Vec<RoundLine>>,
}

impl State {
    /// [K] 100 points end the game, 75 at five players.
    pub fn target(&self) -> i32 {
        if self.players == 5 {
            75
        } else {
            100
        }
    }

    pub fn has_dummy(&self) -> bool {
        self.players == 2
    }

    /// The dummy's top card shows: two players, pile flipped, baskets
    /// started.
    pub fn dummy_top_shown(&self) -> bool {
        self.config.dummy == DummyPile::Shown && self.phase == Phase::Trick
    }

    /// The suit of the first card of the trick (a King Cobra sets none).
    pub fn led(&self) -> Option<Suit> {
        self.trick.iter().find_map(|p| match p.played {
            Played::Card(c) => Some(c.suit),
            Played::King(_) => None,
        })
    }

    /// Clockwise from `who`: seat i's left is seat i + 1. [K] At two players
    /// the dummy sits at the dealer's left.
    pub fn next(&self, who: Who) -> Who {
        if self.has_dummy() {
            match who {
                Who::Seat(s) if s == self.dealer => Who::Dummy,
                Who::Seat(s) => Who::Seat(s.other()),
                Who::Dummy => Who::Seat(self.dealer.other()),
            }
        } else {
            match who {
                Who::Seat(s) => Who::Seat(Seat((s.0 + 1) % self.players)),
                Who::Dummy => unreachable!("no dummy beyond two players"),
            }
        }
    }

    /// Seats at the table this trick: players and the dummy.
    fn positions(&self) -> usize {
        usize::from(self.players) + usize::from(self.has_dummy())
    }

    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        let i = seat.index();
        match self.phase {
            Phase::Seed if self.chosen[i].is_none() => self.hands[i]
                .iter()
                .map(|&card| Action::Seed { card })
                .collect(),
            Phase::Trick if self.turn == Some(Who::Seat(seat)) => {
                // [K] Follow the led suit when possible.
                let hand = &self.hands[i];
                let led = self.led();
                let follows = |c: &Card| led.is_none_or(|suit| c.suit == suit);
                let can_follow = hand.iter().any(follows);
                let mut moves: Vec<Action> = hand
                    .iter()
                    .filter(|c| !can_follow || follows(c))
                    .map(|&card| Action::Play { card })
                    .collect();
                // [K] The King Cobra instead of a card, once a round.
                if self.kings[i] {
                    moves.push(Action::King { share: Share::Win });
                    moves.push(Action::King { share: Share::Loss });
                }
                moves
            }
            _ => Vec::new(),
        }
    }

    /// The deal of a round from the round's own stream of the seed, so that
    /// a log replays identically.
    fn start_round(&mut self) {
        let stream = self.seed ^ u64::from(self.round + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let n = usize::from(self.players);
        // [K] Two players deal as three: the third pile is the dummy's.
        let mut deal = deal_evenly(pack(), self.positions() as u8, stream);
        self.dummy = if self.has_dummy() {
            deal.hands.pop().expect("a dummy pile")
        } else {
            Vec::new()
        };
        self.hands = deal.hands;
        self.set_aside = deal.set_aside;
        self.phase = Phase::Seed;
        self.chosen = vec![None; n];
        self.revealed = vec![None; n];
        self.baskets = vec![0; n];
        self.center = 0;
        self.triples = 0;
        self.kings = vec![true; n];
        self.trick.clear();
        self.last_trick = None;
        self.void = vec![[false; 4]; n];
        self.gained = vec![0; n];
        self.turn = None;
        self.leader = Who::Seat(self.first_leader);
    }

    /// [K] All baskets are started at once.
    fn reveal(&mut self) {
        for i in 0..self.hands.len() {
            let card = self.chosen[i].take().expect("every basket card is chosen");
            self.hands[i].retain(|&c| c != card);
            self.baskets[i] += card.cobras();
            if card.is_triple() {
                self.triples += 1;
            }
            self.revealed[i] = Some(card);
        }
        self.phase = Phase::Trick;
        self.leader = Who::Seat(self.first_leader);
        self.turn = Some(self.leader);
    }

    fn act(&mut self, who: Who, played: Played) {
        if let (Who::Seat(seat), Played::Card(card)) = (who, played) {
            if let Some(led) = self.led() {
                if card.suit != led {
                    self.void[seat.index()][led.index()] = true;
                }
            }
            self.hands[seat.index()].retain(|&c| c != card);
        }
        if let (Who::Seat(seat), Played::King(_)) = (who, played) {
            self.kings[seat.index()] = false;
        }
        self.trick.push(Play { who, played });
        if self.trick.len() < self.positions() {
            self.turn = Some(self.next(who));
            self.run_dummy();
        } else {
            self.end_trick();
        }
    }

    /// [K] The dummy plays the top card of its pile on its turn.
    fn run_dummy(&mut self) {
        if self.turn == Some(Who::Dummy) {
            let card = self.dummy.remove(0);
            self.act(Who::Dummy, Played::Card(card));
        }
    }

    /// [K] The highest card of the led suit wins. A dummy following someone
    /// else's lead is on suit whatever its card, but loses a tie of value to
    /// a card actually of the suit.
    fn trick_winner(&self) -> Option<Who> {
        let led = self.led()?;
        let mut first_card = true;
        let mut best: Option<(u8, bool, Who)> = None;
        for p in &self.trick {
            let Played::Card(card) = p.played else {
                continue;
            };
            let actual = card.suit == led;
            let considered = p.who == Who::Dummy && !first_card;
            first_card = false;
            if !actual && !considered {
                continue;
            }
            let key = (card.value, actual);
            if best.is_none_or(|(v, a, _)| key > (v, a)) {
                best = Some((card.value, actual, p.who));
            }
        }
        best.map(|(_, _, who)| who)
    }

    fn end_trick(&mut self) {
        let winner = self.trick_winner();
        let cards: Vec<Card> = self
            .trick
            .iter()
            .filter_map(|p| match p.played {
                Played::Card(c) => Some(c),
                Played::King(_) => None,
            })
            .collect();
        // [K] The cobras of the cards (not the King Cobras), plus the centre.
        let total = cards.iter().map(|c| c.cobras()).sum::<u32>() + self.center;
        let mut scored = Vec::new();
        let mut losers = Vec::new();
        for p in &self.trick {
            let wins = match p.played {
                Played::Card(_) => Some(p.who) == winner,
                Played::King(share) => share == Share::Win,
            };
            if !wins {
                losers.push(p.who);
                continue;
            }
            // [K] A winner turns in its whole basket; the dummy keeps none.
            if let Who::Seat(seat) = p.who {
                let cobras = std::mem::take(&mut self.baskets[seat.index()]);
                let points = basket_points(cobras);
                self.scores[seat.index()] += points;
                self.gained[seat.index()] += points;
                scored.push(Scored {
                    seat,
                    cobras,
                    points,
                });
            }
        }
        // [K] Divided as evenly as possible; the rest stays in the centre.
        // With no loser, everything stays there.
        let share = if losers.is_empty() {
            0
        } else {
            total / losers.len() as u32
        };
        self.center = total - share * losers.len() as u32;
        for who in &losers {
            if let Who::Seat(seat) = who {
                self.baskets[seat.index()] += share;
            }
        }
        self.triples += cards.iter().filter(|c| c.is_triple()).count() as u32;
        // [K] The outright winner leads; without one, the trick's leader.
        self.leader = winner.unwrap_or(self.leader);
        self.last_trick = Some(TrickResult {
            plays: std::mem::take(&mut self.trick),
            winner,
            scored,
            losers,
            share,
            center: self.center,
        });
        // ❓ A hand left empty (a King Cobra keeps one card more) ends the
        // round too.
        let empty =
            self.hands.iter().any(Vec::is_empty) || (self.has_dummy() && self.dummy.is_empty());
        if self.triples >= TRIPLES_TO_END || empty {
            self.end_round();
        } else {
            self.turn = Some(self.leader);
            self.run_dummy();
        }
    }

    /// [K] A point lost per cobra left in the basket; the game ends once a
    /// player reaches the target.
    fn end_round(&mut self) {
        let lines = (0..self.hands.len())
            .map(|i| {
                let penalty = self.baskets[i];
                self.scores[i] -= penalty as i32;
                RoundLine {
                    gained: self.gained[i],
                    penalty,
                    total: self.scores[i],
                }
            })
            .collect();
        self.last_round = Some(lines);
        self.baskets.iter_mut().for_each(|b| *b = 0);
        self.center = 0;
        self.turn = None;
        if self.scores.iter().any(|&s| s >= self.target()) {
            self.phase = Phase::Over;
            return;
        }
        self.round += 1;
        // ❓ The deal passes to the left (alternates at two players).
        self.dealer = Seat((self.dealer.0 + 1) % self.players);
        self.first_leader = self.fewest_points_leader();
        self.start_round();
    }

    /// [K] The player with the fewest points leads the round's first trick.
    /// ❓ Among equals, the first clockwise after the last round's leader.
    fn fewest_points_leader(&self) -> Seat {
        let n = self.players;
        let low = *self.scores.iter().min().expect("players");
        (1..=n)
            .map(|k| Seat((self.first_leader.0 + k) % n))
            .find(|s| self.scores[s.index()] == low)
            .expect("a lowest score")
    }
}

/// Seat helpers for two players.
trait OtherSeat {
    fn other(self) -> Seat;
}

impl OtherSeat for Seat {
    fn other(self) -> Seat {
        Seat(1 - self.0)
    }
}

impl Game for Cobras {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cards-cobras",
            name: "Cobras",
            min_players: MIN_PLAYERS,
            max_players: MAX_PLAYERS,
            hidden_info: true,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "dummy",
            kind: OptionKind::Choice {
                values: &["Hidden", "Shown"],
                default: "Shown",
            },
        }]
    }

    fn check_config(_config: &Config, players: u8) -> Result<(), Illegal> {
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&players) {
            return Err(Illegal::new(format!(
                "Cobras se joue de {MIN_PLAYERS} à {MAX_PLAYERS} joueurs."
            )));
        }
        Ok(())
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        // [K] The first round's leader is drawn at random.
        let first_leader = Seat(((seed >> 32) % u64::from(players)) as u8);
        let mut s = State {
            config: config.clone(),
            players,
            seed,
            round: 0,
            // ❓ Seat 0 deals the first round.
            dealer: Seat(0),
            phase: Phase::Seed,
            hands: vec![Vec::new(); n],
            dummy: Vec::new(),
            set_aside: Vec::new(),
            chosen: vec![None; n],
            revealed: vec![None; n],
            baskets: vec![0; n],
            center: 0,
            triples: 0,
            kings: vec![true; n],
            first_leader,
            leader: Who::Seat(first_leader),
            trick: Vec::new(),
            turn: None,
            last_trick: None,
            void: vec![[false; 4]; n],
            gained: vec![0; n],
            scores: vec![0; n],
            last_round: None,
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Seed => Pending(
                (0..s.players)
                    .map(Seat)
                    .filter(|x| s.chosen[x.index()].is_none())
                    .collect(),
            ),
            Phase::Trick => match s.turn {
                Some(Who::Seat(seat)) => Pending::one(seat),
                _ => Pending::none(),
            },
            Phase::Over => Pending::none(),
        }
    }

    /// While baskets are started nobody learns who has already chosen.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        if s.phase != Phase::Seed {
            return Self::pending(s);
        }
        match viewer {
            Viewer::Seat(seat) if Self::pending(s).contains(seat) => Pending::one(seat),
            _ => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat.index() >= s.hands.len() {
            return Vec::new();
        }
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new(if s.phase == Phase::Seed {
                "Tu as déjà choisi la carte de ton panier."
            } else {
                "Ce n'est pas ton tour."
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new(match action {
                Action::Seed { .. } => "Ce n'est pas le moment d'amorcer ton panier.",
                Action::Play { card } if !s.hands[seat.index()].contains(card) => {
                    "Cette carte n'est pas dans ta main."
                }
                Action::Play { .. } if s.phase == Phase::Trick => {
                    "Il faut fournir la couleur demandée."
                }
                Action::Play { .. } => "Amorce d'abord ton panier.",
                Action::King { .. } if s.phase == Phase::Trick => {
                    "Tu as déjà joué ton Cobra Royal cette manche."
                }
                Action::King { .. } => "Amorce d'abord ton panier.",
            }));
        }
        match *action {
            Action::Seed { card } => {
                s.chosen[seat.index()] = Some(card);
                if Self::pending(s).is_empty() {
                    s.reveal();
                }
            }
            Action::Play { card } => s.act(Who::Seat(seat), Played::Card(card)),
            Action::King { share } => s.act(Who::Seat(seat), Played::King(share)),
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        (s.phase == Phase::Over).then(|| Outcome::Ranked(standings(&s.scores)))
    }

    /// The lowest card allowed, never the King Cobra.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        s.moves(seat)
            .into_iter()
            .filter_map(|a| match a {
                Action::Seed { card } | Action::Play { card } => Some((card.value, card.suit, a)),
                Action::King { .. } => None,
            })
            .min_by_key(|&(v, suit, _)| (v, suit))
            .map(|(_, _, a)| a)
    }
}
