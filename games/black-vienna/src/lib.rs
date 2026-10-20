//! Black Vienna (Gilbert Obermair), sources and rules in RULES.md: ask the
//! others about three suspects at a time, deduce who holds whom, and name
//! the three criminals set aside before anyone else.

mod determinize;
pub mod knowledge;
pub mod material;
pub mod testing;
mod view;

pub use material::{deal, hand_sizes, suspects, CardId, Deal, Suspect, INVESTIGATIONS};
pub use view::{PileView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

pub struct BlackVienna;

/// [R] Tokens in the reserve at the start.
pub const TOKENS: u8 = 40;
/// [R] Investigation cards played before a special investigation.
pub const SPECIAL_AFTER: usize = 6;
/// [R] With this many tokens or fewer left, the game stops (❓5: checked
/// right after each answer).
pub const STOP_AT: u8 = 3;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Lay the top card of `pile` (and, in a special investigation, an
    /// earlier card left without tokens, `extra`) before `target`. ❓7:
    /// `pile` is `None` only when the piles are empty and `extra` is set.
    Ask {
        pile: Option<u8>,
        target: Seat,
        extra: Option<CardId>,
    },
    /// "Black Vienna!": name the trio (sorted, three distinct suspects).
    Accuse { trio: [Suspect; 3] },
    /// After a right accusation: one's own guess at the trio.
    Guess { trio: [Suspect; 3] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// The player to move accuses or asks.
    Turn,
    /// After a right accusation, the others name a trio, all at once.
    Guess,
    Over,
}

/// One investigation card answered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    pub by: Seat,
    pub target: Seat,
    pub card: CardId,
    /// [R] How many of its suspects the target holds.
    pub count: u8,
    /// Tokens actually taken from the reserve (❓6: fewer when it runs out).
    pub placed: u8,
    /// An earlier card laid again in a special investigation.
    pub special: bool,
}

/// Why a game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    /// Someone named the trio.
    Solved,
    /// [R] Three tokens or fewer left.
    Reserve,
    /// ❓7: nothing left to ask, nobody left to accuse.
    Stuck,
    /// [R] The last player accused wrongly.
    Failed,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Asked {
        by: Seat,
        target: Seat,
        answers: Vec<Answer>,
    },
    /// The trio named stays secret (❓3).
    Accused { seat: Seat, right: bool },
    /// The game is over: the trio, everyone's guess, the scores.
    Revealed {
        end: End,
        trio: [Suspect; 3],
        accuser: Option<Seat>,
        guesses: Vec<Option<[Suspect; 3]>>,
        scores: Vec<u16>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub dealer: Seat,
    /// Never in a view.
    pub seed: u64,
    /// Sorted.
    pub hands: Vec<Vec<Suspect>>,
    /// Black Vienna, sorted.
    pub trio: [Suspect; 3],
    /// The top of each pile is its last card.
    pub piles: [Vec<CardId>; 3],
    pub answers: Vec<Answer>,
    pub reserve: u8,
    /// Who has been asked at least once.
    pub asked: Vec<bool>,
    /// Who has accused wrongly.
    pub wrong: Vec<bool>,
    pub phase: Phase,
    pub turn: Seat,
    /// [R] The only player who has not accused wrongly, once all others
    /// have: one final question, then an accusation.
    pub last: Option<Seat>,
    pub final_asked: bool,
    /// Guess: each seat's trio (the accuser's included).
    pub guesses: Vec<Option<[Suspect; 3]>>,
    pub accuser: Option<Seat>,
    pub scores: Vec<u16>,
    /// Since the last action.
    pub events: Vec<Event>,
}

/// Every trio one may name: three distinct suspects, sorted.
pub fn all_trios() -> impl Iterator<Item = [Suspect; 3]> {
    let all: Vec<Suspect> = Suspect::all().collect();
    let n = all.len();
    (0..n).flat_map(move |i| {
        let all = all.clone();
        (i + 1..n).flat_map(move |j| {
            let all = all.clone();
            (j + 1..n).map(move |k| [all[i], all[j], all[k]])
        })
    })
}

/// Investigation cards drawn from the piles so far.
pub fn cards_played(s: &State) -> usize {
    s.answers.iter().filter(|a| !a.special).count()
}

/// [R] A special investigation needs everyone asked once and six cards
/// played.
pub fn special_allowed(s: &State) -> bool {
    s.asked.iter().all(|&a| a) && cards_played(s) >= SPECIAL_AFTER
}

/// [R] Cards played whose latest answer took no token, in play order.
pub fn reusable(s: &State) -> Vec<CardId> {
    let mut out: Vec<CardId> = Vec::new();
    for answer in &s.answers {
        out.retain(|&c| c != answer.card);
        if answer.count == 0 {
            out.push(answer.card);
        }
    }
    out
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    fn count(&self, target: Seat, card: CardId) -> u8 {
        let hand = &self.hands[target.index()];
        suspects(card).iter().filter(|s| hand.contains(s)).count() as u8
    }

    fn asks(&self, seat: Seat) -> Vec<Action> {
        if self.last == Some(seat) && self.final_asked {
            return Vec::new();
        }
        let extras: Vec<Option<CardId>> = if special_allowed(self) {
            std::iter::once(None)
                .chain(reusable(self).into_iter().map(Some))
                .collect()
        } else {
            vec![None]
        };
        let piles: Vec<u8> = (0..3u8)
            .filter(|&p| !self.piles[usize::from(p)].is_empty())
            .collect();
        let mut out = Vec::new();
        for target in self.seats().filter(|&t| t != seat) {
            if piles.is_empty() {
                // ❓7: only an earlier card, if any.
                out.extend(extras.iter().flatten().map(|&extra| Action::Ask {
                    pile: None,
                    target,
                    extra: Some(extra),
                }));
            }
            for &pile in &piles {
                out.extend(extras.iter().map(|&extra| Action::Ask {
                    pile: Some(pile),
                    target,
                    extra,
                }));
            }
        }
        out
    }

    fn ask(&mut self, by: Seat, pile: Option<u8>, target: Seat, extra: Option<CardId>) {
        let mut cards = Vec::new();
        if let Some(card) = pile.and_then(|p| self.piles[usize::from(p)].pop()) {
            cards.push((card, false));
        }
        if let Some(card) = extra {
            cards.push((card, true));
        }
        let mut answers = Vec::new();
        for (card, special) in cards {
            let count = self.count(target, card);
            let placed = count.min(self.reserve);
            self.reserve -= placed;
            answers.push(Answer {
                by,
                target,
                card,
                count,
                placed,
                special,
            });
        }
        self.answers.extend(answers.iter().cloned());
        self.asked[target.index()] = true;
        self.events.push(Event::Asked {
            by,
            target,
            answers,
        });
        if self.last == Some(by) {
            // [R] The last player's final question; then an accusation.
            self.final_asked = true;
            return;
        }
        if self.reserve <= STOP_AT {
            self.finish(End::Reserve);
            return;
        }
        self.turn = target;
        self.settle();
    }

    /// ❓7: a player with nothing to do ends the game.
    fn settle(&mut self) {
        if self.phase == Phase::Turn && self.turn_actions(self.turn).is_empty() {
            self.finish(End::Stuck);
        }
    }

    /// [U] One accuses only on one's turn, before asking; never again
    /// after a wrong accusation.
    fn turn_actions(&self, seat: Seat) -> Vec<Action> {
        let mut out = self.asks(seat);
        if !self.wrong[seat.index()] {
            out.extend(all_trios().map(|trio| Action::Accuse { trio }));
        }
        out
    }

    /// [R] Right: the game stops and the others guess; wrong: the accuser
    /// plays on without accusing. Once all but one are wrong, the last one
    /// asks a final question and must accuse (❓4: at once).
    fn accuse(&mut self, seat: Seat, trio: [Suspect; 3]) {
        let right = trio == self.trio;
        self.events.push(Event::Accused { seat, right });
        if self.last == Some(seat) {
            if right {
                self.accuser = Some(seat);
                self.scores[seat.index()] = u16::from(self.reserve);
                self.finish(End::Solved);
            } else {
                self.wrong[seat.index()] = true;
                self.finish(End::Failed);
            }
            return;
        }
        if right {
            self.accuser = Some(seat);
            self.guesses[seat.index()] = Some(trio);
            self.scores[seat.index()] = 3 * u16::from(self.reserve);
            self.phase = Phase::Guess;
            self.close_guesses();
            return;
        }
        self.wrong[seat.index()] = true;
        let left: Vec<Seat> = self.seats().filter(|x| !self.wrong[x.index()]).collect();
        if let [last] = left[..] {
            self.last = Some(last);
            self.final_asked = false;
            self.turn = last;
        }
        self.settle();
    }

    /// [R] Once everyone has guessed, each right guess scores a point per
    /// token left.
    fn close_guesses(&mut self) {
        if !BlackVienna::pending(self).is_empty() {
            return;
        }
        for seat in self.seats().collect::<Vec<_>>() {
            if Some(seat) != self.accuser && self.guesses[seat.index()] == Some(self.trio) {
                self.scores[seat.index()] = u16::from(self.reserve);
            }
        }
        self.finish(End::Solved);
    }

    fn finish(&mut self, end: End) {
        self.phase = Phase::Over;
        self.events.push(Event::Revealed {
            end,
            trio: self.trio,
            accuser: self.accuser,
            guesses: self.guesses.clone(),
            scores: self.scores.clone(),
        });
    }
}

impl Game for BlackVienna {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "black-vienna",
            name: "Black Vienna",
            min_players: 3,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        // ❓1: the dealer is drawn from the seed.
        let dealer = Seat((seed % u64::from(players.max(1))) as u8);
        let d = deal(players, dealer, seed);
        State {
            players,
            dealer,
            seed,
            hands: d.hands,
            trio: d.trio,
            piles: d.piles,
            answers: Vec::new(),
            reserve: TOKENS,
            asked: vec![false; n],
            wrong: vec![false; n],
            phase: Phase::Turn,
            turn: dealer,
            last: None,
            final_asked: false,
            guesses: vec![None; n],
            accuser: None,
            scores: vec![0; n],
            events: Vec::new(),
        }
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Turn => Pending::one(s.turn),
            Phase::Guess => Pending(
                s.seats()
                    .filter(|x| !s.wrong[x.index()] && s.guesses[x.index()].is_none())
                    .collect(),
            ),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        match s.phase {
            Phase::Turn if s.turn == seat => s.turn_actions(seat),
            Phase::Guess if BlackVienna::pending(s).contains(seat) => {
                all_trios().map(|trio| Action::Guess { trio }).collect()
            }
            _ => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if !Self::pending(s).contains(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        s.events.clear();
        match *action {
            Action::Ask {
                pile,
                target,
                extra,
            } => s.ask(seat, pile, target, extra),
            Action::Accuse { trio } => s.accuse(seat, trio),
            Action::Guess { trio } => {
                s.guesses[seat.index()] = Some(trio);
                s.close_guesses();
            }
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// While the others guess, nobody learns who has already guessed.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        let pending = Self::pending(s);
        match (s.phase, viewer) {
            (Phase::Guess, Viewer::Seat(seat)) if pending.contains(seat) => Pending::one(seat),
            (Phase::Guess, _) => Pending::none(),
            _ => pending,
        }
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
