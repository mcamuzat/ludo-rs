//! Zombie Life: the Origames variant of Pairs with refuges and a horde
//! (after James Ernest and Paul Peterson). Rules and their sources: RULES.md.

pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply};

pub struct ZombieLife;

/// [Z] Points that end the game.
pub const DEFAULT_TARGET: u8 = 21;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Points that end the game. `None` (or 0): 21.
    pub target: Option<u8>,
}

/// Where a drawn card goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    /// The cards in front of the player.
    Refuge,
    /// The cards in the middle of the table, for everyone.
    Horde,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Take the top card of the draw pile, then `Place` it.
    Draw,
    /// Put the drawn card face up in a zone that lacks its rank.
    Place { zone: Zone },
    /// Take a card of the horde into one's refuge.
    Take { card: Rank },
    /// Hide until the end of the round.
    Pass,
    /// The dealer puts one of its cards in front of the next seat to serve.
    Give { card: Rank },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// The dealer hands out the first card of each refuge.
    Dealing,
    Play,
    /// The seat to move has drawn a card and chooses where it goes.
    Placing,
    Over,
}

/// A seat in the current round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    In,
    /// Hidden: plays no more, and scores at the end of the round.
    Passed,
    /// Overrun by the zombies: scores nothing.
    Out,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// The first card of `seat`'s refuge.
    Dealt {
        seat: Seat,
        card: Rank,
    },
    Starts {
        seat: Seat,
    },
    Drew {
        seat: Seat,
        card: Rank,
    },
    Placed {
        seat: Seat,
        card: Rank,
        zone: Zone,
    },
    Took {
        seat: Seat,
        card: Rank,
    },
    /// `hidden` cards of the refuge were turned face down.
    Passed {
        seat: Seat,
        hidden: u8,
    },
    /// The card drawn was in the horde and in the refuge: `seat` is out.
    Overrun {
        seat: Seat,
        card: Rank,
    },
    /// End of a round: a seat that passed scores its face-up cards.
    Scored {
        seat: Seat,
        points: u8,
    },
    /// `seat` deals the next round.
    Dealer {
        seat: Seat,
    },
    Reshuffled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub target: u8,
    /// The current round, from 1.
    pub round: u16,
    pub supply: Supply,
    /// For the tie-breaks. Never in a view.
    pub seed: u64,
    /// Cards in front of each seat. Never two of a rank.
    pub refuges: Vec<Vec<Rank>>,
    /// Community cards. Never two of a rank.
    pub horde: Vec<Rank>,
    pub status: Vec<Status>,
    /// Cards of a passed seat's refuge that are face down. They all score
    /// the same, so which ones does not matter (❓2).
    pub hidden: Vec<u8>,
    /// Seats overrun in the current round, in order.
    pub eliminated: Vec<Seat>,
    pub scores: Vec<u16>,
    pub phase: Phase,
    /// Seat to move; the dealer while dealing.
    pub turn: Option<Seat>,
    /// Placing: the card just drawn.
    pub drawn: Option<Rank>,
    /// Dealing: the dealer's cards not yet given.
    pub dealing: Vec<Rank>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u16) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

