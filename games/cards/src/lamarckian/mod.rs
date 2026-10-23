//! Lamarckian Poker (James Ernest and Dave Howell, Cheapass Games 1996),
//! with the changes of its reworking *Stamp Showdown*: a pack cut to the
//! table, a free pick when nothing matches, and four scored showdowns.
//! Rules and their sources: RULES.md, where [L] is the Poker Suite and [S]
//! the Stamp Showdown reviews.

mod determinize;
mod heuristics;
pub mod poker;
pub mod testing;
mod view;

pub use poker::{best_hand, Category, PokerHand};
pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{pack_52, shuffled, Card, Rank, Suit};

pub struct Lamarckian;

/// [L] Cards in each starting hand and in each pool.
pub const HAND_SIZE: usize = 4;
pub const POOL_SIZE: usize = 4;
/// [S] Turns (pools) in each competition.
pub const TURNS: u8 = 7;
/// [S] Competitions in a game.
pub const DEFAULT_COMPETITIONS: u8 = 4;
pub const MAX_COMPETITIONS: u8 = 4;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// [S] 4; fewer for a shorter game (1 is close to the original).
    pub competitions: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            competitions: DEFAULT_COMPETITIONS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a hand card face down as this turn's mark.
    Mark { card: Card },
    /// The mark matched nothing: take this pool card.
    Pick { card: Card },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone picks a mark in secret.
    Choose,
    /// One player picks a pool card.
    Pick,
    Over,
}

/// One seat's result at a showdown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowdownLine {
    pub hand: PokerHand,
    /// 0: best hand; equal hands share a place.
    pub place: u8,
    pub points: u32,
    pub score: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// `lowest`: the lowest rank left in the pack ([S] the low cards out).
    Competition {
        no: u8,
        lowest: Rank,
    },
    PoolDealt {
        cards: Vec<Card>,
    },
    /// The marks, by seat.
    Revealed {
        cards: Vec<Option<Card>>,
    },
    Captured {
        seat: Seat,
        mark: Card,
        cards: Vec<Card>,
    },
    Picked {
        seat: Seat,
        mark: Card,
        card: Card,
    },
    Showdown {
        places: Vec<u8>,
        points: Vec<u32>,
        scores: Vec<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub config: Config,
    /// From 1.
    pub competition: u8,
    /// From 1 to `TURNS`.
    pub turn_no: u8,
    /// For each competition's shuffle. Never in a view.
    pub seed: u64,
    /// Face down; the top is the last element. Never in a view.
    pub deck: Vec<Card>,
    pub pool: Vec<Card>,
    pub discard: Vec<Card>,
    pub hands: Vec<Vec<Card>>,
    /// The cards of each hand that everyone saw it take from the pool.
    pub known: Vec<Vec<Card>>,
    /// Choose: the mark each seat has picked, still in its hand. Secret.
    pub chosen: Vec<Option<Card>>,
    /// The marks revealed this turn, by seat (public).
    pub revealed: Vec<Option<Card>>,
    /// Revealed marks still to act.
    pub waiting: Vec<(Seat, Card)>,
    /// Pick: who picks, with its mark.
    pub picking: Option<(Seat, Card)>,
    pub scores: Vec<u32>,
    pub last_showdown: Option<Vec<ShowdownLine>>,
    pub phase: Phase,
    /// Since the last public step.
    pub events: Vec<Event>,
}

/// [S] The low cards leave the pack so that it lasts exactly seven pools:
/// 4 × players + 28 cards, the twos to the (7 − players)s out. ❓1
pub fn lowest_rank(players: u8) -> Rank {
    Rank::ALL[usize::from(6u8.saturating_sub(players))]
}

pub fn pack(players: u8) -> Vec<Card> {
    let lowest = lowest_rank(players);
    pack_52()
        .into_iter()
        .filter(|c| c.rank() >= Some(lowest))
        .collect()
}

/// [L] Marks act from the highest: aces high, then spades, hearts, diamonds,
/// clubs. A key that sorts the strongest card last.
pub fn strength(card: Card) -> (Option<Rank>, u8) {
    let suit = match card.suit() {
        Some(Suit::Spades) => 3,
        Some(Suit::Hearts) => 2,
        Some(Suit::Diamonds) => 1,
        Some(Suit::Clubs) | None => 0,
    };
    (card.rank(), suit)
}

