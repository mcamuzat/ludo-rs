//! Bread Basket (James Ernest, Crab Fragment Labs, v3.11): play cards into
//! one line and pick up the sandwiches you complete. Rules and their
//! sources: RULES.md, where [B] is the v3.11 rules sheet.

pub mod cards;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use cards::{completes, filling_points, full_deck, hand_penalty, hand_size, Card, Completion};

pub struct BreadBasket;

/// [B] "A full game is six rounds, or one deal per player, whichever you
/// prefer."
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rounds {
    #[default]
    Six,
    PerPlayer,
}

impl Rounds {
    pub fn count(self, players: u8) -> u8 {
        match self {
            Rounds::Six => 6,
            Rounds::PerPlayer => players,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub rounds: Rounds,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a hand card at the end of the line.
    Play { card: Card },
    /// Draw a card from the deck.
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Play,
    Over,
}

/// Why a hand ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandEnd {
    /// The deck ran out.
    DeckEmpty,
    /// A player ran out of cards and the others had their last turn.
    WentOut,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Dealt {
        round: u8,
        dealer: Seat,
        first: Seat,
    },
    Played {
        seat: Seat,
        card: Card,
    },
    /// The sandwich picked up: its bread, its middle cards and their score.
    Sandwich {
        seat: Seat,
        bread: Card,
        filling: Vec<Card>,
        points: i32,
    },
    /// Two matching cards in a row, discarded.
    Air {
        seat: Seat,
        card: Card,
    },
    Drew {
        seat: Seat,
    },
    /// The line was empty: a card from the deck starts it again.
    LineStarted {
        card: Card,
    },
    /// The first player out of cards: the others play once more.
    WentOut {
        seat: Seat,
    },
    /// Per seat: the filling picked up this hand, the cards left in hand,
    /// and the game totals.
    HandOver {
        end: HandEnd,
        gained: Vec<i32>,
        penalties: Vec<i32>,
        scores: Vec<i32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub config: Config,
    /// The current hand, from 1.
    pub round_no: u8,
    /// For each hand's shuffle. Never in a view.
    pub seed: u64,
    pub dealer: Seat,
    /// Face down; the top is the last element. Never in a view.
    pub deck: Vec<Card>,
    /// Face up: bread and air sandwiches.
    pub discard: Vec<Card>,
    /// The line in the middle of the table, oldest card first.
    pub line: Vec<Card>,
    pub hands: Vec<Vec<Card>>,
    /// The middle cards each seat picked up this hand, face up.
    pub taken: Vec<Vec<Card>>,
    pub scores: Vec<i32>,
    /// The first seat to run out of cards this hand.
    pub out: Option<Seat>,
    pub phase: Phase,
    pub turn: Seat,
    /// Since the last action of a player.
    pub events: Vec<Event>,
}

/// One independent stream per hand.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl State {
    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// The distinct values in `seat`'s hand, lowest first.
    pub fn playable(&self, seat: Seat) -> Vec<Card> {
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        cards
    }

    /// [B] Shuffle, deal by player count, and start the line. The player on
    /// the dealer's left leads.
    fn deal(&mut self, dealer: Seat) {
        let n = usize::from(self.players);
        self.round_no += 1;
        self.deck = full_deck();
        self.deck.shuffle(&mut ChaCha8Rng::seed_from_u64(mix(
            self.seed,
            self.round_no,
        )));
        self.hands = vec![Vec::new(); n];
        for _ in 0..hand_size(self.players) {
            for hand in &mut self.hands {
                hand.extend(self.deck.pop());
            }
        }
        self.discard.clear();
        self.line.clear();
        self.taken = vec![Vec::new(); n];
        self.out = None;
        self.dealer = dealer;
        self.turn = self.next(dealer);
        self.events.push(Event::Dealt {
            round: self.round_no,
            dealer,
            first: self.turn,
        });
        self.restart_line();
    }

    /// [B] "If the line is ever empty, add another card from the deck."
    fn restart_line(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.line.push(card);
            self.events.push(Event::LineStarted { card });
        }
    }