const START_STREAM: u64 = 0x5851_F42D_4C95_7F2D;
const DEALER_STREAM: u64 = 0x1405_7B7E_F767_814F;

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    /// Seats still playing the round, in seat order.
    pub fn active(&self) -> Vec<Seat> {
        self.seats()
            .filter(|s| self.status[s.index()] == Status::In)
            .collect()
    }

    /// Seats after `seat` around the table, `seat` itself last.
    fn around_from(&self, seat: Seat) -> impl Iterator<Item = Seat> + '_ {
        (1..=self.players).map(move |k| Seat((seat.0 + k) % self.players))
    }

    /// Dealing: the seat the next card goes to — from the dealer's left,
    /// the dealer last.
    pub fn deal_to(&self) -> Option<Seat> {
        if self.phase != Phase::Dealing {
            return None;
        }
        let dealer = self.turn?;
        self.around_from(dealer)
            .find(|s| self.refuges[s.index()].is_empty())
    }

    /// One of `tied`, from the seed (❓5, ❓6).
    fn pick(&self, tied: &[Seat], stream: u64) -> Option<Seat> {
        match tied {
            [] => None,
            [only] => Some(*only),
            _ => {
                let mut rng = ChaCha8Rng::seed_from_u64(mix(self.seed, self.round) ^ stream);
                // A `u32` sample: `usize` would draw differently on wasm32.
                Some(tied[rng.gen_range(0..tied.len() as u32) as usize])
            }
        }
    }

    fn draw(&mut self) -> Option<Rank> {
        let before = self.supply.shuffles();
        let card = self.supply.draw();
        if self.supply.shuffles() != before {
            self.events.push(Event::Reshuffled);
        }
        card
    }

    /// [Z] The lowest card starts.
    fn begin_play(&mut self) {
        let low = self.refuges.iter().filter_map(|r| r.first().copied()).min();
        let tied: Vec<Seat> = self
            .seats()
            .filter(|s| self.refuges[s.index()].first().copied() == low)
            .collect();
        let first = self.pick(&tied, START_STREAM).unwrap_or(Seat(0));
        self.events.push(Event::Starts { seat: first });
        self.phase = Phase::Play;
        self.turn = Some(first);
    }

    fn give(&mut self, card: Rank) {
        let (Some(to), Some(at)) = (self.deal_to(), self.dealing.iter().position(|&c| c == card))
        else {
            return;
        };
        self.dealing.remove(at);
        self.refuges[to.index()].push(card);
        self.events.push(Event::Dealt { seat: to, card });
    }

    /// Gives the cards that need no choice (all of one rank), then starts
    /// the round once everyone is served.
    fn deal_on(&mut self) {
        while let Some(&card) = self.dealing.first() {
            if self.dealing.iter().any(|&c| c != card) {
                return;
            }
            let before = self.dealing.len();
            self.give(card);
            if self.dealing.len() == before {
                // Nobody left to serve: never the case with one card per seat.
                let rest = std::mem::take(&mut self.dealing);
                self.supply.discard(rest);
            }
        }
        self.begin_play();
    }

    /// [Z] `seat` is overrun: its refuge and the card are discarded, and
    /// every hidden seat turns one card back up.
    fn overrun(&mut self, seat: Seat, card: Rank) {
        let i = seat.index();
        self.status[i] = Status::Out;
        self.eliminated.push(seat);
        let refuge = std::mem::take(&mut self.refuges[i]);
        self.supply.discard(refuge);
        self.supply.discard([card]);
        for hidden in &mut self.hidden {
            *hidden = hidden.saturating_sub(1);
        }
        self.events.push(Event::Overrun { seat, card });
    }

    /// After `seat` acted: the round ends, or the turn passes left.
    fn advance(&mut self, seat: Seat) {
        let next = self
            .around_from(seat)
            .find(|s| self.status[s.index()] == Status::In);
        match next {
            Some(next) => {
                self.phase = Phase::Play;
                self.turn = Some(next);
            }
            None => self.end_round(),
        }
    }

    /// [Z] Seats that passed score a point per face-up card; then the game
    /// ends, or the lowest score deals the next round.
    fn end_round(&mut self) {
        for seat in self.seats().collect::<Vec<_>>() {
            let i = seat.index();
            if self.status[i] != Status::Passed {
                continue;
            }
            let points = (self.refuges[i].len() as u8).saturating_sub(self.hidden[i]);
            self.scores[i] += u16::from(points);
            self.events.push(Event::Scored { seat, points });
        }
        if self.scores.iter().any(|&s| s >= u16::from(self.target)) {
            self.phase = Phase::Over;
            self.turn = None;
            return;
        }
        // ❓3: the horde goes to the discards with the refuges.
        for refuge in &mut self.refuges {
            self.supply.discard(std::mem::take(refuge));
        }
        self.supply.discard(std::mem::take(&mut self.horde));

        let low = self.scores.iter().copied().min();
        let tied: Vec<Seat> = self
            .seats()
            .filter(|s| Some(self.scores[s.index()]) == low)
            .collect();
        // [Z] Among equal scores, the first eliminated deals.
        let dealer = self
            .eliminated
            .iter()
            .copied()
            .find(|s| tied.contains(s))
            .or_else(|| self.pick(&tied, DEALER_STREAM))
            .unwrap_or(Seat(0));

        let n = usize::from(self.players);
        self.round = self.round.saturating_add(1);
        self.status = vec![Status::In; n];
        self.hidden = vec![0; n];
        self.eliminated.clear();
        self.events.push(Event::Dealer { seat: dealer });
        for _ in 0..n {
            if let Some(card) = self.draw() {
                self.dealing.push(card);
            }
        }
        self.phase = Phase::Dealing;
        self.turn = Some(dealer);
        self.deal_on();
    }

    fn play_actions(&self, seat: Seat) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.supply.can_draw() {
            actions.push(Action::Draw);
        }
        // [Z] The last player in may not take from the horde.
        if self.active().len() > 1 {
            let refuge = &self.refuges[seat.index()];
            let mut cards: Vec<Rank> = self
                .horde
                .iter()
                .copied()
                .filter(|&card| !has_rank(refuge, card))
                .collect();
            cards.sort_unstable();
            actions.extend(cards.into_iter().map(|card| Action::Take { card }));
        }
        actions.push(Action::Pass);
        actions
    }

    fn place_actions(&self, seat: Seat) -> Vec<Action> {
        let Some(card) = self.drawn else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        if !has_rank(&self.refuges[seat.index()], card) {
            actions.push(Action::Place { zone: Zone::Refuge });
        }
        if !has_rank(&self.horde, card) {
            actions.push(Action::Place { zone: Zone::Horde });
        }
        actions
    }

    fn deal_actions(&self) -> Vec<Action> {
        let mut cards = self.dealing.clone();
        cards.sort_unstable();
        cards.dedup();
        cards
            .into_iter()
            .map(|card| Action::Give { card })
            .collect()
    }

    /// `action` is legal for `seat`.
    fn play(&mut self, seat: Seat, action: &Action) {
        self.events.clear();
        let i = seat.index();
        match *action {
            Action::Draw => {
                let Some(card) = self.draw() else {
                    return;
                };
                if has_rank(&self.refuges[i], card) && has_rank(&self.horde, card) {
                    self.overrun(seat, card);
                    self.advance(seat);
                } else {
                    self.events.push(Event::Drew { seat, card });
                    self.drawn = Some(card);
                    self.phase = Phase::Placing;
                }
            }
            Action::Place { zone } => {
                let Some(card) = self.drawn.take() else {
                    return;
                };
                match zone {
                    Zone::Refuge => self.refuges[i].push(card),
                    Zone::Horde => self.horde.push(card),
                }
                self.events.push(Event::Placed { seat, card, zone });
                self.advance(seat);
            }
            Action::Take { card } => {
                if let Some(at) = self.horde.iter().position(|&c| c == card) {
                    self.horde.remove(at);
                    self.refuges[i].push(card);
                }
                self.events.push(Event::Took { seat, card });
                self.advance(seat);
            }
            Action::Pass => {
                self.status[i] = Status::Passed;
                // ❓1: one card per player still in, the passer not counted.
                let others = self.active().len();
                let hidden = self.refuges[i].len().min(others) as u8;
                self.hidden[i] = hidden;
                self.events.push(Event::Passed { seat, hidden });
                self.advance(seat);
            }
            Action::Give { card } => {
                self.give(card);
                self.deal_on();
            }
        }
    }
}

