//! Jour de chance: Continuous Pairs (James Ernest and Paul Peterson). Rules and their
//! sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply, DECK_SIZE};

pub struct ContinuousPairs;

/// [R] Cards burned after every shuffle.
pub const BURN: u8 = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// The losing score, capped at `MAX_TARGET`. `None` (or 0): the
    /// booklet's, from the player count.
    pub target: Option<u8>,
}

/// The highest losing score a config may ask for (the booklet's own
/// maximum). It keeps scores far below the 385 points of the deck, so the
/// game always ends before the cards run out.
pub const MAX_TARGET: u8 = 31;

/// [R] 60 / players + 1, kept at 11 from 6 players up.
pub fn default_target(players: u8) -> u8 {
    match players {
        2 => 31,
        3 => 21,
        4 => 16,
        5 => 13,
        _ => 11,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Take a card.
    Hit,
    /// Take `card` from `seat`'s stack as a scoring card.
    Fold { seat: Seat, card: Rank },
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// `seat` received `card` without pairing.
    Dealt { seat: Seat, card: Rank },
    /// A tie-break card that paired: discarded and replaced.
    Replaced { seat: Seat, card: Rank },
    /// `seat` paired `card` and scored it.
    Paired { seat: Seat, card: Rank },
    /// `seat` folded, taking `card` from `from`.
    Folded { seat: Seat, from: Seat, card: Rank },
    /// The discards became the draw pile.
    Reshuffled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub target: u8,
    pub supply: Supply,
    /// Face-up cards of each seat. A stack never holds two cards of a rank.
    pub stacks: Vec<Vec<Rank>>,
    /// Scoring cards of each seat; they never return to the supply.
    pub scored: Vec<Vec<Rank>>,
    /// Seat to move; `None` once the game is over.
    pub turn: Option<Seat>,
    pub loser: Option<Seat>,
    /// Since the previous decision (or since the deal).
    pub events: Vec<Event>,
}

impl State {
    /// [R] One card each; the lowest card starts, ties broken by more cards.
    pub fn deal(players: u8, target: u8, supply: Supply) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            target,
            supply,
            stacks: vec![Vec::new(); n],
            scored: vec![Vec::new(); n],
            turn: None,
            loser: None,
            events: Vec::new(),
        };
        let mut drawn: Vec<(Seat, Rank)> = Vec::new();
        for seat in (0..players).map(Seat) {
            if let Some(card) = s.draw_unpaired(seat) {
                drawn.push((seat, card));
            }
        }
        let mut tied = lowest_of(&drawn);
        // [R] Tied players get one more card each, compared among themselves.
        while tied.len() > 1 {
            let mut round: Vec<(Seat, Rank)> = Vec::new();
            for &seat in &tied {
                match s.draw_unpaired(seat) {
                    Some(card) => round.push((seat, card)),
                    None => break,
                }
            }
            // ❓4: out of cards, the first of the tied players starts.
            if round.len() < tied.len() {
                break;
            }
            tied = lowest_of(&round);
        }
        s.turn = Some(tied.first().copied().unwrap_or(Seat(0)));
        s
    }

    pub fn score(&self, seat: Seat) -> u8 {
        self.scored[seat.index()].iter().map(|c| c.value()).sum()
    }

    /// Every face-up card with the seat holding it.
    pub fn in_play(&self) -> impl Iterator<Item = (Seat, Rank)> + '_ {
        self.stacks
            .iter()
            .enumerate()
            .flat_map(|(i, stack)| stack.iter().map(move |&card| (Seat(i as u8), card)))
    }

    /// The cheapest fold.
    pub fn lowest_in_play(&self) -> Option<Rank> {
        self.in_play().map(|(_, card)| card).min()
    }

    /// Points `seat` expects to score by taking a card.
    pub fn expected_hit_cost(&self, seat: Seat) -> f64 {
        let counts = self.supply.draw_pool();
        let total: u32 = counts.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 0.0;
        }
        let weighted: u32 = self.stacks[seat.index()]
            .iter()
            .map(|card| u32::from(counts[card.index()]) * u32::from(card.value()))
            .sum();
        f64::from(weighted) / f64::from(total)
    }

    fn draw(&mut self) -> Option<Rank> {
        let before = self.supply.shuffles();
        let card = self.supply.draw();
        if self.supply.shuffles() != before {
            self.events.push(Event::Reshuffled);
        }
        card
    }

    /// [R] A card for `seat` that does not pair its stack: a pairing card is
    /// discarded and replaced. `None` when the supply cannot provide one.
    fn draw_unpaired(&mut self, seat: Seat) -> Option<Rank> {
        for _ in 0..DECK_SIZE {
            let card = self.draw()?;
            if has_rank(&self.stacks[seat.index()], card) {
                self.supply.discard([card]);
                self.events.push(Event::Replaced { seat, card });
            } else {
                self.stacks[seat.index()].push(card);
                self.events.push(Event::Dealt { seat, card });
                return Some(card);
            }
        }
        None
    }

    fn hit(&mut self, seat: Seat, card: Rank) {
        let i = seat.index();
        if has_rank(&self.stacks[i], card) {
            // [R] Score the pair's rank and keep one of its cards; only this
            // player's stack is discarded.
            let stack = std::mem::take(&mut self.stacks[i]);
            self.supply.discard(stack);
            self.scored[i].push(card);
            self.events.push(Event::Paired { seat, card });
        } else {
            self.stacks[i].push(card);
            self.events.push(Event::Dealt { seat, card });
        }
    }

    fn fold(&mut self, seat: Seat, from: Seat, at: usize) {
        let card = self.stacks[from.index()].remove(at);
        self.scored[seat.index()].push(card);
        let stack = std::mem::take(&mut self.stacks[seat.index()]);
        self.supply.discard(stack);
        self.events.push(Event::Folded { seat, from, card });
    }

    /// After `seat` scored or took a card: the game ends, or the turn moves.
    fn finish_turn(&mut self, seat: Seat) {
        if self.score(seat) >= self.target {
            // [R] The first player to reach the target loses.
            self.loser = Some(seat);
            self.turn = None;
        } else {
            self.advance(seat);
        }
    }

    /// The turn passes left. [R] A player with no cards automatically hits,
    /// and that is their turn.
    fn advance(&mut self, from: Seat) {
        let mut seat = from;
        // Each empty stack passed gets a card, so a full lap finds a player
        // with cards. Every stack empty with nothing to draw would put all 55
        // cards (385 points) among the scoring cards, which `MAX_TARGET` rules
        // out: someone reaches the target long before.
        for _ in 0..2 * usize::from(self.players) {
            seat = Seat((seat.0 + 1) % self.players);
            if !self.stacks[seat.index()].is_empty() {
                break;
            }
            // ❓3: with nothing to draw, a player without cards is skipped.
            if let Some(card) = self.draw() {
                self.stacks[seat.index()].push(card);
                self.events.push(Event::Dealt { seat, card });
            }
        }
        self.turn = Some(seat);
    }
}

