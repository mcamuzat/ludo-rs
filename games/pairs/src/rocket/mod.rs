//! Rocket (James Ernest): a casino game against a dealer who plays by
//! fixed rules. Hit or fold; a pair costs you, the dealer's pair (or a 3)
//! pays you. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::View;

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply, DECK_SIZE};

pub struct Rocket;

/// ❓1: everyone bets 10, so payouts are in chips (a tenth of the bet).
pub const UNIT: u32 = 1;
/// ❓2
pub const DEFAULT_ROUNDS: u8 = 10;
pub const MAX_ROUNDS: u8 = 30;
/// [R] Reshuffle when about 20% of the deck remains.
pub const RESHUFFLE_BELOW: usize = DECK_SIZE / 5;
/// [R] A 3 is sudden death for the dealer.
pub const DEATH_CARD: u8 = 3;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds in a game, capped at `MAX_ROUNDS`. `None` (or 0): 10.
    pub rounds: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Hit,
    /// Pay the lowest card in sight and leave the round.
    Fold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    In,
    Folded,
    Busted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Play,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new round: one card to each player, then to the dealer.
    Dealt {
        round: u8,
        cards: Vec<Rank>,
        dealer: Rank,
    },
    Hit {
        seat: Seat,
        card: Rank,
    },
    /// `seat` paired `card` and paid `paid`.
    Busted {
        seat: Seat,
        card: Rank,
        paid: u32,
    },
    Folded {
        seat: Seat,
        paid: u32,
    },
    DealerHit {
        card: Rank,
    },
    /// The dealer paired (or drew a 3): each of `seats` gets `amount`.
    DealerLost {
        card: Rank,
        seats: Vec<Seat>,
        amount: u32,
    },
    Reshuffled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub rounds_total: u8,
    pub round_no: u8,
    pub supply: Supply,
    pub stacks: Vec<Vec<Rank>>,
    pub dealer: Vec<Rank>,
    pub status: Vec<Status>,
    pub turn: Option<Seat>,
    pub phase: Phase,
    /// Each player's balance against the house.
    pub chips: Vec<i32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    fn draw(&mut self) -> Option<Rank> {
        let before = self.supply.shuffles();
        let card = self.supply.draw();
        if self.supply.shuffles() != before {
            self.events.push(Event::Reshuffled);
        }
        card
    }

    /// [R] What folding costs: the lowest card in the player's stack or
    /// the dealer's, times a tenth of the bet.
    pub fn fold_cost(&self, seat: Seat) -> u32 {
        self.stacks[seat.index()]
            .iter()
            .chain(&self.dealer)
            .map(|c| u32::from(c.value()))
            .min()
            .unwrap_or(0)
            * UNIT
    }

    fn start_round(&mut self) {
        if self.round_no >= self.rounds_total {
            self.phase = Phase::Over;
            self.turn = None;
            return;
        }
        self.round_no += 1;
        for i in 0..usize::from(self.players) {
            let stack = std::mem::take(&mut self.stacks[i]);
            self.supply.discard(stack);
        }
        let dealer = std::mem::take(&mut self.dealer);
        self.supply.discard(dealer);
        // [R] Reshuffle with about 20% of the deck left.
        if self.supply.deck_len() < RESHUFFLE_BELOW {
            self.supply.reshuffle();
            self.events.push(Event::Reshuffled);
        }
        self.status = vec![Status::In; usize::from(self.players)];
        let cards: Vec<Rank> = (0..self.players).filter_map(|_| self.draw()).collect();
        for (i, &card) in cards.iter().enumerate() {
            self.stacks[i].push(card);
        }
        let Some(up) = self.draw() else {
            return;
        };
        self.dealer.push(up);
        self.events.push(Event::Dealt {
            round: self.round_no,
            cards,
            dealer: up,
        });
        // [R] A 3 kills the dealer even as her first card.
        if up.value() == DEATH_CARD {
            self.dealer_loses(up, u32::from(DEATH_CARD));
            return;
        }
        // [R] The action begins on the dealer's left.
        self.turn = Some(Seat(0));
    }

    fn dealer_loses(&mut self, card: Rank, value: u32) {
        let seats: Vec<Seat> = self
            .seats()
            .filter(|s| self.status[s.index()] == Status::In)
            .collect();
        let amount = value * UNIT;
        for s in &seats {
            self.chips[s.index()] += amount as i32;
        }
        self.events.push(Event::DealerLost {
            card,
            seats,
            amount,
        });
        self.start_round();
    }

    /// The next player still in after `seat`; after the last, the dealer.
    fn advance(&mut self, seat: Seat) {
        let next = (seat.0 + 1..self.players)
            .map(Seat)
            .find(|s| self.status[s.index()] == Status::In);
        match next {
            Some(next) => self.turn = Some(next),
            None => self.dealer_turn(),
        }
    }

    /// [R] With every player out, the round is over. Otherwise the dealer
    /// always hits: a pair or a 3 pays every surviving player; else play
    /// resumes with the first player.
    fn dealer_turn(&mut self) {
        let first = self.seats().find(|s| self.status[s.index()] == Status::In);
        let Some(first) = first else {
            self.start_round();
            return;
        };
        let Some(card) = self.draw() else {
            self.start_round();
            return;
        };
        if has_rank(&self.dealer, card) || card.value() == DEATH_CARD {
            self.dealer.push(card);
            self.dealer_loses(card, u32::from(card.value()));
            return;
        }
        self.dealer.push(card);
        self.events.push(Event::DealerHit { card });
        self.turn = Some(first);
    }

    fn hit(&mut self, seat: Seat) {
        let Some(card) = self.draw() else {
            return;
        };
        let i = seat.index();
        let paired = has_rank(&self.stacks[i], card);
        self.stacks[i].push(card);
        if paired {
            // [R] A pair costs its value times a tenth of the bet.
            let paid = u32::from(card.value()) * UNIT;
            self.chips[i] -= paid as i32;
            self.status[i] = Status::Busted;
            self.events.push(Event::Busted { seat, card, paid });
        } else {
            self.events.push(Event::Hit { seat, card });
        }
        self.advance(seat);
    }
}

impl Game for Rocket {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-rocket",
            name: "Rocket",
            min_players: 1,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let rounds_total = config
            .rounds
            .filter(|&r| r > 0)
            .map_or(DEFAULT_ROUNDS, |r| r.min(MAX_ROUNDS));
        let n = usize::from(players);
        let mut s = State {
            players,
            rounds_total,
            round_no: 0,
            supply: Supply::new(full_deck(), 0, seed),
            stacks: vec![Vec::new(); n],
            dealer: Vec::new(),
            status: vec![Status::In; n],
            turn: None,
            phase: Phase::Play,
            chips: vec![0; n],
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Play || s.turn != Some(seat) {
            return Vec::new();
        }
        // ❓4: with no card left anywhere, only folding.
        if s.supply.can_draw() {
            vec![Action::Hit, Action::Fold]
        } else {
            vec![Action::Fold]
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Il n'y a plus de carte à tirer."));
        }
        s.events.clear();
        match action {
            Action::Hit => s.hit(seat),
            Action::Fold => {
                let paid = s.fold_cost(seat);
                s.chips[seat.index()] -= paid as i32;
                s.status[seat.index()] = Status::Folded;
                s.events.push(Event::Folded { seat, paid });
                s.advance(seat);
            }
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        let ranks = s
            .chips
            .iter()
            .map(|&mine| s.chips.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
