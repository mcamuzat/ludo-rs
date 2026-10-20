//! Horn of Plenty (James Ernest): play a card face down and declare its
//! rank; whoever answers rightly decides who takes it into a face-up hand,
//! where pairs are thrown away. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, TableView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, lowest_discarding_ties, Rank, Supply};

pub struct HornOfPlenty;

/// [R] Face-down cards dealt to each player, after the face-up one.
pub const HAND_SIZE: usize = 4;
/// [R] Game points that end the game.
pub const TARGET: u8 = 11;
/// ❓2: a game stops after this many rounds, ties at the top kept.
pub const MAX_ROUNDS: u8 = 30;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Play a hand card face down, declaring a rank (true or not).
    Play { card: Rank, declared: Rank },
    /// The declared rank is the card's.
    Agree,
    /// The declared rank is not the card's.
    Disagree,
    /// Leave the answer to the next player on the left.
    Pass,
    /// Put the revealed card into the face-up hand of `to`.
    Give { to: Seat },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// The player to move plays a card and declares its rank.
    Declare,
    /// The asked player agrees, disagrees or passes.
    Answer,
    /// The card is revealed; the chooser decides where it goes.
    Place,
    Over,
}

/// The card on the table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnTable {
    pub poser: Seat,
    pub card: Rank,
    pub declared: Rank,
    /// Answer: who must answer now.
    pub asked: Seat,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new round: the face-up card of each seat, the tie-break cards
    /// (discarded) and who plays first.
    Dealt {
        round: u8,
        upcards: Vec<Rank>,
        tie_breaks: Vec<(Seat, Rank)>,
        leader: Seat,
    },
    Declared {
        seat: Seat,
        declared: Rank,
    },
    Passed {
        seat: Seat,
    },
    /// `seat` agreed (or not); the card is revealed. `chooser` places it.
    Answered {
        seat: Seat,
        agreed: bool,
        card: Rank,
        right: bool,
        chooser: Seat,
    },
    /// Everyone passed: the card is revealed and its player places it.
    Unanswered {
        card: Rank,
    },
    /// The card went into the face-up hand of `to`; `paired`: both cards of
    /// that rank were thrown away.
    Given {
        by: Seat,
        to: Seat,
        card: Rank,
        paired: bool,
    },
    /// End of a round: each seat's card points, the game points it scored,
    /// and the totals.
    Scored {
        points: Vec<u32>,
        gained: Vec<u8>,
        scores: Vec<u8>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    /// The current round, from 1.
    pub round_no: u8,
    /// Its draw pile holds the cards not dealt: nobody sees them.
    pub supply: Supply,
    /// For each round's deal. Never in a view.
    pub seed: u64,
    /// Face-down cards; only their owner sees them.
    pub hands: Vec<Vec<Rank>>,
    /// Face-up hands.
    pub shown: Vec<Vec<Rank>>,
    /// Game points.
    pub scores: Vec<u8>,
    pub phase: Phase,
    /// Declare: who plays the next card.
    pub turn: Seat,
    pub table: Option<OnTable>,
    /// Place: who decides, and where the card may go.
    pub chooser: Option<Seat>,
    pub recipients: Vec<Seat>,
    /// Since the last card played.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

/// [R] Each card counts its rank.
pub fn card_points(cards: &[Rank]) -> u32 {
    cards.iter().map(|c| u32::from(c.value())).sum()
}

/// [R] The best hand scores as many game points as there are players, the
/// next one less, and so on; tied hands both take the higher amount, and
/// the next place is skipped (decision of the user: 5-5-3-2-1).
pub fn round_points(points: &[u32]) -> Vec<u8> {
    let n = points.len();
    points
        .iter()
        .map(|&mine| (n - points.iter().filter(|&&theirs| theirs > mine).count()) as u8)
        .collect()
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    fn next(&self, seat: Seat) -> Seat {
        Seat((seat.0 + 1) % self.players)
    }

    /// [R] A fresh deal: one face-up card each, the lowest leads (ties
    /// broken by cards then thrown away), then the face-down cards. ❓1:
    /// every round, the lowest face-up card leads again.
    fn start_round(&mut self) {
        let n = usize::from(self.players);
        self.round_no += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.round_no));
        self.shown = vec![Vec::new(); n];
        self.hands = vec![Vec::new(); n];
        let mut upcards = Vec::new();
        for i in 0..n {
            if let Some(card) = self.supply.draw() {
                self.shown[i].push(card);
                upcards.push((Seat(i as u8), card));
            }
        }
        let (leader, tie_breaks) = lowest_discarding_ties(&mut self.supply, &upcards);
        for _ in 0..HAND_SIZE {
            for hand in &mut self.hands {
                if let Some(card) = self.supply.draw() {
                    hand.push(card);
                }
            }
        }
        self.events.push(Event::Dealt {
            round: self.round_no,
            upcards: upcards.iter().map(|&(_, card)| card).collect(),
            tie_breaks,
            leader,
        });
        self.phase = Phase::Declare;
        self.turn = leader;
        self.table = None;
        self.chooser = None;
        self.recipients.clear();
    }

    /// [R] The card is revealed and `chooser` places it among `recipients`.
    fn reveal(&mut self, chooser: Seat, recipients: Vec<Seat>) {
        self.phase = Phase::Place;
        self.chooser = Some(chooser);
        self.recipients = recipients;
    }

    /// [R] Into the face-up hand of `to`; a pair is thrown away at once.
    fn give(&mut self, to: Seat) {
        let Some(table) = self.table.take() else {
            return;
        };
        let hand = &mut self.shown[to.index()];
        let paired = hand.contains(&table.card);
        if paired {
            hand.retain(|&c| c != table.card);
            self.supply.discard([table.card, table.card]);
        } else {
            hand.push(table.card);
        }
        self.events.push(Event::Given {
            by: self.chooser.unwrap_or(table.poser),
            to,
            card: table.card,
            paired,
        });
        self.chooser = None;
        self.recipients.clear();
        if self.hands.iter().all(Vec::is_empty) {
            self.end_round();
            return;
        }
        self.phase = Phase::Declare;
        let mut next = self.next(table.poser);
        while self.hands[next.index()].is_empty() {
            next = self.next(next);
        }
        self.turn = next;
    }

    /// [R] The round scores by card points; the game ends once someone has
    /// 11 game points, unless the top is tied (another round is played).
    fn end_round(&mut self) {
        let points: Vec<u32> = self.shown.iter().map(|h| card_points(h)).collect();
        let gained = round_points(&points);
        for (score, gain) in self.scores.iter_mut().zip(&gained) {
            *score = score.saturating_add(*gain);
        }
        self.events.push(Event::Scored {
            points,
            gained,
            scores: self.scores.clone(),
        });
        let top = self.scores.iter().copied().max().unwrap_or(0);
        let leaders = self.scores.iter().filter(|&&s| s == top).count();
        if (top >= TARGET && leaders == 1) || self.round_no >= MAX_ROUNDS {
            self.phase = Phase::Over;
            self.table = None;
        } else {
            self.start_round();
        }
    }

    /// Who must decide now.
    fn to_move(&self) -> Option<Seat> {
        match self.phase {
            Phase::Declare => Some(self.turn),
            Phase::Answer => self.table.as_ref().map(|t| t.asked),
            Phase::Place => self.chooser,
            Phase::Over => None,
        }
    }
}

