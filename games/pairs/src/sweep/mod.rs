//! Sweep (James Ernest): shoot a card at five targets; the lowest shooter
//! takes the highest targets, the others sweep up everything below them.
//! Collect the most cards of each rank. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::View;

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{count_by_rank, full_deck, Rank, Supply};

pub struct Sweep;

/// [R] Five cards in each hand, five targets each round.
pub const HAND_SIZE: usize = 5;
pub const TARGETS: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a shooter, face up.
    Shoot { card: Rank },
    /// Tied shooters take turns: take this target.
    Capture { card: Rank },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// One after another, everyone plays a shooter.
    Play,
    /// A tied shooter chooses its target.
    Capture,
    Over,
}

/// Tied shooters taking turns at the targets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tie {
    /// In the order they were played.
    pub seats: Vec<Seat>,
    pub rank: Rank,
    /// The lowest shooters: they shoot at the highest original targets.
    pub lowest: bool,
    /// Whose turn it is in `seats`.
    pub next: usize,
}

/// One rank at the final count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankScore {
    pub rank: Rank,
    pub counts: Vec<u8>,
    pub winners: Vec<Seat>,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    TargetsDealt {
        round: u8,
        targets: Vec<Rank>,
        leader: Seat,
    },
    Shot {
        seat: Seat,
        card: Rank,
    },
    Captured {
        seat: Seat,
        cards: Vec<Rank>,
    },
    /// Targets nobody took.
    Discarded {
        cards: Vec<Rank>,
    },
    Scored {
        hands: Vec<Vec<Rank>>,
        ranks: Vec<RankScore>,
        scores: Vec<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub rounds_total: u8,
    pub round_no: u8,
    /// The undealt cards: nobody sees them.
    pub supply: Supply,
    pub hands: Vec<Vec<Rank>>,
    /// Per seat, the cards everyone knows are in its hand (captured, not
    /// played since).
    pub known: Vec<Vec<Rank>>,
    pub targets: Vec<Rank>,
    /// The highest rank among this round's five dealt targets.
    pub top: Option<Rank>,
    /// This round's shooters still in front of their players, in the
    /// order played (a shooter that falls into the middle leaves).
    pub shooters: Vec<(Seat, Rank)>,
    /// The lowest shooter's rank, once everyone has shot.
    pub lowest_shot: Option<Rank>,
    /// Groups of shooters still to capture, lowest first.
    pub groups: Vec<Vec<Seat>>,
    pub tie: Option<Tie>,
    pub leader: Seat,
    /// Who took the last target: leads the next round.
    pub last_taker: Option<Seat>,
    pub turn: Option<Seat>,
    pub phase: Phase,
    pub discard: Vec<Rank>,
    pub scores: Vec<u32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    fn take_from_hand(&mut self, seat: Seat, card: Rank) {
        let i = seat.index();
        if let Some(at) = self.hands[i].iter().position(|&c| c == card) {
            self.hands[i].remove(at);
        }
        if let Some(at) = self.known[i].iter().position(|&c| c == card) {
            self.known[i].remove(at);
        }
    }

    fn shooter_rank(&self, seat: Seat) -> Option<Rank> {
        self.shooters
            .iter()
            .find(|&&(s, _)| s == seat)
            .map(|&(_, r)| r)
    }

    /// [R] Deal five targets; play starts with the last taker.
    fn start_round(&mut self) {
        if self.round_no >= self.rounds_total {
            self.score();
            return;
        }
        self.round_no += 1;
        self.targets = (0..TARGETS).filter_map(|_| self.supply.draw()).collect();
        self.top = self.targets.iter().copied().max();
        self.shooters.clear();
        self.lowest_shot = None;
        self.leader = self.last_taker.unwrap_or(self.leader);
        self.events.push(Event::TargetsDealt {
            round: self.round_no,
            targets: self.targets.clone(),
            leader: self.leader,
        });
        self.phase = Phase::Play;
        self.turn = None;
        self.next_shooter(None);
    }

    /// The next seat to shoot, from the leader around; ❓2 an empty hand
    /// does not shoot. With everyone done, the captures.
    fn next_shooter(&mut self, after: Option<Seat>) {
        let start = after.map_or(0, |a| {
            (a.0 + self.players - self.leader.0) % self.players + 1
        });
        let next = (start..self.players)
            .map(|k| Seat((self.leader.0 + k) % self.players))
            .find(|s| !self.hands[s.index()].is_empty());
        match next {
            Some(seat) => self.turn = Some(seat),
            None => {
                self.turn = None;
                self.begin_captures();
            }
        }
    }

    /// [R] Shooters capture in ascending order; shooters of one rank form
    /// a group, in the order they were played.
    fn begin_captures(&mut self) {
        self.lowest_shot = self.shooters.iter().map(|&(_, r)| r).min();
        let mut ranks: Vec<Rank> = self.shooters.iter().map(|&(_, r)| r).collect();
        ranks.sort_unstable();
        ranks.dedup();
        self.groups = ranks
            .into_iter()
            .map(|rank| {
                self.shooters
                    .iter()
                    .filter(|&&(_, r)| r == rank)
                    .map(|&(s, _)| s)
                    .collect()
            })
            .collect();
        self.groups.reverse(); // popped from the end: lowest first
        self.resolve();
    }

    /// The targets a shooter of `rank` may take: [R] the lowest shoot at
    /// the highest original rank, the others at everything below them.
    fn eligible(&self, rank: Rank, lowest: bool) -> Vec<Rank> {
        self.targets
            .iter()
            .copied()
            .filter(|&t| {
                if lowest {
                    Some(t) == self.top
                } else {
                    t < rank
                }
            })
            .collect()
    }

    fn capture(&mut self, seat: Seat, cards: Vec<Rank>) {
        if cards.is_empty() {
            return;
        }
        for card in &cards {
            if let Some(at) = self.targets.iter().position(|c| c == card) {
                self.targets.remove(at);
            }
        }
        self.hands[seat.index()].extend(&cards);
        self.known[seat.index()].extend(&cards);
        self.last_taker = Some(seat);
        self.events.push(Event::Captured { seat, cards });
    }

    /// [R] After capturing, shooters fall into the middle as targets.
    fn fall(&mut self, seats: &[Seat]) {
        for &seat in seats {
            if let Some(at) = self.shooters.iter().position(|&(s, _)| s == seat) {
                let (_, rank) = self.shooters.remove(at);
                self.targets.push(rank);
            }
        }
    }

    /// Settles captures until one needs a choice, then the round's end.
    fn resolve(&mut self) {
        loop {
            if let Some(tie) = self.tie.clone() {
                let eligible = self.eligible(tie.rank, tie.lowest);
                if eligible.is_empty() {
                    self.fall(&tie.seats);
                    self.tie = None;
                    continue;
                }
                let seat = tie.seats[tie.next];
                let mut ranks = eligible.clone();
                ranks.sort_unstable();
                ranks.dedup();
                if ranks.len() > 1 {
                    self.phase = Phase::Capture;
                    self.turn = Some(seat);
                    return;
                }
                self.capture(seat, vec![ranks[0]]);
                self.advance_tie();
                continue;
            }
            let Some(group) = self.groups.pop() else {
                break;
            };
            let Some(rank) = self.shooter_rank(group[0]) else {
                continue;
            };
            let is_lowest = Some(rank) == self.lowest_shot;
            if let [seat] = group[..] {
                let cards = self.eligible(rank, is_lowest);
                self.capture(seat, cards);
                self.fall(&group);
            } else {
                // [R] Tied shooters take turns, from the first played.
                self.tie = Some(Tie {
                    seats: group,
                    rank,
                    lowest: is_lowest,
                    next: 0,
                });
            }
        }
        // [R] Whatever is left is discarded.
        let left = std::mem::take(&mut self.targets);
        if !left.is_empty() {
            self.events.push(Event::Discarded {
                cards: left.clone(),
            });
        }
        self.discard.extend(left);
        self.shooters.clear();
        self.turn = None;
        self.start_round();
    }

    fn advance_tie(&mut self) {
        if let Some(tie) = &mut self.tie {
            tie.next = (tie.next + 1) % tie.seats.len();
        }
    }

    /// [R] The most cards of a rank score its value; ties all score it.
    fn score(&mut self) {
        let counts: Vec<[u8; 10]> = self.hands.iter().map(|h| count_by_rank(h)).collect();
        let mut scores = vec![0u32; usize::from(self.players)];
        let mut ranks = Vec::new();
        for rank in Rank::all() {
            let per_seat: Vec<u8> = counts.iter().map(|c| c[rank.index()]).collect();
            let most = per_seat.iter().copied().max().unwrap_or(0);
            let winners: Vec<Seat> = if most == 0 {
                Vec::new()
            } else {
                self.seats()
                    .filter(|s| per_seat[s.index()] == most)
                    .collect()
            };
            for w in &winners {
                scores[w.index()] += u32::from(rank.value());
            }
            ranks.push(RankScore {
                rank,
                counts: per_seat,
                winners,
            });
        }
        self.events.push(Event::Scored {
            hands: self.hands.clone(),
            ranks,
            scores: scores.clone(),
        });
        self.scores = scores;
        self.turn = None;
        self.phase = Phase::Over;
    }

    fn distinct(cards: &[Rank]) -> Vec<Rank> {
        let mut cards = cards.to_vec();
        cards.sort_unstable();
        cards.dedup();
        cards
    }
}