/// [L] A mark captures the pool cards of its rank or its suit.
pub fn matches(mark: Card, card: Card) -> bool {
    mark.rank() == card.rank() || mark.suit() == card.suit()
}

/// For display: by suit, then highest first.
pub fn sorted(mut cards: Vec<Card>) -> Vec<Card> {
    cards.sort_by(|a, b| a.suit().cmp(&b.suit()).then(b.rank().cmp(&a.rank())));
    cards
}

/// One independent stream per competition.
fn mix(seed: u64, competition: u8) -> u64 {
    seed ^ u64::from(competition).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

fn remove(cards: &mut Vec<Card>, card: Card) -> bool {
    match cards.iter().position(|&c| c == card) {
        Some(at) => {
            cards.remove(at);
            true
        }
        None => false,
    }
}

impl State {
    /// [S] A new competition: the pack cut to the table, four cards each.
    fn start_competition(&mut self) {
        let n = usize::from(self.players);
        self.competition += 1;
        self.turn_no = 0;
        self.deck = shuffled(pack(self.players), mix(self.seed, self.competition));
        self.hands = vec![Vec::new(); n];
        self.known = vec![Vec::new(); n];
        for _ in 0..HAND_SIZE {
            for hand in &mut self.hands {
                hand.extend(self.deck.pop());
            }
        }
        self.pool.clear();
        self.discard.clear();
        self.events.push(Event::Competition {
            no: self.competition,
            lowest: lowest_rank(self.players),
        });
        self.start_turn();
    }

    /// [L] Four new cards in the pool, then everyone chooses a mark.
    fn start_turn(&mut self) {
        self.discard.append(&mut self.pool);
        if self.turn_no >= TURNS {
            self.showdown();
            return;
        }
        self.turn_no += 1;
        for _ in 0..POOL_SIZE {
            self.pool.extend(self.deck.pop());
        }
        self.events.push(Event::PoolDealt {
            cards: self.pool.clone(),
        });
        let n = usize::from(self.players);
        self.chosen = vec![None; n];
        self.revealed = vec![None; n];
        self.phase = Phase::Choose;
    }

    /// [L] Reveal the marks; they act from the highest.
    fn reveal(&mut self) {
        for (i, mark) in self.chosen.iter().enumerate() {
            if let Some(mark) = *mark {
                remove(&mut self.hands[i], mark);
                remove(&mut self.known[i], mark);
            }
        }
        self.revealed = std::mem::take(&mut self.chosen);
        self.chosen = vec![None; usize::from(self.players)];
        self.events.push(Event::Revealed {
            cards: self.revealed.clone(),
        });
        self.waiting = self
            .revealed
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.map(|c| (Seat(i as u8), c)))
            .collect();
        self.waiting.sort_by_key(|&(_, c)| strength(c));
        self.run();
    }

    /// Resolve the marks in order until someone must pick or the turn ends.
    fn run(&mut self) {
        while let Some((seat, mark)) = self.waiting.pop() {
            let captured: Vec<Card> = self
                .pool
                .iter()
                .copied()
                .filter(|&c| matches(mark, c))
                .collect();
            if !captured.is_empty() {
                self.pool.retain(|&c| !matches(mark, c));
                self.gain(seat, &captured);
                self.pool.push(mark);
                self.events.push(Event::Captured {
                    seat,
                    mark,
                    cards: captured,
                });
                continue;
            }
            match self.pool.as_slice() {
                // The pool is never empty here: each mark falls back in.
                [] => self.pool.push(mark),
                &[only] => self.pick(seat, mark, only),
                _ => {
                    // [S] Any single card: the player's choice.
                    self.picking = Some((seat, mark));
                    self.phase = Phase::Pick;
                    return;
                }
            }
        }
        self.start_turn();
    }

    fn gain(&mut self, seat: Seat, cards: &[Card]) {
        self.hands[seat.index()].extend_from_slice(cards);
        self.known[seat.index()].extend_from_slice(cards);
    }

    /// [S] A mark that matched nothing takes one pool card, then falls in.
    fn pick(&mut self, seat: Seat, mark: Card, card: Card) {
        remove(&mut self.pool, card);
        self.gain(seat, &[card]);
        self.pool.push(mark);
        self.events.push(Event::Picked { seat, mark, card });
    }

    /// [S] Everyone's best poker hand: the best scores as many points as
    /// there are players, each next place one less. ❓2 ❓3
    fn showdown(&mut self) {
        let n = self.players;
        let hands: Vec<PokerHand> = self
            .hands
            .iter()
            .map(|h| {
                best_hand(h).unwrap_or(PokerHand {
                    category: Category::HighCard,
                    cards: Vec::new(),
                })
            })
            .collect();
        let places: Vec<u8> = hands
            .iter()
            .map(|mine| hands.iter().filter(|theirs| *theirs > mine).count() as u8)
            .collect();
        let points: Vec<u32> = places.iter().map(|&p| u32::from(n - p)).collect();
        for (score, p) in self.scores.iter_mut().zip(&points) {
            *score += p;
        }
        self.events.push(Event::Showdown {
            places: places.clone(),
            points: points.clone(),
            scores: self.scores.clone(),
        });
        self.last_showdown = Some(
            hands
                .into_iter()
                .enumerate()
                .map(|(i, hand)| ShowdownLine {
                    hand,
                    place: places[i],
                    points: points[i],
                    score: self.scores[i],
                })
                .collect(),
        );
        if self.competition >= self.config.competitions {
            self.phase = Phase::Over;
        } else {
            self.start_competition();
        }
    }
}