    /// [B] A card played at the end of the line: a sandwich is picked up
    /// (its bread discarded), an air sandwich discarded with a forced draw
    /// (❓ before the line is started again).
    fn play(&mut self, seat: Seat, card: Card) {
        let hand = &mut self.hands[seat.index()];
        if let Some(at) = hand.iter().position(|&c| c == card) {
            hand.remove(at);
        }
        self.events.push(Event::Played { seat, card });
        match completes(&self.line, card) {
            Completion::Sandwich { start } => {
                let filling = self.line.split_off(start + 1);
                let bread = self.line.pop().expect("the opening bread");
                self.discard.extend([bread, card]);
                self.taken[seat.index()].extend(&filling);
                self.events.push(Event::Sandwich {
                    seat,
                    bread,
                    points: filling_points(&filling),
                    filling,
                });
            }
            Completion::Air => {
                let twin = self.line.pop().expect("the matching card");
                self.discard.extend([twin, card]);
                self.events.push(Event::Air { seat, card });
                self.draw(seat);
            }
            Completion::Nothing => self.line.push(card),
        }
    }

    fn draw(&mut self, seat: Seat) {
        if let Some(card) = self.deck.pop() {
            self.hands[seat.index()].push(card);
            self.events.push(Event::Drew { seat });
        }
    }

    /// After `seat`'s action: restart an empty line, end the hand on an
    /// empty deck, note who ran out of cards, and pass the turn.
    fn advance(&mut self, seat: Seat) {
        if self.line.is_empty() {
            self.restart_line();
        }
        // [B] "If the deck runs out, the hand ends immediately."
        if self.deck.is_empty() {
            self.end_hand(HandEnd::DeckEmpty);
            return;
        }
        // [B] "If a player runs out of cards, all other players get one
        // more turn." ❓ Another player running out meanwhile changes
        // nothing.
        if self.out.is_none() && self.hands[seat.index()].is_empty() {
            self.out = Some(seat);
            self.events.push(Event::WentOut { seat });
        }
        let next = self.next(seat);
        if self.out == Some(next) {
            self.end_hand(HandEnd::WentOut);
        } else {
            self.turn = next;
        }
    }

    /// [B] Each player adds the points picked up and subtracts the cards
    /// left in hand. Then the deal passes left, or the game ends.
    fn end_hand(&mut self, end: HandEnd) {
        let gained: Vec<i32> = self.taken.iter().map(|t| filling_points(t)).collect();
        let penalties: Vec<i32> = self
            .hands
            .iter()
            .map(|h| h.iter().copied().map(hand_penalty).sum())
            .collect();
        for ((score, g), p) in self.scores.iter_mut().zip(&gained).zip(&penalties) {
            *score += g - p;
        }
        self.events.push(Event::HandOver {
            end,
            gained,
            penalties,
            scores: self.scores.clone(),
        });
        if self.round_no >= self.config.rounds.count(self.players) {
            self.phase = Phase::Over;
        } else {
            let dealer = self.next(self.dealer);
            self.deal(dealer);
        }
    }
}

impl Game for BreadBasket {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-bread-basket",
            name: "Bread Basket",
            min_players: 2,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "rounds",
            kind: OptionKind::Choice {
                values: &["Six", "PerPlayer"],
                default: "Six",
            },
        }]
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            config: config.clone(),
            round_no: 0,
            seed,
            dealer: Seat(0),
            deck: Vec::new(),
            discard: Vec::new(),
            line: Vec::new(),
            hands: vec![Vec::new(); n],
            taken: vec![Vec::new(); n],
            scores: vec![0; n],
            out: None,
            phase: Phase::Play,
            turn: Seat(0),
            events: Vec::new(),
        };
        // ❓ The first dealer is drawn from the seed.
        let dealer = Seat((seed % u64::from(players.max(1))) as u8);
        s.deal(dealer);
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Play => Pending::one(s.turn),
            Phase::Over => Pending::none(),
        }
    }

    /// [B] Play one card into the line, or draw one card.
    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Play || s.turn != seat {
            return Vec::new();
        }
        s.playable(seat)
            .into_iter()
            .map(|card| Action::Play { card })
            .chain([Action::Draw])
            .collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != seat {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Tu n'as pas cette carte."));
        }
        s.events.clear();
        match *action {
            Action::Play { card } => s.play(seat, card),
            Action::Draw => s.draw(seat),
        }
        s.advance(seat);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// ❓ Equal totals share a rank.
    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        let ranks = s
            .scores
            .iter()
            .map(|&mine| s.scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
