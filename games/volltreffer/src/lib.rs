//! Volltreffer (Günter Burkhardt), sources and rules in RULES.md: hire eight
//! adventurers with your points, show groups, play eight tricks, and end a
//! round in the Sanctuary, as close to 66 as you can.

mod determinize;
mod heuristics;
mod view;

pub use view::{HandInfo, View};

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// [R] Cards per hand, so hires and tricks per round.
pub const HAND: usize = 8;
/// [P] The last space of the track.
pub const TRACK_END: u32 = 80;
/// [R][P] Beyond 80, counting goes on from 41 ("Oops! TELEPORTER").
pub const TELEPORT_TO: u32 = 41;
/// [R] The heart of the Sanctuary.
pub const GOAL: u32 = 66;
/// [R] What a trick, the "5 tribes" and the pre-emptive action score.
pub const FIVE: u32 = 5;
/// [R][P] The cost of each place of the display, cheapest first.
pub const COSTS: [u32; 4] = [0, 1, 2, 4];

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// [R] Variant: the Sanctuary shrinks to 63–69.
    pub narrow_sanctuary: bool,
}

impl Config {
    pub fn sanctuary(&self) -> std::ops::RangeInclusive<u32> {
        if self.narrow_sanctuary {
            63..=69
        } else {
            61..=70
        }
    }
}

/// [R] The five tribes. The rules do not name them: the site uses colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tribe {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
}

impl Tribe {
    pub const ALL: [Tribe; 5] = [
        Tribe::Red,
        Tribe::Green,
        Tribe::Blue,
        Tribe::Yellow,
        Tribe::Purple,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

/// [R] An adventurer: a value from 1 to 10 in one of the five tribes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub tribe: Tribe,
    pub value: u8,
}

impl Card {
    pub fn new(tribe: Tribe, value: u8) -> Self {
        Self { tribe, value }
    }
}

/// [R] The 50 cards, 1 to 10 in each tribe.
pub fn pack() -> Vec<Card> {
    Tribe::ALL
        .iter()
        .flat_map(|&tribe| (1..=10).map(move |value| Card { tribe, value }))
        .collect()
}

/// [R][P] A gain past 80 goes on from 41.
pub fn gain(score: u32, points: u32) -> u32 {
    let mut s = score + points;
    while s > TRACK_END {
        s -= TRACK_END - TELEPORT_TO + 1;
    }
    s
}

/// [R] A party of 3, 4 or 5 cards of the same value.
pub fn party_points(size: usize) -> u32 {
    match size {
        3 => 5,
        4 => 10,
        5 => 25,
        _ => 0,
    }
}

/// The values `hand` holds at least three of, in increasing order.
pub fn parties(hand: &[Card]) -> Vec<u8> {
    (1..=10)
        .filter(|&v| hand.iter().filter(|c| c.value == v).count() >= 3)
        .collect()
}

/// [R] One card of each of the five tribes.
pub fn has_five_tribes(hand: &[Card]) -> bool {
    Tribe::ALL
        .iter()
        .all(|&t| hand.iter().any(|c| c.tribe == t))
}

/// The points a publication scores with `hand`.
pub fn publication_points(hand: &[Card], groups: &[u8], tribes: bool) -> u32 {
    let parties: u32 = groups
        .iter()
        .map(|&v| party_points(hand.iter().filter(|c| c.value == v).count()))
        .sum();
    parties + if tribes { FIVE } else { 0 }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// [R] Phase 1: hiring adventurers.
    Hire,
    /// [R] Phase 2: forming a party.
    Publish,
    /// [R] Phase 3: tricks.
    Tricks,
    Over,
}

/// `groups` lists values in increasing order, so that a move has a single
/// spelling.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Hire { card: Card },
    Publish { groups: Vec<u8>, tribes: bool },
    Play { card: Card },
}

/// A card of the display and when it arrived (later ones go left on a tie).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shown {
    pub card: Card,
    pub arrival: u32,
}

/// A hire, seen by everyone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HireEvent {
    pub seat: Seat,
    pub card: Card,
    pub cost: u32,
}