impl Game for Sweep {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-sweep",
            name: "Sweep",
            min_players: 3,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut supply = Supply::new(full_deck(), 0, seed);
        let hands: Vec<Vec<Rank>> = (0..n)
            .map(|_| (0..HAND_SIZE).filter_map(|_| supply.draw()).collect())
            .collect();
        // [R] Deal targets until the deck runs out: 11 − players rounds.
        let rounds_total = (supply.deck_len() / TARGETS) as u8;
        // [R] A random player leads the first round.
        let leader = Seat((ChaCha8Rng::seed_from_u64(seed).next_u32() % u32::from(players)) as u8);
        let mut s = State {
            players,
            rounds_total,
            round_no: 0,
            supply,
            hands,
            known: vec![Vec::new(); n],
            targets: Vec::new(),
            top: None,
            shooters: Vec::new(),
            lowest_shot: None,
            groups: Vec::new(),
            tie: None,
            leader,
            last_taker: None,
            turn: None,
            phase: Phase::Play,
            discard: Vec::new(),
            scores: Vec::new(),
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Play | Phase::Capture => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != Some(seat) {
            return Vec::new();
        }
        match s.phase {
            Phase::Play => State::distinct(&s.hands[seat.index()])
                .into_iter()
                .map(|card| Action::Shoot { card })
                .collect(),
            Phase::Capture => match &s.tie {
                Some(tie) => State::distinct(&s.eligible(tie.rank, tie.lowest))
                    .into_iter()
                    .map(|card| Action::Capture { card })
                    .collect(),
                None => Vec::new(),
            },
            Phase::Over => Vec::new(),
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
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        s.events.clear();
        match *action {
            Action::Shoot { card } => {
                s.take_from_hand(seat, card);
                s.shooters.push((seat, card));
                s.events.push(Event::Shot { seat, card });
                s.next_shooter(Some(seat));
            }
            Action::Capture { card } => {
                s.capture(seat, vec![card]);
                s.advance_tie();
                s.phase = Phase::Play;
                s.resolve();
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
            .scores
            .iter()
            .map(|&mine| s.scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
