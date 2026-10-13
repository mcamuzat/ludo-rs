//! L'Auberge des Pirates: multiplayer Regent without chips (Origames
//! edition of a James Ernest and Paul Peterson game). Rules and their
//! sources: RULES.md.

mod determinize;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, pair_in, Rank, Supply};

pub struct Auberge;

/// [A] Cards dealt to each player at the start of a round.
pub const HAND: usize = 3;
/// ❓2
pub const DEFAULT_ROUNDS: u8 = 5;
pub const MAX_ROUNDS: u8 = 15;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds in a game, capped at `MAX_ROUNDS`. `None` (or 0): 5.
    pub rounds: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Setup: the hand card to put face up.
    Place { card: Rank },
    /// Take a card in hand, unseen.
    Draw,
    /// Play a hand card face up in front of `target`.
    Attack { target: Seat, card: Rank },
    /// Put the top card of the draw pile face up in front of `target`.
    Dare { target: Seat },
    /// Claim `target` holds a pair.
    Accuse { target: Seat },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone chooses a card to place, at the same time.
    Setup,
    Play,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Setup is over: `seat` had chosen `card`.
    Placed {
        seat: Seat,
        card: Rank,
    },
    Starts {
        seat: Seat,
    },
    Drew {
        seat: Seat,
    },
    Attacked {
        seat: Seat,
        target: Seat,
        card: Rank,
    },
    Dared {
        seat: Seat,
        target: Seat,
        card: Rank,
        /// The card paired `target`'s face-up cards: `seat` is out.
        paired: bool,
    },
    Accused {
        seat: Seat,
        target: Seat,
        guilty: bool,
        /// The hand shown. Views empty it for everyone but `seat` and `target`.
        hand: Vec<Rank>,
    },
    Out {
        seat: Seat,
    },
    /// `None`: the round was stuck and nobody won it.
    RoundWon {
        seat: Option<Seat>,
    },
    Reshuffled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub rounds: u8,
    /// The current round, from 1.
    pub round: u8,
    pub supply: Supply,
    /// For each round's deal and tie-break. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// Face-up cards in front of each seat. Never two of a rank.
    pub fronts: Vec<Vec<Rank>>,
    /// Setup: the card each seat has chosen, still in its hand.
    pub placed: Vec<Option<Rank>>,
    /// Eliminated from the current round.
    pub out: Vec<bool>,
    /// Face-down discards each seat has seen (a hand shown by a right
    /// accusation, its own discarded hand). Forgotten at a reshuffle.
    pub known: Vec<Vec<Rank>>,
    pub phase: Phase,
    /// Seat to move during play.
    pub turn: Option<Seat>,
    /// Rounds won.
    pub wins: Vec<u8>,
    /// Since the previous decision in play, or since the last reveal.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