/// What a seat showed in phase 2.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub groups: Vec<u8>,
    pub tribes: bool,
    /// The cards shown, visible to all until the tricks start.
    pub cards: Vec<Card>,
    pub points: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakenTrick {
    pub plays: Vec<(Seat, Card)>,
    pub winner: Seat,
}

/// One seat's round, for the score sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundLine {
    pub spent: u32,
    pub melds: u32,
    pub preemptive: bool,
    pub tricks: u8,
    /// The score at the end of the round.
    pub score: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub players: u8,
    pub seed: u64,
    /// From 0.
    pub round: u32,
    pub scores: Vec<u32>,
    /// [R] Who starts the hiring, passed to the left each round.
    pub bargaining: Seat,
    /// [R] The "Blessing": its tribe is the trump of the round.
    pub blessing: Card,
    /// Face down, the last card on top.
    pub deck: Vec<Card>,
    /// Cheapest place first.
    pub display: Vec<Shown>,
    pub arrivals: u32,
    /// Sorted.
    pub hands: Vec<Vec<Card>>,
    pub phase: Phase,
    pub current: Seat,
    /// This round's hires, in order.
    pub hires: Vec<HireEvent>,
    pub publications: Vec<Option<Publication>>,
    pub preemptive: Option<Seat>,
    pub trick: Vec<(Seat, Card)>,
    pub last_trick: Option<TakenTrick>,
    /// Tribes each seat failed to follow this round: it holds none.
    pub voids: Vec<[bool; 5]>,
    pub lines: Vec<RoundLine>,
    pub last_round: Option<Vec<RoundLine>>,
    /// Moves applied so far (the AI's simulation horizon).
    pub moves: u32,
}

impl State {
    pub fn n(&self) -> usize {
        usize::from(self.players)
    }

    pub fn trump(&self) -> Tribe {
        self.blessing.tribe
    }

    pub fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// [R] Trumps on the dearest places, the highest dearest; then the other
    /// cards by value; on a tie the later card goes left.
    fn sort_display(&mut self) {
        let trump = self.trump();
        self.display.sort_by_key(|s| {
            (
                s.card.tribe == trump,
                s.card.value,
                std::cmp::Reverse(s.arrival),
            )
        });
    }