impl Game for Lamarckian {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cards-lamarckian",
            name: "Lamarckian Poker",
            min_players: 2,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "competitions",
            kind: OptionKind::Number {
                min: 1,
                max: u32::from(MAX_COMPETITIONS),
                default: u32::from(DEFAULT_COMPETITIONS),
            },
        }]
    }

    fn check_config(config: &Config, _players: u8) -> Result<(), Illegal> {
        if (1..=MAX_COMPETITIONS).contains(&config.competitions) {
            Ok(())
        } else {
            Err(Illegal::new(format!("De 1 à {MAX_COMPETITIONS} concours.")))
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            config: config.clone(),
            competition: 0,
            turn_no: 0,
            seed,
            deck: Vec::new(),
            pool: Vec::new(),
            discard: Vec::new(),
            hands: vec![Vec::new(); n],
            known: vec![Vec::new(); n],
            chosen: vec![None; n],
            revealed: vec![None; n],
            waiting: Vec::new(),
            picking: None,
            scores: vec![0; n],
            last_showdown: None,
            phase: Phase::Choose,
            events: Vec::new(),
        };
        s.start_competition();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Choose => Pending(
                (0..s.players)
                    .map(Seat)
                    .filter(|x| s.chosen[x.index()].is_none() && !s.hands[x.index()].is_empty())
                    .collect(),
            ),
            Phase::Pick => s
                .picking
                .map_or_else(Pending::none, |(seat, _)| Pending::one(seat)),
            Phase::Over => Pending::none(),
        }
    }

    /// While marks are chosen nobody learns who has already chosen.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        if s.phase != Phase::Choose {
            return Self::pending(s);
        }
        match viewer {
            Viewer::Seat(seat) if Self::pending(s).contains(seat) => Pending::one(seat),
            _ => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if !Self::pending(s).contains(seat) {
            return Vec::new();
        }
        match s.phase {
            Phase::Choose => sorted(s.hands[seat.index()].clone())
                .into_iter()
                .map(|card| Action::Mark { card })
                .collect(),
            Phase::Pick => s.pool.iter().map(|&card| Action::Pick { card }).collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new(if s.phase == Phase::Choose {
                "Tu as déjà choisi ta carte."
            } else {
                "Ce n'est pas ton tour."
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Mark { card } => {
                s.chosen[seat.index()] = Some(card);
                if Self::pending(s).is_empty() {
                    s.events.clear();
                    s.reveal();
                }
            }
            Action::Pick { card } => {
                s.events.clear();
                let (_, mark) = s.picking.take().expect("a pending pick");
                s.phase = Phase::Choose;
                s.pick(seat, mark, card);
                s.run();
            }
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// [S] The most points after the last competition; ❓ equal totals
    /// share a rank.
    fn outcome(s: &State) -> Option<Outcome> {
        (s.phase == Phase::Over).then(|| Outcome::Ranked(crate::standings(&s.scores)))
    }
}