const TIE_STREAM: u64 = 0x5851_F42D_4C95_7F2D;

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    /// Seats still in the round, in seat order.
    pub fn alive(&self) -> Vec<Seat> {
        self.seats().filter(|s| !self.out[s.index()]).collect()
    }

    /// ❓7: the next seat still in the round.
    fn left_of(&self, seat: Seat) -> Seat {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| !self.out[s.index()])
            .unwrap_or(seat)
    }

    /// The pair `seat` holds, between its hand and its face-up cards.
    pub fn own_pair(&self, seat: Seat) -> Option<Rank> {
        let mut cards = self.hands[seat.index()].clone();
        cards.extend(&self.fronts[seat.index()]);
        pair_in(&cards)
    }

    /// Per rank, the cards `observer` cannot place: not face up, not in the
    /// seen discards, not in its hand, not among the discards it has seen.
    pub fn unlocated_for(&self, observer: Option<Seat>) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let own = observer
            .into_iter()
            .flat_map(|o| self.hands[o.index()].iter().chain(&self.known[o.index()]));
        for card in self
            .fronts
            .iter()
            .flatten()
            .chain(self.supply.seen())
            .chain(own)
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// The share of the cards `observer` cannot place that would pair
    /// `cards`.
    fn share(&self, observer: Option<Seat>, cards: &[Rank]) -> f64 {
        let pool = self.unlocated_for(observer);
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 0.0;
        }
        let mut ranks = cards.to_vec();
        ranks.sort_unstable();
        ranks.dedup();
        let matching: u32 = ranks.iter().map(|c| u32::from(pool[c.index()])).sum();
        f64::from(matching) / f64::from(total)
    }

    /// Estimated chance that `target` holds a pair, from what `observer`
    /// sees: its hidden cards taken as drawn at random from the cards
    /// `observer` cannot place.
    pub fn pair_chance(&self, observer: Option<Seat>, target: Seat) -> f64 {
        let hidden = self.hands[target.index()].len() as f64;
        if hidden == 0.0 {
            return 0.0;
        }
        let pool = self.unlocated_for(observer);
        let total: f64 = pool.iter().map(|&c| f64::from(c)).sum();
        if total < 2.0 {
            return 0.0;
        }
        let with_front = self.share(observer, &self.fronts[target.index()]);
        let alike: f64 = pool
            .iter()
            .map(|&c| f64::from(c) * (f64::from(c) - 1.0).max(0.0))
            .sum::<f64>()
            / (total * (total - 1.0));
        let clean =
            (1.0 - with_front).powf(hidden) * (1.0 - alike).powf(hidden * (hidden - 1.0) / 2.0);
        1.0 - clean
    }

    /// Chance that a dare on `target` pairs its face-up cards, eliminating
    /// the darer.
    pub fn dare_risk(&self, observer: Option<Seat>, target: Seat) -> f64 {
        self.share(observer, &self.fronts[target.index()])
    }

    /// Chance that the card `seat` draws gives it a pair.
    pub fn draw_risk(&self, seat: Seat) -> f64 {
        let mut cards = self.hands[seat.index()].clone();
        cards.extend(&self.fronts[seat.index()]);
        self.share(Some(seat), &cards)
    }

    fn draw(&mut self) -> Option<Rank> {
        let before = self.supply.shuffles();
        let card = self.supply.draw();
        if self.supply.shuffles() != before {
            // The discards are in the draw pile again: nobody knows where
            // the cards they saw went.
            self.events.push(Event::Reshuffled);
            self.known.iter_mut().for_each(Vec::clear);
        }
        card
    }

    /// [A] Three cards each, then everyone places one. Leaves `events` alone.
    fn start_round(&mut self) {
        let n = usize::from(self.players);
        self.round += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.round));
        self.hands = vec![Vec::new(); n];
        for _ in 0..HAND {
            for i in 0..n {
                if let Some(card) = self.supply.draw() {
                    self.hands[i].push(card);
                }
            }
        }
        self.fronts = vec![Vec::new(); n];
        self.placed = vec![None; n];
        self.out = vec![false; n];
        self.known = vec![Vec::new(); n];
        self.phase = Phase::Setup;
        self.turn = None;
    }

    /// [A] Everyone has chosen: the cards are turned up and the lowest starts.
    fn reveal(&mut self) {
        self.events.clear();
        let mut shown: Vec<(Seat, Rank)> = Vec::new();
        for seat in self.seats().collect::<Vec<_>>() {
            let i = seat.index();
            let Some(card) = self.placed[i].take() else {
                continue;
            };
            let Some(at) = self.hands[i].iter().position(|&c| c == card) else {
                continue;
            };
            self.hands[i].remove(at);
            self.fronts[i].push(card);
            self.events.push(Event::Placed { seat, card });
            shown.push((seat, card));
        }
        let low = shown.iter().map(|&(_, card)| card).min();
        let tied: Vec<Seat> = shown
            .iter()
            .filter(|&&(_, card)| Some(card) == low)
            .map(|&(seat, _)| seat)
            .collect();
        // ❓1: a tie is drawn by lot.
        let first = match tied.as_slice() {
            [] => Seat(0),
            [only] => *only,
            _ => {
                let mut rng = ChaCha8Rng::seed_from_u64(mix(self.seed, self.round) ^ TIE_STREAM);
                tied[rng.gen_range(0..tied.len())]
            }
        };
        self.events.push(Event::Starts { seat: first });
        self.phase = Phase::Play;
        self.pass_turn(first);
    }

    /// ❓3: face-up cards are discarded face up, the hand face down.
    fn eliminate(&mut self, seat: Seat) {
        let i = seat.index();
        self.out[i] = true;
        let front = std::mem::take(&mut self.fronts[i]);
        self.supply.discard(front);
        let hand = std::mem::take(&mut self.hands[i]);
        self.known[i].extend(&hand);
        self.supply.discard_unseen(hand);
        self.events.push(Event::Out { seat });
    }

    /// `next` moves, unless the round is over.
    fn pass_turn(&mut self, next: Seat) {
        if let [winner] = self.alive().as_slice() {
            // [A] The last player standing wins.
            return self.end_round(Some(*winner));
        }
        self.turn = Some(next);
        if self.play_actions(next).is_empty() {
            // ❓5: nothing to draw and nothing to play.
            self.end_round(None);
        }
    }

    fn end_round(&mut self, winner: Option<Seat>) {
        if let Some(seat) = winner {
            self.wins[seat.index()] += 1;
        }
        self.events.push(Event::RoundWon { seat: winner });
        if self.round >= self.rounds {
            self.phase = Phase::Over;
            self.turn = None;
        } else {
            self.start_round();
        }
    }

    /// What `seat` may do on its turn.
    fn play_actions(&self, seat: Seat) -> Vec<Action> {
        let others: Vec<Seat> = self.alive().into_iter().filter(|&s| s != seat).collect();
        let mut actions = Vec::new();
        // ❓5: with nothing to draw, drawing and daring are off.
        if self.supply.can_draw() {
            actions.push(Action::Draw);
            actions.extend(others.iter().map(|&target| Action::Dare { target }));
        }
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        for &target in &others {
            for &card in &cards {
                // [A] An attack may not make a visible pair.
                if !has_rank(&self.fronts[target.index()], card) {
                    actions.push(Action::Attack { target, card });
                }
            }
        }
        // ❓4: a player without hand cards cannot hold a pair.
        actions.extend(
            others
                .iter()
                .filter(|t| !self.hands[t.index()].is_empty())
                .map(|&target| Action::Accuse { target }),
        );
        actions
    }

    fn play(&mut self, seat: Seat, action: &Action) {
        self.events.clear();
        let next = match *action {
            // Not a play action: `apply` routes it elsewhere.
            Action::Place { .. } => seat,
            Action::Draw => {
                if let Some(card) = self.draw() {
                    self.hands[seat.index()].push(card);
                }
                self.events.push(Event::Drew { seat });
                self.left_of(seat)
            }
            Action::Attack { target, card } => {
                let hand = &mut self.hands[seat.index()];
                if let Some(at) = hand.iter().position(|&c| c == card) {
                    hand.remove(at);
                    self.fronts[target.index()].push(card);
                }
                self.events.push(Event::Attacked { seat, target, card });
                target
            }
            Action::Dare { target } => match self.draw() {
                Some(card) => {
                    let paired = has_rank(&self.fronts[target.index()], card);
                    self.events.push(Event::Dared {
                        seat,
                        target,
                        card,
                        paired,
                    });
                    if paired {
                        // [A] The darer is out and the card is discarded.
                        self.supply.discard([card]);
                        self.eliminate(seat);
                        self.left_of(seat)
                    } else {
                        self.fronts[target.index()].push(card);
                        target
                    }
                }
                None => seat,
            },
            Action::Accuse { target } => {
                let hand = self.hands[target.index()].clone();
                let guilty = self.own_pair(target).is_some();
                self.events.push(Event::Accused {
                    seat,
                    target,
                    guilty,
                    hand: hand.clone(),
                });
                if guilty {
                    // The accuser has seen the hand that now goes face down.
                    self.known[seat.index()].extend(&hand);
                    self.eliminate(target);
                    self.left_of(seat)
                } else {
                    self.eliminate(seat);
                    target
                }
            }
        };
        self.pass_turn(next);
    }
}