    fn reveal(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.display.push(Shown {
                card,
                arrival: self.arrivals,
            });
            self.arrivals += 1;
            self.sort_display();
        }
    }

    /// The cost of a card of the display.
    pub fn cost_of(&self, card: Card) -> Option<u32> {
        self.display
            .iter()
            .position(|s| s.card == card)
            .map(|i| COSTS[i])
    }

    /// [R] A new round: shuffle all, turn the Blessing, lay the display.
    fn deal_round(&mut self) {
        let stream = self.seed ^ u64::from(self.round + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut deck = pack();
        deck.shuffle(&mut ChaCha8Rng::seed_from_u64(stream));
        self.blessing = deck.pop().expect("50 cards");
        self.deck = deck;
        self.display.clear();
        self.arrivals = 0;
        for _ in 0..COSTS.len() {
            self.reveal();
        }
        for hand in &mut self.hands {
            hand.clear();
        }
        self.phase = Phase::Hire;
        self.current = self.bargaining;
        self.hires.clear();
        self.publications = vec![None; self.n()];
        self.preemptive = None;
        self.trick.clear();
        self.voids = vec![[false; 5]; self.n()];
        self.lines = vec![RoundLine::default(); self.n()];
    }

    fn score(&mut self, seat: Seat, points: u32) {
        let s = &mut self.scores[seat.index()];
        *s = gain(*s, points);
    }

    /// The cards `seat` may play to the current trick.
    pub fn playable(&self, seat: Seat) -> Vec<Card> {
        let hand = &self.hands[seat.index()];
        // [R] Follow the tribe led when you can.
        if let Some(&(_, lead)) = self.trick.first() {
            let following: Vec<Card> = hand
                .iter()
                .copied()
                .filter(|c| c.tribe == lead.tribe)
                .collect();
            if !following.is_empty() {
                return following;
            }
        }
        hand.clone()
    }

    /// [R] The highest trump wins, else the highest card of the tribe led.
    pub fn trick_winner(&self) -> Seat {
        let trump = self.trump();
        let lead = self.trick[0].1.tribe;
        self.trick
            .iter()
            .max_by_key(|(_, c)| (c.tribe == trump, c.tribe == lead, c.value))
            .expect("a full trick")
            .0
    }

    /// [R] Lowest sum of values; ties go to the most 1s, then 2s, … Equal
    /// hands: the first from the bargaining card's holder (site choice).
    fn preemptive_seat(&self) -> Seat {
        (0..self.players)
            .map(|k| Seat((self.bargaining.0 + k) % self.players))
            .min_by_key(|&seat| {
                let hand = &self.hands[seat.index()];
                let sum: u32 = hand.iter().map(|c| u32::from(c.value)).sum();
                let counts: Vec<std::cmp::Reverse<usize>> = (1..=10)
                    .map(|v| std::cmp::Reverse(hand.iter().filter(|c| c.value == v).count()))
                    .collect();
                (sum, counts)
            })
            .expect("at least three seats")
    }

    /// Sort key for the final standing: in the Sanctuary first, then
    /// closest to 66, then highest score.
    fn standing_key(&self, seat: usize) -> (bool, u32, std::cmp::Reverse<u32>) {
        let score = self.scores[seat];
        (
            !self.config.sanctuary().contains(&score),
            score.abs_diff(GOAL),
            std::cmp::Reverse(score),
        )
    }

    /// Ranks by `standing_key`; equal keys tie.
    pub fn ranks(&self) -> Vec<u8> {
        (0..self.n())
            .map(|i| {
                let mine = self.standing_key(i);
                (0..self.n())
                    .filter(|&j| self.standing_key(j) < mine)
                    .count() as u8
            })
            .collect()
    }

    /// [R] After the eighth trick: someone in the Sanctuary wins, else a
    /// new round starts with the bargaining card passed left.
    fn end_round(&mut self) {
        for (line, &score) in self.lines.iter_mut().zip(&self.scores) {
            line.score = score;
        }
        self.last_round = Some(self.lines.clone());
        let sanctuary = self.config.sanctuary();
        if self.scores.iter().any(|s| sanctuary.contains(s)) {
            self.phase = Phase::Over;
            return;
        }
        self.round += 1;
        self.bargaining = self.next(self.bargaining);
        self.deal_round();
    }

    /// [R] Phase 2 order: from the bargaining card's holder (❓, see RULES.md).
    fn after_publication(&mut self) {
        let next = self.next(self.current);
        if self.publications[next.index()].is_none() {
            self.current = next;
            return;
        }
        // [R] The lowest sum takes the pre-emptive action and leads.
        let seat = self.preemptive_seat();
        self.preemptive = Some(seat);
        self.score(seat, FIVE);
        self.lines[seat.index()].preemptive = true;
        self.phase = Phase::Tricks;
        self.current = seat;
    }
}

pub struct Volltreffer;

impl Volltreffer {
    /// Every subset of the combinations `hand` may show.
    pub fn publications(hand: &[Card]) -> Vec<Action> {
        let groups = parties(hand);
        let tribes_options: &[bool] = if has_five_tribes(hand) {
            &[false, true]
        } else {
            &[false]
        };
        let mut actions = Vec::new();
        for mask in 0..(1u32 << groups.len()) {
            let chosen: Vec<u8> = groups
                .iter()
                .enumerate()
                .filter(|&(i, _)| mask & (1 << i) != 0)
                .map(|(_, &v)| v)
                .collect();
            for &tribes in tribes_options {
                actions.push(Action::Publish {
                    groups: chosen.clone(),
                    tribes,
                });
            }
        }
        actions
    }
}