fn lowest_of(cards: &[(Seat, Rank)]) -> Vec<Seat> {
    let low = cards.iter().map(|&(_, card)| card).min();
    cards
        .iter()
        .filter(|&&(_, card)| Some(card) == low)
        .map(|&(seat, _)| seat)
        .collect()
}

impl Game for ContinuousPairs {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-continuous",
            name: "Jour de chance",
            min_players: 2,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let target = config
            .target
            .filter(|&t| t > 0)
            .map(|t| t.min(MAX_TARGET))
            .unwrap_or_else(|| default_target(players));
        State::deal(players, target, Supply::new(full_deck(), BURN, seed))
    }

    fn pending(s: &State) -> Pending {
        match s.turn {
            Some(seat) => Pending::one(seat),
            None => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != Some(seat) {
            return Vec::new();
        }
        let mut actions = Vec::new();
        if s.supply.can_draw() {
            actions.push(Action::Hit);
        }
        // [R] Continuous Pairs: a fold may take any card in play.
        actions.extend(
            s.in_play()
                .map(|(from, card)| Action::Fold { seat: from, card }),
        );
        actions
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        let Some(turn) = s.turn else {
            return Err(Illegal::new("La partie est terminée."));
        };
        if turn != seat {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match *action {
            Action::Hit => {
                if !s.supply.can_draw() {
                    return Err(Illegal::new("Il n'y a plus de carte à piocher."));
                }
                s.events.clear();
                if let Some(card) = s.draw() {
                    s.hit(seat, card);
                }
            }
            Action::Fold { seat: from, card } => {
                let at = s
                    .stacks
                    .get(from.index())
                    .and_then(|stack| stack.iter().position(|&c| c == card))
                    .ok_or_else(|| Illegal::new("Cette carte n'est pas en jeu."))?;
                s.events.clear();
                s.fold(seat, from, at);
            }
        }
        s.finish_turn(seat);
        Ok(())
    }

    fn view(s: &State, _viewer: Viewer) -> View {
        View::of(s)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        // ❓1: the loser is last, everyone else tied first.
        let loser = s.loser?;
        let mut ranks = vec![0u8; usize::from(s.players)];
        ranks[loser.index()] = 1;
        Some(Outcome::Ranked(ranks))
    }
}
