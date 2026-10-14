//! Captain's Orders (James Ernest): the Captain holds the lowest card and
//! wins by running out of cards; the Mutineers win by unmasking a Captain
//! or by holding the most points. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, lowest, Rank, Supply};

pub struct CaptainsOrders;

/// [C] Six cards each.
pub const HAND_SIZE: usize = 6;
/// [C] Rounds to win the game (❓6: configurable).
pub const DEFAULT_WINS: u8 = 3;
pub const MAX_WINS: u8 = 9;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds to win, capped at `MAX_WINS`. `None` (or 0): 3.
    pub wins: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a hand card as an Order; while an Order is out, belay it with
    /// a lower card.
    Order { card: Rank },
    /// Follow the Order: pass its owner a card of its rank or higher.
    Follow { card: Rank },
    /// Pass a card face down; `to` then discards one.
    Pass { card: Rank, to: Seat },
    /// Ask `from` for a card, then discard one.
    Ask { from: Seat },
    /// The card given to whoever asked.
    Give { card: Rank },
    /// Face down, after receiving a card.
    Discard { card: Rank },
    /// Name `seat` as the Captain; ends the round.
    Accuse { seat: Seat },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Phase {
    /// No Order on the table: the seat to move plays, passes, asks or accuses.
    Turn,
    /// An Order is out: the seat to move follows, belays or accuses.
    Respond,
    /// The seat to move gives `asker` a card.
    Give {
        asker: Seat,
    },
    /// The seat to move discards; then `after` plays.
    Discard {
        after: Seat,
    },
    Over,
}

/// The Order on the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub card: Rank,
    pub owner: Seat,
}

/// What happened this round. A card some seats cannot see is masked in
/// their view (`None`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new Order; `belay`: it replaced a higher one.
    Ordered {
        seat: Seat,
        card: Rank,
        belay: bool,
    },
    Followed {
        seat: Seat,
        owner: Seat,
        card: Option<Rank>,
    },
    /// The Order went all the way round; its owner discards it.
    OrderDone {
        owner: Seat,
        card: Rank,
    },
    Passed {
        from: Seat,
        to: Seat,
        card: Option<Rank>,
    },
    Asked {
        from: Seat,
        to: Seat,
    },
    Gave {
        from: Seat,
        to: Seat,
        card: Option<Rank>,
    },
    Discarded {
        seat: Seat,
        card: Option<Rank>,
    },
}

/// How a round ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Ending {
    /// `accuser` named `target`; `right`: a Mutineer named a Captain.
    Accused {
        accuser: Seat,
        target: Seat,
        right: bool,
    },
    /// `seat` ran out of cards with `card`; `right`: it was a Captain.
    Emptied { seat: Seat, card: Rank, right: bool },
}

/// A finished round, every hand revealed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundResult {
    pub round: u8,
    pub ending: Ending,
    pub hands: Vec<Vec<Rank>>,
    pub captains: Vec<Seat>,
    pub points: Vec<u32>,
    pub winners: Vec<Seat>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub wins_needed: u8,
    /// The current round, from 1.
    pub round: u8,
    pub dealer: Seat,
    pub wins: Vec<u8>,
    /// Its draw pile holds the undealt cards; face-down discards go to it.
    pub supply: Supply,
    /// For each round's deal. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// The face-down discards of each seat this round (only it knows them).
    pub discarded: Vec<Vec<Rank>>,
    pub order: Option<Order>,
    pub phase: Phase,
    pub turn: Option<Seat>,
    /// This round's events, with every card.
    pub log: Vec<Event>,
    pub last_round: Option<RoundResult>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

fn take(hand: &mut Vec<Rank>, card: Rank) {
    if let Some(at) = hand.iter().position(|&c| c == card) {
        hand.remove(at);
    }
}