impl Game for Volltreffer {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "volltreffer",
            name: "Volltreffer",
            min_players: 3,
            max_players: 5,
            hidden_info: true,
            clock: true,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        // [R] 25 points at three or four players, 30 at five.
        let start = if players == 5 { 30 } else { 25 };
        let bargaining = Seat((seed % u64::from(players)) as u8);
        let mut s = State {
            config: config.clone(),
            players,
            seed,
            round: 0,
            scores: vec![start; n],
            bargaining,
            blessing: Card::new(Tribe::Red, 1),
            deck: Vec::new(),
            display: Vec::new(),
            arrivals: 0,
            hands: vec![Vec::new(); n],
            phase: Phase::Hire,
            current: bargaining,
            hires: Vec::new(),
            publications: vec![None; n],
            preemptive: None,
            trick: Vec::new(),
            last_trick: None,
            voids: vec![[false; 5]; n],
            lines: vec![RoundLine::default(); n],
            last_round: None,
            moves: 0,
        };
        s.deal_round();
        s
    }

    fn pending(s: &State) -> Pending {
        if s.phase == Phase::Over {
            Pending::none()
        } else {
            Pending::one(s.current)
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase == Phase::Over || seat != s.current {
            return Vec::new();
        }
        match s.phase {
            // [U] Only a card the seat can pay for; the one at 0 always is.
            Phase::Hire => s
                .display
                .iter()
                .zip(COSTS)
                .filter(|&(_, cost)| cost <= s.scores[seat.index()])
                .map(|(shown, _)| Action::Hire { card: shown.card })
                .collect(),
            Phase::Publish => Self::publications(&s.hands[seat.index()]),
            Phase::Tricks => s
                .playable(seat)
                .into_iter()
                .map(|card| Action::Play { card })
                .collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est finie."));
        }
        if seat != s.current {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis maintenant."));
        }
        s.moves += 1;
        match action {
            Action::Hire { card } => {
                let cost = s.cost_of(*card).expect("legal: on the display");
                s.display.retain(|shown| shown.card != *card);
                s.scores[seat.index()] -= cost;
                s.lines[seat.index()].spent += cost;
                let hand = &mut s.hands[seat.index()];
                hand.push(*card);
                hand.sort();
                s.hires.push(HireEvent {
                    seat,
                    card: *card,
                    cost,
                });
                if s.hires.len() == HAND * s.n() {
                    s.phase = Phase::Publish;
                    s.current = s.bargaining;
                } else {
                    s.reveal();
                    s.current = s.next(seat);
                }
            }
            Action::Publish { groups, tribes } => {
                let hand = &s.hands[seat.index()];
                let points = publication_points(hand, groups, *tribes);
                let mut cards: Vec<Card> = hand
                    .iter()
                    .copied()
                    .filter(|c| groups.contains(&c.value))
                    .collect();
                if *tribes {
                    // [R] One card of each tribe: the lowest not yet shown.
                    for t in Tribe::ALL {
                        if !cards.iter().any(|c| c.tribe == t) {
                            let pick = *hand
                                .iter()
                                .find(|c| c.tribe == t)
                                .expect("legal: five tribes");
                            cards.push(pick);
                        }
                    }
                }
                cards.sort();
                s.publications[seat.index()] = Some(Publication {
                    groups: groups.clone(),
                    tribes: *tribes,
                    cards,
                    points,
                });
                s.lines[seat.index()].melds = points;
                s.score(seat, points);
                s.after_publication();
            }
            Action::Play { card } => {
                if let Some(&(_, lead)) = s.trick.first() {
                    if card.tribe != lead.tribe {
                        s.voids[seat.index()][lead.tribe.index()] = true;
                    }
                }
                s.hands[seat.index()].retain(|c| c != card);
                s.trick.push((seat, *card));
                if s.trick.len() < s.n() {
                    s.current = s.next(seat);
                } else {
                    let winner = s.trick_winner();
                    // [R] The winner scores 5 at once and leads the next.
                    s.score(winner, FIVE);
                    s.lines[winner.index()].tricks += 1;
                    s.last_trick = Some(TakenTrick {
                        plays: std::mem::take(&mut s.trick),
                        winner,
                    });
                    s.current = winner;
                    if s.hands.iter().all(Vec::is_empty) {
                        s.end_round();
                    }
                }
            }
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        (s.phase == Phase::Over).then(|| Outcome::Ranked(s.ranks()))
    }

    /// The free card, showing nothing, the first card that may be played.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        match s.phase {
            Phase::Publish => Some(Action::Publish {
                groups: Vec::new(),
                tribes: false,
            }),
            _ => legal.into_iter().next(),
        }
        .filter(|_| seat == s.current && s.phase != Phase::Over)
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "narrow_sanctuary",
            kind: OptionKind::Toggle { default: false },
        }]
    }
}
