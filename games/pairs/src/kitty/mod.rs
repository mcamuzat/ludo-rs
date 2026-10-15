//! Kitty (James Ernest): pass a card left, bid in secret, and the lowest
//! unique bid takes the target turned from the kitty. Rules and their
//! sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::View;

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{count_by_rank, full_deck, Rank};

pub struct Kitty;

/// [R] The kitty is always seven cards.
pub const KITTY_SIZE: usize = 7;

/// [R] Hard at four players (the basic rules), soft otherwise (❓1).
pub fn is_soft(players: u8) -> bool {
    players != 4
}

/// [R] Five players play without three cards, so the hands come out even.
pub fn removed_count(players: u8) -> usize {
    if players == 5 {
        3
    } else {
        0
    }
}

pub fn hand_size(players: u8) -> usize {
    (crate::deck::DECK_SIZE - KITTY_SIZE - removed_count(players)) / usize::from(players)
}

/// [R] Hard: seven targets of one card. Soft: 1-2-2-2.
pub fn target_size(soft: bool, round: u8) -> usize {
    if soft && round > 1 {
        2
    } else {
        1
    }
}

/// [R] Seven rounds hard, four soft: one per target.
pub fn rounds_total(soft: bool) -> u8 {
    if soft {
        4
    } else {
        7
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// The card passed to the player on the left, chosen in secret.
    Pass { card: Rank },
    /// The bid card, chosen in secret.
    Bid { card: Rank },
    /// Take one of the target cards.
    Take { card: Rank },
    /// After winning with the 1: hand a target card to another player.
    Give { card: Rank, to: Seat },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone chooses a card to pass, at the same time.
    Pass,
    /// Everyone chooses a bid, at the same time.
    Bid,
    /// The winning bids claim the target, lowest first.
    Claim,
    Over,
}

/// One rank at the final count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankScore {
    pub rank: Rank,
    /// Cards of that rank in each hand.
    pub counts: Vec<u8>,
    /// Who scores it: the most cards, ties all scoring.
    pub winners: Vec<Seat>,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    TargetRevealed {
        round: u8,
        cards: Vec<Rank>,
    },
    /// Every bid; `winners`: the lowest unique bids, lowest first, one
    /// per target card.
    BidsRevealed {
        bids: Vec<Rank>,
        winners: Vec<Seat>,
    },
    Took {
        seat: Seat,
        card: Rank,
    },
    Gave {
        seat: Seat,
        to: Seat,
        card: Rank,
    },
    /// Target cards nobody claimed.
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
    pub soft: bool,
    /// The current round, from 1.
    pub round_no: u8,
    /// Face down; the next target is the last element.
    pub kitty: Vec<Rank>,
    /// Out of play unseen (five players).
    pub removed: Vec<Rank>,
    pub hands: Vec<Vec<Rank>>,
    /// The target cards still to claim.
    pub target: Vec<Rank>,
    /// Pass or Bid: the card each seat has chosen, still in its hand.
    pub placed: Vec<Option<Rank>>,
    /// This round: the card each seat passed (known to it and to the
    /// player on its left only).
    pub passed: Vec<Option<Rank>>,
    /// This round: the card each seat received.
    pub received: Vec<Option<Rank>>,
    /// This round's revealed bids.
    pub bids: Vec<Rank>,
    /// Winning bids still to claim, lowest first.
    pub claims: Vec<Seat>,
    /// Bids and unclaimed targets, face up.
    pub discard: Vec<Rank>,
    pub phase: Phase,
    /// The final scores.
    pub scores: Vec<u32>,
    /// Since the last reveal or claim.
    pub events: Vec<Event>,
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    pub fn left(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    fn take_from_hand(&mut self, seat: Seat, card: Rank) {
        let hand = &mut self.hands[seat.index()];
        if let Some(at) = hand.iter().position(|&c| c == card) {
            hand.remove(at);
        }
    }

    fn distinct(cards: &[Rank]) -> Vec<Rank> {
        let mut cards = cards.to_vec();
        cards.sort_unstable();
        cards.dedup();
        cards
    }

    /// [R] Turn the next target over; with the kitty empty, score.
    fn start_round(&mut self) {
        let size = target_size(self.soft, self.round_no + 1);
        if self.kitty.is_empty() {
            self.score();
            return;
        }
        self.round_no += 1;
        let at = self.kitty.len().saturating_sub(size);
        self.target = self.kitty.split_off(at);
        self.events.push(Event::TargetRevealed {
            round: self.round_no,
            cards: self.target.clone(),
        });
        let n = usize::from(self.players);
        self.placed = vec![None; n];
        self.passed = vec![None; n];
        self.received = vec![None; n];
        self.bids.clear();
        self.phase = Phase::Pass;
    }

    /// [R] Every chosen card goes to the player on the left, together.
    fn resolve_passes(&mut self) {
        let chosen: Vec<Option<Rank>> = std::mem::take(&mut self.placed);
        for (i, card) in chosen.iter().enumerate() {
            if let Some(card) = *card {
                self.take_from_hand(Seat(i as u8), card);
            }
        }
        for (i, card) in chosen.iter().enumerate() {
            if let Some(card) = *card {
                let to = self.left(Seat(i as u8));
                self.hands[to.index()].push(card);
                self.received[to.index()] = Some(card);
            }
        }
        self.passed = chosen;
        self.placed = vec![None; usize::from(self.players)];
        self.phase = Phase::Bid;
    }

    /// [R] The bids are revealed. The lowest unique bid takes the target;
    /// tied bids cannot win. Soft: the second lowest unique bid takes the
    /// other card. The 1 is unbeatable, but gives its card away.
    fn resolve_bids(&mut self) {
        self.events.clear();
        let chosen: Vec<Option<Rank>> = std::mem::take(&mut self.placed);
        self.bids = chosen.iter().flatten().copied().collect();
        for (i, card) in chosen.iter().enumerate() {
            if let Some(card) = *card {
                self.take_from_hand(Seat(i as u8), card);
            }
        }
        let counts = count_by_rank(&self.bids);
        let mut unique: Vec<Seat> = self
            .seats()
            .filter(|s| counts[self.bids[s.index()].index()] == 1)
            .collect();
        unique.sort_by_key(|s| self.bids[s.index()]);
        unique.truncate(self.target.len());
        self.events.push(Event::BidsRevealed {
            bids: self.bids.clone(),
            winners: unique.clone(),
        });
        self.claims = unique;
        self.placed = vec![None; usize::from(self.players)];
        self.phase = Phase::Claim;
        self.advance_claims();
    }

    fn bid_one(&self, seat: Seat) -> bool {
        self.bids[seat.index()].value() == 1
    }

    /// Settles the claims that leave no choice; stops at the first that does.
    fn advance_claims(&mut self) {
        while let Some(&seat) = self.claims.first() {
            if self.target.is_empty() {
                self.claims.clear();
                break;
            }
            let choices = Self::distinct(&self.target);
            if self.bid_one(seat) || choices.len() > 1 {
                return;
            }
            self.claim(seat, choices[0], None);
        }
        self.end_round();
    }

    fn claim(&mut self, seat: Seat, card: Rank, to: Option<Seat>) {
        if let Some(at) = self.target.iter().position(|&c| c == card) {
            self.target.remove(at);
        }
        let receiver = to.unwrap_or(seat);
        self.hands[receiver.index()].push(card);
        self.events.push(match to {
            Some(to) => Event::Gave { seat, to, card },
            None => Event::Took { seat, card },
        });
        self.claims.retain(|&s| s != seat);
    }

    /// [R] Unclaimed targets and the bids are discarded.
    fn end_round(&mut self) {
        let unclaimed = std::mem::take(&mut self.target);
        if !unclaimed.is_empty() {
            self.events.push(Event::Discarded {
                cards: unclaimed.clone(),
            });
        }
        self.discard.extend(unclaimed);
        let bids = std::mem::take(&mut self.bids);
        self.discard.extend(bids);
        self.start_round();
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
        self.target.clear();
        self.claims.clear();
        self.phase = Phase::Over;
    }

    fn claim_actions(&self, seat: Seat) -> Vec<Action> {
        if self.claims.first() != Some(&seat) {
            return Vec::new();
        }
        let cards = Self::distinct(&self.target);
        if self.bid_one(seat) {
            cards
                .into_iter()
                .flat_map(|card| {
                    self.seats()
                        .filter(move |&to| to != seat)
                        .map(move |to| Action::Give { card, to })
                })
                .collect()
        } else {
            cards
                .into_iter()
                .map(|card| Action::Take { card })
                .collect()
        }
    }
}

impl Game for Kitty {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-kitty",
            name: "Kitty",
            min_players: 3,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        let mut deck = full_deck();
        deck.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
        let kitty = deck.split_off(deck.len() - KITTY_SIZE);
        let removed = deck.split_off(deck.len() - removed_count(players));
        let size = hand_size(players);
        let hands = (0..usize::from(players))
            .map(|i| deck[i * size..(i + 1) * size].to_vec())
            .collect();
        let n = usize::from(players);
        let mut s = State {
            players,
            soft: is_soft(players),
            round_no: 0,
            kitty,
            removed,
            hands,
            target: Vec::new(),
            placed: vec![None; n],
            passed: vec![None; n],
            received: vec![None; n],
            bids: Vec::new(),
            claims: Vec::new(),
            discard: Vec::new(),
            phase: Phase::Pass,
            scores: Vec::new(),
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Pass | Phase::Bid => Pending(
                s.seats()
                    .filter(|x| s.placed[x.index()].is_none())
                    .collect(),
            ),
            Phase::Claim => s
                .claims
                .first()
                .map_or_else(Pending::none, |&seat| Pending::one(seat)),
            Phase::Over => Pending::none(),
        }
    }

    /// While cards are chosen nobody learns who has already chosen.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        match (s.phase, viewer) {
            (Phase::Pass | Phase::Bid, Viewer::Seat(seat)) => match s.placed.get(seat.index()) {
                Some(None) => Pending::one(seat),
                _ => Pending::none(),
            },
            (Phase::Pass | Phase::Bid, Viewer::Spectator) => Pending::none(),
            _ => Self::pending(s),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat.index() >= usize::from(s.players) {
            return Vec::new();
        }
        let hand = || State::distinct(&s.hands[seat.index()]);
        match s.phase {
            Phase::Pass if s.placed[seat.index()].is_none() => hand()
                .into_iter()
                .map(|card| Action::Pass { card })
                .collect(),
            Phase::Bid if s.placed[seat.index()].is_none() => hand()
                .into_iter()
                .map(|card| Action::Bid { card })
                .collect(),
            Phase::Claim => s.claim_actions(seat),
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
                Phase::Pass | Phase::Bid => "Tu as déjà choisi ta carte.",
                _ => "Ce n'est pas ton tour.",
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Pass { card } | Action::Bid { card } => {
                s.placed[seat.index()] = Some(card);
                if s.placed.iter().all(Option::is_some) {
                    if s.phase == Phase::Pass {
                        s.resolve_passes();
                    } else {
                        s.resolve_bids();
                    }
                }
            }
            Action::Take { card } => {
                s.events.clear();
                s.claim(seat, card, None);
                s.advance_claims();
            }
            Action::Give { card, to } => {
                s.events.clear();
                s.claim(seat, card, Some(to));
                s.advance_claims();
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