fn distinct(hand: &[Rank]) -> Vec<Rank> {
    let mut cards = hand.to_vec();
    cards.sort_unstable();
    cards.dedup();
    cards
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    fn others(&self, seat: Seat) -> impl Iterator<Item = Seat> {
        self.seats().filter(move |&s| s != seat)
    }

    /// [C] The seats holding the lowest card of all hands.
    pub fn captains(&self) -> Vec<Seat> {
        let low = self.hands.iter().filter_map(|h| lowest(h)).min();
        self.seats()
            .filter(|s| low.is_some() && lowest(&self.hands[s.index()]) == low)
            .collect()
    }

    pub fn points(&self, seat: Seat) -> u32 {
        self.hands[seat.index()]
            .iter()
            .map(|c| u32::from(c.value()))
            .sum()
    }

    /// [C] Deal six cards each; the seat left of the dealer plays first.
    /// Leaves `last_round` alone.
    fn start_round(&mut self) {
        let n = usize::from(self.players);
        self.round += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.round));
        self.hands = vec![Vec::new(); n];
        for _ in 0..HAND_SIZE {
            for i in 0..n {
                if let Some(card) = self.supply.draw() {
                    self.hands[i].push(card);
                }
            }
        }
        self.discarded = vec![Vec::new(); n];
        self.order = None;
        self.log.clear();
        self.phase = Phase::Turn;
        self.turn = Some(self.next(self.dealer));
    }

    /// Scores the round, counts its wins, and deals the next one (or ends
    /// the game).
    fn end_round(&mut self, ending: Ending) {
        let captains = self.captains();
        // [C] Whoever was wrong cannot win on points.
        let excluded = match ending {
            Ending::Accused { accuser, .. } => accuser,
            Ending::Emptied { seat, .. } => seat,
        };
        let points: Vec<u32> = self.seats().map(|s| self.points(s)).collect();
        let winners = match ending {
            Ending::Emptied {
                seat, right: true, ..
            } => vec![seat],
            Ending::Accused {
                accuser,
                right: true,
                ..
            } => vec![accuser],
            // [C] Otherwise the Mutineer with the most points wins (❓4:
            // nobody when every other seat is a Captain).
            _ => {
                let mutineers: Vec<Seat> = self
                    .seats()
                    .filter(|s| !captains.contains(s) && *s != excluded)
                    .collect();
                let best = mutineers.iter().map(|s| points[s.index()]).max();
                mutineers
                    .into_iter()
                    .filter(|s| Some(points[s.index()]) == best)
                    .collect()
            }
        };
        for w in &winners {
            self.wins[w.index()] += 1;
        }
        self.last_round = Some(RoundResult {
            round: self.round,
            ending,
            hands: self.hands.clone(),
            captains,
            points,
            winners,
        });
        if self.wins.iter().any(|&w| w >= self.wins_needed) {
            self.phase = Phase::Over;
            self.turn = None;
            self.order = None;
        } else {
            self.dealer = self.next(self.dealer);
            self.start_round();
        }
    }

    /// [C] Whoever empties their hand reveals `card` and declares
    /// themselves Captain (❓1: always). ❓3: right when no other hand held
    /// a lower card; `to` received it and keeps it for scoring.
    fn emptied(&mut self, seat: Seat, card: Rank, to: Option<Seat>) {
        let right = self.others(seat).all(|s| {
            let mut hand = self.hands[s.index()].clone();
            if Some(s) == to {
                take(&mut hand, card);
            }
            lowest(&hand).is_none_or(|low| card <= low)
        });
        self.end_round(Ending::Emptied { seat, card, right });
    }

    /// Moves `card` from `from` to `to`; ends the round if `from` is empty.
    /// Returns whether play goes on.
    fn hand_over(&mut self, from: Seat, to: Seat, card: Rank) -> bool {
        take(&mut self.hands[from.index()], card);
        self.hands[to.index()].push(card);
        if self.hands[from.index()].is_empty() {
            self.emptied(from, card, Some(to));
            return false;
        }
        true
    }

    /// [C] Every hand is revealed. A Mutineer naming a Captain wins;
    /// otherwise the accuser cannot win.
    fn accuse(&mut self, accuser: Seat, target: Seat) {
        let captains = self.captains();
        let right = !captains.contains(&accuser) && captains.contains(&target);
        self.end_round(Ending::Accused {
            accuser,
            target,
            right,
        });
    }

    /// Hands the Order to the seat after `after`; once it went round, its
    /// owner discards it.
    fn respond_next(&mut self, after: Seat) {
        let Some(order) = self.order else {
            return;
        };
        let next = self.next(after);
        if next == order.owner {
            // [C] Back to its owner: discarded, and the turn passes left.
            self.supply.discard([order.card]);
            self.log.push(Event::OrderDone {
                owner: order.owner,
                card: order.card,
            });
            self.order = None;
            self.phase = Phase::Turn;
            self.turn = Some(self.next(order.owner));
        } else {
            self.turn = Some(next);
        }
    }

    fn turn_actions(&self, seat: Seat) -> Vec<Action> {
        let cards = distinct(&self.hands[seat.index()]);
        let mut actions: Vec<Action> = cards.iter().map(|&card| Action::Order { card }).collect();
        for &card in &cards {
            actions.extend(self.others(seat).map(|to| Action::Pass { card, to }));
        }
        actions.extend(self.others(seat).map(|from| Action::Ask { from }));
        actions.extend(self.others(seat).map(|seat| Action::Accuse { seat }));
        actions
    }

    fn respond_actions(&self, seat: Seat, order: Order) -> Vec<Action> {
        let mut actions: Vec<Action> = distinct(&self.hands[seat.index()])
            .into_iter()
            .map(|card| {
                if card >= order.card {
                    Action::Follow { card }
                } else {
                    Action::Order { card }
                }
            })
            .collect();
        actions.extend(self.others(seat).map(|seat| Action::Accuse { seat }));
        actions
    }

    fn play(&mut self, seat: Seat, action: &Action) {
        let i = seat.index();
        match *action {
            Action::Accuse { seat: target } => self.accuse(seat, target),
            Action::Order { card } => {
                take(&mut self.hands[i], card);
                let belay = self.order.is_some();
                if let Some(old) = self.order.take() {
                    self.supply.discard([old.card]);
                }
                self.log.push(Event::Ordered { seat, card, belay });
                if self.hands[i].is_empty() {
                    self.emptied(seat, card, None);
                    return;
                }
                self.order = Some(Order { card, owner: seat });
                self.phase = Phase::Respond;
                self.respond_next(seat);
            }
            Action::Follow { card } => {
                let Some(order) = self.order else {
                    return;
                };
                self.log.push(Event::Followed {
                    seat,
                    owner: order.owner,
                    card: Some(card),
                });
                if self.hand_over(seat, order.owner, card) {
                    self.respond_next(seat);
                }
            }
            Action::Pass { card, to } => {
                self.log.push(Event::Passed {
                    from: seat,
                    to,
                    card: Some(card),
                });
                if self.hand_over(seat, to, card) {
                    self.phase = Phase::Discard {
                        after: self.next(seat),
                    };
                    self.turn = Some(to);
                }
            }
            Action::Ask { from } => {
                self.log.push(Event::Asked {
                    from: seat,
                    to: from,
                });
                self.phase = Phase::Give { asker: seat };
                self.turn = Some(from);
            }
            Action::Give { card } => {
                let Phase::Give { asker } = self.phase else {
                    return;
                };
                self.log.push(Event::Gave {
                    from: seat,
                    to: asker,
                    card: Some(card),
                });
                if self.hand_over(seat, asker, card) {
                    self.phase = Phase::Discard {
                        after: self.next(asker),
                    };
                    self.turn = Some(asker);
                }
            }
            Action::Discard { card } => {
                let Phase::Discard { after } = self.phase else {
                    return;
                };
                take(&mut self.hands[i], card);
                self.discarded[i].push(card);
                self.supply.discard_unseen([card]);
                self.log.push(Event::Discarded {
                    seat,
                    card: Some(card),
                });
                self.phase = Phase::Turn;
                self.turn = Some(after);
            }
        }
    }
}