impl Game for ZombieLife {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-zombie-life",
            name: "Zombie Life",
            min_players: 3,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            target: config.target.filter(|&t| t > 0).unwrap_or(DEFAULT_TARGET),
            round: 1,
            supply: Supply::new(full_deck(), 0, seed),
            seed,
            refuges: vec![Vec::new(); n],
            horde: Vec::new(),
            status: vec![Status::In; n],
            hidden: vec![0; n],
            eliminated: Vec::new(),
            scores: vec![0; n],
            phase: Phase::Play,
            turn: None,
            drawn: None,
            dealing: Vec::new(),
            events: Vec::new(),
        };
        // [Z] First round: one card face up in front of each player.
        for seat in s.seats().collect::<Vec<_>>() {
            if let Some(card) = s.supply.draw() {
                s.refuges[seat.index()].push(card);
                s.events.push(Event::Dealt { seat, card });
            }
        }
        s.begin_play();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Over => Pending::none(),
            _ => s.turn.map_or_else(Pending::none, Pending::one),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat.index() >= usize::from(s.players) || s.turn != Some(seat) {
            return Vec::new();
        }
        match s.phase {
            Phase::Dealing => s.deal_actions(),
            Phase::Play => s.play_actions(seat),
            Phase::Placing => s.place_actions(seat),
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        s.play(seat, action);
        Ok(())
    }

    fn view(s: &State, _viewer: Viewer) -> View {
        View::of(s)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        // ❓7: equal scores stay tied.
        let ranks = s
            .scores
            .iter()
            .map(|&mine| s.scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