impl Game for HornOfPlenty {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-horn-of-plenty",
            name: "Horn of Plenty",
            min_players: 2,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            round_no: 0,
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: vec![Vec::new(); n],
            shown: vec![Vec::new(); n],
            scores: vec![0; n],
            phase: Phase::Declare,
            turn: Seat(0),
            table: None,
            chooser: None,
            recipients: Vec::new(),
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        s.to_move().map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.to_move() != Some(seat) {
            return Vec::new();
        }
        match s.phase {
            Phase::Declare => {
                let mut cards = s.hands[seat.index()].clone();
                cards.sort_unstable();
                cards.dedup();
                cards
                    .into_iter()
                    .flat_map(|card| {
                        Rank::all().map(move |declared| Action::Play { card, declared })
                    })
                    .collect()
            }
            Phase::Answer => vec![Action::Agree, Action::Disagree, Action::Pass],
            Phase::Place => s.recipients.iter().map(|&to| Action::Give { to }).collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.to_move() != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Play { card, declared } => {
                s.events.clear();
                let hand = &mut s.hands[seat.index()];
                if let Some(at) = hand.iter().position(|&c| c == card) {
                    hand.remove(at);
                }
                s.table = Some(OnTable {
                    poser: seat,
                    card,
                    declared,
                    asked: s.next(seat),
                });
                s.phase = Phase::Answer;
                s.events.push(Event::Declared { seat, declared });
            }
            Action::Pass => {
                s.events.push(Event::Passed { seat });
                let next = s.next(seat);
                let Some(table) = s.table.as_mut() else {
                    return Ok(());
                };
                if next == table.poser {
                    // [R] Passed all the way round: its player places it
                    // anywhere (❓3: in any seat's face-up hand).
                    let (poser, card) = (table.poser, table.card);
                    s.events.push(Event::Unanswered { card });
                    let everyone = s.seats().collect();
                    s.reveal(poser, everyone);
                } else {
                    table.asked = next;
                }
            }
            Action::Agree | Action::Disagree => {
                let Some(table) = s.table.as_ref() else {
                    return Ok(());
                };
                let agreed = *action == Action::Agree;
                let right = (table.card == table.declared) == agreed;
                // [R] Right: the answerer decides; wrong: the card's player.
                let chooser = if right { seat } else { table.poser };
                let mut recipients = vec![seat, table.poser];
                recipients.sort_unstable();
                s.events.push(Event::Answered {
                    seat,
                    agreed,
                    card: table.card,
                    right,
                    chooser,
                });
                s.reveal(chooser, recipients);
            }
            Action::Give { to } => s.give(to),
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
