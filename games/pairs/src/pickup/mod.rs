//! Pickup (James Ernest, Cheapass Games 2018): everyone plays a card in
//! secret, then each picks up a card from the pool, knocking out that rank
//! elsewhere. Rules and their sources: RULES.md, where [P] is the monthly
//! sheet.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, Rank};

pub struct Pickup;

/// [P] Cards in each pool.
pub const POOL_SIZE: usize = 5;
/// ❓ A long game stops after this many deals, at the current ranking.
pub const MAX_DEALS: u8 = 30;

/// [P] Cards dealt to each player: 7, 6 or 5 for 2, 3–4 or 5–6 players.
pub fn hand_size(players: u8) -> usize {
    match players {
        0..=2 => 7,
        3 | 4 => 6,
        _ => 5,
    }
}

/// [P] The score that ends a long game: 150, 100 or 75 for 2, 3–4 or 5–6
/// players.
pub fn target(players: u8) -> u16 {
    match players {
        0..=2 => 150,
        3 | 4 => 100,
        _ => 75,
    }
}

/// [P] One deal, or "for a longer game", deals until someone reaches
/// `target`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Length {
    #[default]
    OneDeal,
    Target,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub length: Length,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a hand card face down.
    Choose { card: Rank },
    /// Pick up a card of this rank from the pool.
    Take { card: Rank },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone picks a card in secret.
    Choose,
    /// One player picks which card to take.
    Take,
    Over,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Dealt {
        deal: u8,
        dealer: Seat,
    },
    PoolDealt {
        cards: Vec<Rank>,
    },
    /// The cards played this turn, by seat.
    Revealed {
        cards: Vec<Option<Rank>>,
    },
    /// `knocked`: how many cards of that rank left each other pile.
    Took {
        seat: Seat,
        played: Rank,
        card: Rank,
        knocked: Vec<(Seat, u8)>,
        pool_knocked: u8,
    },
    Stacked {
        seat: Seat,
        card: Rank,
    },
    DealOver {
        points: Vec<u16>,
        scores: Vec<u16>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub config: Config,
    pub deal_no: u8,
    /// For each deal's shuffle. Never in a view.
    pub seed: u64,
    pub dealer: Seat,
    /// Face down; the top is the last element. Never in a view.
    pub deck: Vec<Rank>,
    pub pool: Vec<Rank>,
    pub discard: Vec<Rank>,
    pub hands: Vec<Vec<Rank>>,
    /// Choose: the card each seat has picked, still in its hand. Secret.
    pub chosen: Vec<Option<Rank>>,
    /// The cards revealed this turn, by seat (public).
    pub revealed: Vec<Option<Rank>>,
    /// Revealed cards still to act.
    pub waiting: Vec<(Seat, Rank)>,
    /// Take: who picks, with the card it played.
    pub taking: Option<(Seat, Rank)>,
    /// [P] Ties act from the left of this seat.
    pub last_actor: Seat,
    pub piles: Vec<Vec<Rank>>,
    pub scores: Vec<u16>,
    pub phase: Phase,
    /// Since the last public step.
    pub events: Vec<Event>,
}