impl Game for CaptainsOrders {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-captains-orders",
            name: "Captain's Orders",
            min_players: 4,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let wins_needed = config
            .wins
            .filter(|&w| w > 0)
            .map_or(DEFAULT_WINS, |w| w.min(MAX_WINS));
        // The first round is dealt by the last seat: seat 0 plays first.
        let mut s = State {
            players,
            wins_needed,
            round: 0,
            dealer: Seat(players.saturating_sub(1)),
            wins: vec![0; usize::from(players)],
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: Vec::new(),
            discarded: Vec::new(),
            order: None,
            phase: Phase::Turn,
            turn: None,
            log: Vec::new(),
            last_round: None,
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != Some(seat) || seat.index() >= usize::from(s.players) {
            return Vec::new();
        }
        let hand = &s.hands[seat.index()];
        match (s.phase, s.order) {
            (Phase::Turn, _) => s.turn_actions(seat),
            (Phase::Respond, Some(order)) => s.respond_actions(seat, order),
            (Phase::Give { .. }, _) => distinct(hand)
                .into_iter()
                .map(|card| Action::Give { card })
                .collect(),
            (Phase::Discard { .. }, _) => distinct(hand)
                .into_iter()
                .map(|card| Action::Discard { card })
                .collect(),
            (Phase::Respond, None) | (Phase::Over, _) => Vec::new(),
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

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        // ❓6: ranked by rounds won; equal counts stay tied.
        let ranks = s
            .wins
            .iter()
            .map(|&mine| s.wins.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
