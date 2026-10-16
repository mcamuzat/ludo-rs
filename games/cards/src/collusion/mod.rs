//! Collusion (David Parlett): plain tricks for four, where you score a bonus
//! by taking exactly as many tricks as one other player. Players talk at
//! the table with set phrases. Rules and their sources: RULES.md.

pub mod testing;
mod view;

pub use view::View;

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{deal_evenly, pack_52, Card, Rank, Suit};
use crate::standings;

pub struct Collusion;

pub const PLAYERS: usize = 4;
pub const DEFAULT_TARGET: u32 = 100;
pub const MIN_TARGET: u32 = 50;
pub const MAX_TARGET: u32 = 100;
/// Cards per hand, so tricks per deal.
pub const TRICKS: u8 = 13;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// [C] Game is 100 up, though most reviewers prefer 50.
    pub target: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target: DEFAULT_TARGET,
        }
    }
}

/// A set phrase said at the table. [C] Agreements bind no one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Phrase {
    /// "I'm going for `n` tricks."
    Aim { n: u8 },
    /// "I don't know yet."
    Unsure,
    /// "Is anyone aiming for `n`?"
    WhoAims { n: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Play {
        card: Card,
    },
    /// Once a turn, before playing, and only with a real choice of card.
    Say {
        phrase: Phrase,
    },
}

/// [C] A K Q J 10 … 2 in each suit: the ace is the highest.
pub fn rank_value(rank: Rank) -> u8 {
    Rank::ALL
        .iter()
        .position(|&r| r == rank)
        .expect("every rank is in Rank::ALL") as u8
}

pub fn suit_index(suit: Suit) -> usize {
    Suit::ALL
        .iter()
        .position(|&s| s == suit)
        .expect("every suit is in Suit::ALL")
}

/// [C] The deal's bonuses: 10 to each of exactly two players with the same
/// number of tricks; 20 to the fewest when all differ; 30 to the fourth
/// when three tie.
pub fn bonuses(tricks: [u8; 4]) -> [u32; 4] {
    let alike = |i: usize| tricks.iter().filter(|&&t| t == tricks[i]).count();
    let mut bonus = [0; 4];
    if (0..PLAYERS).any(|i| alike(i) == 3) {
        for (i, b) in bonus.iter_mut().enumerate() {
            if alike(i) == 1 {
                *b = 30;
            }
        }
    } else if (0..PLAYERS).all(|i| alike(i) == 1) {
        let fewest = (0..PLAYERS)
            .min_by_key(|&i| tricks[i])
            .expect("four players");
        bonus[fewest] = 20;
    } else {
        for (i, b) in bonus.iter_mut().enumerate() {
            if alike(i) == 2 {
                *b = 10;
            }
        }
    }
    bonus
}

/// One player's score for a deal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DealLine {
    pub tricks: u8,
    pub bonus: u32,
    /// [C] No bonus, and the tricks would have reached the target: they were
    /// taken off instead.
    pub deducted: bool,
    /// The total after the deal.
    pub total: u32,
}

/// [C] Adds each player's tricks and bonus to `scores`. The target is only
/// reached with a bonus: without one, tricks that would reach it are
/// deducted instead.
pub fn score_deal(scores: &mut [u32; 4], tricks: [u8; 4], target: u32) -> [DealLine; 4] {
    let bonus = bonuses(tricks);
    std::array::from_fn(|i| {
        let won = u32::from(tricks[i]);
        let deducted = bonus[i] == 0 && scores[i] + won >= target;
        scores[i] = if deducted {
            scores[i].saturating_sub(won)
        } else {
            scores[i] + won + bonus[i]
        };
        DealLine {
            tricks: tricks[i],
            bonus: bonus[i],
            deducted,
            total: scores[i],
        }
    })
}

/// The last trick taken: who won it and the cards, in the order played.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TakenTrick {
    pub winner: Seat,
    pub cards: Vec<(Seat, Card)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    /// Every deal comes from it: never in a view.
    pub seed: u64,
    /// From 0.
    pub deal_no: u32,
    pub dealer: Seat,
    pub hands: Vec<Vec<Card>>,
    /// The trick being played, in the order played.
    pub trick: Vec<(Seat, Card)>,
    pub last_trick: Option<TakenTrick>,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// Tricks taken this deal.
    pub taken: [u8; 4],
    /// `voids[seat][suit_index]`: the seat failed to follow that suit this
    /// deal, so holds none of it (public knowledge).
    pub voids: [[bool; 4]; 4],
    /// The player to move has already talked this turn.
    pub said: bool,
    /// This deal's phrases, in order.
    pub talk: Vec<(Seat, Phrase)>,
    pub scores: [u32; 4],
    /// The previous deal's scoring.
    pub last_deal: Option<[DealLine; 4]>,
}

impl State {
    /// The suit led to the current trick.
    fn led(&self) -> Option<Suit> {
        self.trick.first().and_then(|(_, card)| card.suit())
    }

    /// [C] Follow suit if you can; otherwise play any card.
    pub fn playable(&self, seat: Seat) -> Vec<Card> {
        let hand = &self.hands[seat.index()];
        match self.led() {
            Some(led) if hand.iter().any(|c| c.suit() == Some(led)) => hand
                .iter()
                .copied()
                .filter(|c| c.suit() == Some(led))
                .collect(),
            _ => hand.clone(),
        }
    }