/// One independent stream per deal.
fn mix(seed: u64, deal: u8) -> u64 {
    seed ^ u64::from(deal).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

/// Distinct ranks, lowest first.
fn distinct(cards: &[Rank]) -> Vec<Rank> {
    let mut cards = cards.to_vec();
    cards.sort_unstable();
    cards.dedup();
    cards
}

impl State {
    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// The pool ranks a `card` may take: equal or lower.
    pub fn takeable(&self, card: Rank) -> Vec<Rank> {
        distinct(&self.pool)
            .into_iter()
            .filter(|&c| c <= card)
            .collect()
    }

    fn deal(&mut self, dealer: Seat) {
        let n = usize::from(self.players);
        self.deal_no += 1;
        self.deck = full_deck();
        self.deck
            .shuffle(&mut ChaCha8Rng::seed_from_u64(mix(self.seed, self.deal_no)));
        self.hands = vec![Vec::new(); n];
        for _ in 0..hand_size(self.players) {
            for hand in &mut self.hands {
                hand.extend(self.deck.pop());
            }
        }
        self.pool.clear();
        self.discard.clear();
        self.piles = vec![Vec::new(); n];
        self.dealer = dealer;
        // [P] The first tie of the deal starts left of the dealer.
        self.last_actor = dealer;
        self.events.push(Event::Dealt {
            deal: self.deal_no,
            dealer,
        });
        self.start_turn();
    }

    /// [P] A new pool of five cards (❓ fewer if the deck runs short), then
    /// everyone chooses. Empty hands end the deal.
    fn start_turn(&mut self) {
        self.discard.append(&mut self.pool);
        if self.hands.iter().all(Vec::is_empty) {
            self.end_deal();
            return;
        }
        while self.pool.len() < POOL_SIZE {
            let Some(card) = self.deck.pop() else { break };
            self.pool.push(card);
        }
        self.pool.sort_unstable();
        self.events.push(Event::PoolDealt {
            cards: self.pool.clone(),
        });
        self.chosen = vec![None; usize::from(self.players)];
        self.revealed = vec![None; usize::from(self.players)];
        self.phase = Phase::Choose;
    }

    /// [P] Reveal, then act from the lowest card to the highest.
    fn reveal(&mut self) {
        for (i, card) in self.chosen.iter().enumerate() {
            if let Some(card) = card {
                let hand = &mut self.hands[i];
                if let Some(at) = hand.iter().position(|c| c == card) {
                    hand.remove(at);
                }
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
        self.run();
    }

    /// [P] The next to act: the lowest card; ties go to the closest player
    /// on the left of whoever acted last.
    fn next_actor(&self) -> Option<usize> {
        let n = self.players;
        let distance = |seat: Seat| (seat.0 + n - self.last_actor.0 - 1) % n;
        (0..self.waiting.len()).min_by_key(|&k| {
            let (seat, card) = self.waiting[k];
            (card, distance(seat))
        })
    }

    /// Act in order until someone must choose a card or the turn is over.
    fn run(&mut self) {
        while let Some(k) = self.next_actor() {
            let (seat, card) = self.waiting.remove(k);
            self.last_actor = seat;
            let ranks = self.takeable(card);
            match ranks.as_slice() {
                [] => {
                    // [P] Stacking: no knockout.
                    self.piles[seat.index()].push(card);
                    self.events.push(Event::Stacked { seat, card });
                }
                [only] => self.take(seat, card, *only),
                _ => {
                    // ❓ A real choice is the player's move.
                    self.taking = Some((seat, card));
                    self.phase = Phase::Take;
                    return;
                }
            }
        }
        self.start_turn();
    }

    /// [P] Keep one card of `rank` from the pool and discard the played
    /// card; that rank is knocked out of the pool and the other piles.
    fn take(&mut self, seat: Seat, played: Rank, rank: Rank) {
        if let Some(at) = self.pool.iter().position(|&c| c == rank) {
            self.pool.remove(at);
        }
        self.piles[seat.index()].push(rank);
        self.discard.push(played);
        let before = self.pool.len();
        self.discard
            .extend(self.pool.iter().copied().filter(|&c| c == rank));
        self.pool.retain(|&c| c != rank);
        let pool_knocked = (before - self.pool.len()) as u8;
        let mut knocked = Vec::new();
        for (i, pile) in self.piles.iter_mut().enumerate() {
            if i == seat.index() {
                continue;
            }
            let count = pile.iter().filter(|&&c| c == rank).count();
            if count > 0 {
                pile.retain(|&c| c != rank);
                self.discard.extend(std::iter::repeat_n(rank, count));
                knocked.push((Seat(i as u8), count as u8));
            }
        }
        self.events.push(Event::Took {
            seat,
            played,
            card: rank,
            knocked,
            pool_knocked,
        });
    }

    /// [P] Each card in a pile is worth its face value. ❓ A long game ends
    /// after the deal where someone reaches the target.
    fn end_deal(&mut self) {
        let points: Vec<u16> = self
            .piles
            .iter()
            .map(|p| p.iter().map(|c| u16::from(c.value())).sum())
            .collect();
        for (score, p) in self.scores.iter_mut().zip(&points) {
            *score += p;
        }
        self.events.push(Event::DealOver {
            points,
            scores: self.scores.clone(),
        });
        let reached = self.scores.iter().any(|&s| s >= target(self.players));
        if self.config.length == Length::OneDeal || reached || self.deal_no >= MAX_DEALS {
            self.phase = Phase::Over;
        } else {
            let dealer = self.next(self.dealer);
            self.deal(dealer);
        }
    }
}

impl Game for Pickup {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-pickup",
            name: "Pickup",
            min_players: 2,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "length",
            kind: OptionKind::Choice {
                values: &["OneDeal", "Target"],
                default: "OneDeal",
            },
        }]
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            config: config.clone(),
            deal_no: 0,
            seed,
            dealer: Seat(0),
            deck: Vec::new(),
            pool: Vec::new(),
            discard: Vec::new(),
            hands: vec![Vec::new(); n],
            chosen: vec![None; n],
            revealed: vec![None; n],
            waiting: Vec::new(),
            taking: None,
            last_actor: Seat(0),
            piles: vec![Vec::new(); n],
            scores: vec![0; n],
            phase: Phase::Choose,
            events: Vec::new(),
        };
        // ❓ The first dealer is drawn from the seed.
        let dealer = Seat((seed % u64::from(players.max(1))) as u8);
        s.deal(dealer);
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
            Phase::Take => s
                .taking
                .map_or_else(Pending::none, |(seat, _)| Pending::one(seat)),
            Phase::Over => Pending::none(),
        }
    }

    /// While cards are chosen nobody learns who has already chosen.
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
            Phase::Choose => distinct(&s.hands[seat.index()])
                .into_iter()
                .map(|card| Action::Choose { card })
                .collect(),
            Phase::Take => s
                .taking
                .map(|(_, played)| s.takeable(played))
                .unwrap_or_default()
                .into_iter()
                .map(|card| Action::Take { card })
                .collect(),
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
            Action::Choose { card } => {
                s.chosen[seat.index()] = Some(card);
                if Self::pending(s).is_empty() {
                    s.events.clear();
                    s.reveal();
                }
            }
            Action::Take { card } => {
                s.events.clear();
                let (_, played) = s.taking.take().expect("a pending pick");
                s.phase = Phase::Choose;
                s.take(seat, played, card);
                s.run();
            }
        }
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