impl Game for Auberge {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-auberge",
            name: "L'Auberge des Pirates",
            min_players: 2,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let rounds = config
            .rounds
            .filter(|&r| r > 0)
            .map_or(DEFAULT_ROUNDS, |r| r.min(MAX_ROUNDS));
        let mut s = State {
            players,
            rounds,
            round: 0,
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: Vec::new(),
            fronts: Vec::new(),
            placed: Vec::new(),
            out: Vec::new(),
            known: Vec::new(),
            phase: Phase::Setup,
            turn: None,
            wins: vec![0; n],
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Setup => Pending(
                s.seats()
                    .filter(|x| s.placed[x.index()].is_none())
                    .collect(),
            ),
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    /// During setup nobody learns who has already chosen: a seat only sees
    /// itself pending.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        match (s.phase, viewer) {
            (Phase::Setup, Viewer::Seat(seat)) => match s.placed.get(seat.index()) {
                Some(None) => Pending::one(seat),
                _ => Pending::none(),
            },
            (Phase::Setup, Viewer::Spectator) => Pending::none(),
            _ => Self::pending(s),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat.index() >= usize::from(s.players) {
            return Vec::new();
        }
        match s.phase {
            Phase::Setup if s.placed[seat.index()].is_none() => {
                let mut cards = s.hands[seat.index()].clone();
                cards.sort_unstable();
                cards.dedup();
                cards
                    .into_iter()
                    .map(|card| Action::Place { card })
                    .collect()
            }
            Phase::Play if s.turn == Some(seat) => s.play_actions(seat),
            _ => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new(match s.phase {
                Phase::Setup => "Tu as déjà posé ta carte.",
                _ => "Ce n'est pas ton tour.",
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Place { card } => {
                s.placed[seat.index()] = Some(card);
                if s.placed.iter().all(Option::is_some) {
                    s.reveal();
                }
            }
            _ => s.play(seat, action),
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
        // ❓2: ranked by rounds won; equal scores stay tied.
        let ranks = s
            .wins
            .iter()
            .map(|&mine| s.wins.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