    /// The tricks `seat` may still aim for: from those taken to those taken
    /// plus the cards left in its hand (it has not played to this trick).
    pub fn aim_range(&self, seat: Seat) -> std::ops::RangeInclusive<u8> {
        let taken = self.taken[seat.index()];
        taken..=taken + self.hands[seat.index()].len() as u8
    }

    /// A fresh deal from the deal's own stream of the seed, so that a log
    /// replays identically. [C] The player at the dealer's left leads.
    fn start_deal(&mut self) {
        let stream = self.seed ^ u64::from(self.deal_no + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.hands = deal_evenly(pack_52(), PLAYERS as u8, stream).hands;
        self.trick.clear();
        self.last_trick = None;
        self.taken = [0; 4];
        self.voids = [[false; 4]; 4];
        self.said = false;
        self.talk.clear();
        self.turn = Some(next(self.dealer));
    }

    fn play(&mut self, seat: Seat, card: Card) {
        if let (Some(led), Some(suit)) = (self.led(), card.suit()) {
            if suit != led {
                self.voids[seat.index()][suit_index(led)] = true;
            }
        }
        let hand = &mut self.hands[seat.index()];
        hand.retain(|&c| c != card);
        self.trick.push((seat, card));
        self.said = false;
        if self.trick.len() < PLAYERS {
            self.turn = Some(next(seat));
            return;
        }
        // [C] The highest card of the suit led takes the trick; no trumps.
        let led = self.led();
        let winner = self
            .trick
            .iter()
            .filter(|(_, c)| c.suit() == led)
            .max_by_key(|(_, c)| c.rank().map(rank_value))
            .map(|&(s, _)| s)
            .expect("the leader followed its own suit");
        self.taken[winner.index()] += 1;
        self.last_trick = Some(TakenTrick {
            winner,
            cards: std::mem::take(&mut self.trick),
        });
        // [C] The winner of a trick leads to the next.
        self.turn = Some(winner);
        if self.hands.iter().all(Vec::is_empty) {
            self.end_deal();
        }
    }

    fn end_deal(&mut self) {
        self.last_deal = Some(score_deal(&mut self.scores, self.taken, self.config.target));
        // ❓ The game ends after the deal in which someone reaches the
        // target; the highest total wins.
        if self.scores.iter().any(|&s| s >= self.config.target) {
            self.turn = None;
            return;
        }
        let last_trick = self.last_trick.take();
        self.deal_no += 1;
        // ❓ The deal passes to the left.
        self.dealer = next(self.dealer);
        self.start_deal();
        self.last_trick = last_trick;
    }
}

fn next(seat: Seat) -> Seat {
    Seat((seat.0 + 1) % PLAYERS as u8)
}

impl Game for Collusion {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cards-collusion",
            name: "Collusion",
            min_players: 4,
            max_players: 4,
            hidden_info: true,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "target",
            kind: OptionKind::Number {
                min: MIN_TARGET,
                max: MAX_TARGET,
                default: DEFAULT_TARGET,
            },
        }]
    }

    fn check_config(config: &Config, _players: u8) -> Result<(), Illegal> {
        if (MIN_TARGET..=MAX_TARGET).contains(&config.target) {
            Ok(())
        } else {
            Err(Illegal::new(format!(
                "Collusion se joue en {MIN_TARGET} à {MAX_TARGET} points."
            )))
        }
    }

    fn new(config: &Config, _players: u8, seed: u64) -> State {
        let mut s = State {
            config: config.clone(),
            seed,
            deal_no: 0,
            // [C] does not say who deals first: seat 0.
            dealer: Seat(0),
            hands: vec![Vec::new(); PLAYERS],
            trick: Vec::new(),
            last_trick: None,
            turn: None,
            taken: [0; 4],
            voids: [[false; 4]; 4],
            said: false,
            talk: Vec::new(),
            scores: [0; 4],
            last_deal: None,
        };
        s.start_deal();
        s
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != Some(seat) {
            return Vec::new();
        }
        let playable = s.playable(seat);
        let mut legal: Vec<Action> = playable.iter().map(|&card| Action::Play { card }).collect();
        if !s.said && playable.len() >= 2 {
            legal.extend(
                s.aim_range(seat)
                    .map(|n| Phrase::Aim { n })
                    .chain([Phrase::Unsure])
                    .chain((0..=TRICKS).map(|n| Phrase::WhoAims { n }))
                    .map(|phrase| Action::Say { phrase }),
            );
        }
        legal
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.turn.is_none() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new(match action {
                Action::Play { card } if !s.hands[seat.index()].contains(card) => {
                    "Cette carte n'est pas dans ta main."
                }
                Action::Play { .. } => "Il faut fournir à la couleur demandée.",
                Action::Say { .. } if s.said => "Tu as déjà parlé à ce tour.",
                Action::Say { .. } => "Cette phrase n'est pas permise maintenant.",
            }));
        }
        match *action {
            Action::Play { card } => s.play(seat, card),
            Action::Say { phrase } => {
                s.talk.push((seat, phrase));
                s.said = true;
            }
        }
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

    /// The weakest playable card, without a word.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        if s.turn != Some(seat) {
            return None;
        }
        s.playable(seat)
            .into_iter()
            .min_by_key(|c| (c.rank().map(rank_value), c.suit().map(suit_index)))
            .map(|card| Action::Play { card })
    }
}
