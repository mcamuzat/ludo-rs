//! Président (Trou du cul): be the first to shed your cards, climbing over
//! the previous play with as many cards of a higher rank. Rules and their
//! sources: RULES.md.

mod combos;
mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use combos::{exchange_value, face_value, hand_order, plays, strength, value, JOKERS_ALONE};
pub use view::View;

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{deal_evenly, pack_52, pack_54, Card, Face, Rank};

pub struct President;

pub const DEFAULT_ROUNDS: u8 = 5;
pub const MAX_ROUNDS: u8 = 10;

/// No jokers, strong jokers above the 2, or wild jokers (spec 4 §3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Jokers {
    #[default]
    Off,
    High,
    Wild,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds in a game, 1 to `MAX_ROUNDS`.
    pub rounds: u8,
    /// [W] Four of a kind reverses the order.
    pub revolution: bool,
    /// [W] Going out on a 2 (or a joker) makes you the Scum.
    pub no_finish_on_two: bool,
    pub jokers: Jokers,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rounds: DEFAULT_ROUNDS,
            revolution: false,
            no_finish_on_two: false,
            jokers: Jokers::Off,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// `naturals` cards of `rank` and `jokers` jokers; `rank` is `None` for
    /// jokers alone. Suits do not matter: the engine picks them.
    Play {
        rank: Option<Rank>,
        naturals: u8,
        jokers: u8,
    },
    Pass,
    /// President or Vice-President: the cards handed back, in `Face` order.
    Give {
        faces: Vec<Face>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Title {
    President,
    VicePresident,
    Neutral,
    Secretary,
    Scum,
}

/// [W] The title of whoever finished at `place` (0: first out) among
/// `players` (4 or more).
pub fn title(place: usize, players: usize) -> Title {
    match place {
        0 => Title::President,
        1 => Title::VicePresident,
        p if p + 1 == players => Title::Scum,
        p if p + 2 == players => Title::Secretary,
        _ => Title::Neutral,
    }
}

/// Ranks from scores (0: first); equal scores share a rank.
pub fn standings(scores: &[u32]) -> Vec<u8> {
    scores
        .iter()
        .map(|&mine| scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Exchange,
    Play,
    Over,
}

/// The play to beat.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trick {
    pub seat: Seat,
    pub rank: Option<Rank>,
    pub cards: Vec<Card>,
}

/// Cards handed over at the start of a round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transfer {
    pub from: Seat,
    pub to: Seat,
    pub cards: Vec<Card>,
}

/// What happened, for the board to tell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    RoundStarted {
        round: u8,
        dealer: Seat,
        leader: Seat,
        aside: Vec<Card>,
    },
    /// Cards handed over: which ones, only the two players know.
    Exchanged {
        from: Seat,
        to: Seat,
        count: u8,
    },
    Played {
        seat: Seat,
        rank: Option<Rank>,
        cards: Vec<Card>,
    },
    Passed {
        seat: Seat,
    },
    TrickClosed {
        leader: Seat,
    },
    Revolution {
        reversed: bool,
    },
    /// Out of cards; `fouled`: on a forbidden card.
    Finished {
        seat: Seat,
        fouled: bool,
    },
    /// The round's order (first out first), the points it earned and the
    /// scores after it.
    RoundScored {
        order: Vec<Seat>,
        points: Vec<u32>,
        scores: Vec<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub config: Config,
    /// Every deal comes from it: never in a view.
    pub seed: u64,
    pub round_no: u8,
    pub phase: Phase,
    pub hands: Vec<Vec<Card>>,
    pub set_aside: Vec<Card>,
    /// Titles earned in the previous round (empty during the first).
    pub titles: Vec<Title>,
    /// A revolution is in force.
    pub reversed: bool,
    pub trick: Option<Trick>,
    /// Who passed, in a row, since the last play.
    pub passes: Vec<Seat>,
    pub turn: Option<Seat>,
    /// This round: who went out, in order.
    pub out: Vec<Seat>,
    /// This round: who went out on a forbidden card, in order.
    pub fouled: Vec<Seat>,
    /// This round's exchange.
    pub transfers: Vec<Transfer>,
    /// Exchange: who still has to hand cards back.
    pub to_give: Vec<Seat>,
    pub scores: Vec<u32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

impl State {
    fn active(&self) -> Vec<Seat> {
        (0..self.players)
            .map(Seat)
            .filter(|s| !self.hands[s.index()].is_empty())
            .collect()
    }

    /// The next seat to the left of `seat` that still holds cards.
    fn next_active(&self, seat: Seat) -> Seat {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| !self.hands[s.index()].is_empty())
            .unwrap_or(seat)
    }

    fn seat_with(&self, title: Title) -> Option<Seat> {
        self.titles
            .iter()
            .position(|&t| t == title)
            .map(|i| Seat(i as u8))
    }

    /// A fresh deal, from the round's own stream of the seed so that a log
    /// replays identically.
    fn start_round(&mut self) {
        let pack = match self.config.jokers {
            Jokers::Off => pack_52(),
            Jokers::High | Jokers::Wild => pack_54(),
        };
        let stream = self.seed ^ u64::from(self.round_no + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let deal = deal_evenly(pack, self.players, stream);
        self.begin_round(deal.hands, deal.set_aside);
    }

    /// The next round, with these hands. [W] Seat 0 deals the first round,
    /// the Scum the others; the player on the dealer's left leads; from the
    /// second round on, the cards are exchanged first.
    fn begin_round(&mut self, hands: Vec<Vec<Card>>, aside: Vec<Card>) {
        self.round_no += 1;
        self.hands = hands;
        self.set_aside = aside;
        self.reversed = false;
        self.trick = None;
        self.passes.clear();
        self.out.clear();
        self.fouled.clear();
        self.transfers.clear();
        self.to_give.clear();
        let dealer = self.seat_with(Title::Scum).unwrap_or(Seat(0));
        let leader = Seat((dealer.0 + 1) % self.players);
        self.turn = Some(leader);
        self.events.push(Event::RoundStarted {
            round: self.round_no,
            dealer,
            leader,
            aside: self.set_aside.clone(),
        });
        self.phase = Phase::Play;
        let (Some(president), Some(vice), Some(secretary), Some(scum)) = (
            self.seat_with(Title::President),
            self.seat_with(Title::VicePresident),
            self.seat_with(Title::Secretary),
            self.seat_with(Title::Scum),
        ) else {
            return;
        };
        // [W] The Scum hands the President their two best cards, the
        // Secretary the Vice-President their best one.
        self.hand_over(scum, president, 2);
        self.hand_over(secretary, vice, 1);
        self.to_give = vec![president, vice];
        self.phase = Phase::Exchange;
    }

    fn hand_over(&mut self, from: Seat, to: Seat, count: usize) {
        let cards = (0..count)
            .filter_map(|_| take_best(&mut self.hands[from.index()]))
            .collect();
        self.receive(from, to, cards);
    }

    fn receive(&mut self, from: Seat, to: Seat, cards: Vec<Card>) {
        self.hands[to.index()].extend(cards.iter().copied());
        self.events.push(Event::Exchanged {
            from,
            to,
            count: cards.len() as u8,
        });
        self.transfers.push(Transfer { from, to, cards });
    }

    /// [W] The President hands back two cards, the Vice-President one.
    fn gift_size(&self, seat: Seat) -> usize {
        match self.titles.get(seat.index()) {
            Some(Title::President) => 2,
            _ => 1,
        }
    }

    /// Every choice of cards to hand back, by face, from the whole hand
    /// (cards just received included).
    fn gifts(&self, seat: Seat) -> Vec<Action> {
        let mut faces: Vec<Face> = self.hands[seat.index()].iter().map(|c| c.face()).collect();
        faces.sort();
        let mut choices: Vec<Vec<Face>> = Vec::new();
        for (i, &a) in faces.iter().enumerate() {
            if self.gift_size(seat) == 1 {
                choices.push(vec![a]);
            } else {
                for &b in &faces[i + 1..] {
                    choices.push(vec![a, b]);
                }
            }
        }
        choices.sort();
        choices.dedup();
        choices
            .into_iter()
            .map(|faces| Action::Give { faces })
            .collect()
    }

    /// [W] The President gives back to the Scum, the Vice-President to the
    /// Secretary; play starts once both have.
    fn give(&mut self, seat: Seat, faces: &[Face]) {
        let back_to = match self.titles[seat.index()] {
            Title::President => Title::Scum,
            _ => Title::Secretary,
        };
        let to = self.seat_with(back_to).expect("every title is held");
        let cards = faces.iter().map(|&f| self.take(seat, f)).collect();
        self.receive(seat, to, cards);
        self.to_give.retain(|&s| s != seat);
        if self.to_give.is_empty() {
            self.phase = Phase::Play;
        }
    }

    /// Removes a card showing `face` from `seat`'s hand, the first suit in
    /// `Suit` order first.
    fn take(&mut self, seat: Seat, face: Face) -> Card {
        let hand = &mut self.hands[seat.index()];
        let at = hand
            .iter()
            .enumerate()
            .filter(|(_, c)| c.face() == face)
            .min_by_key(|(_, c)| **c)
            .map(|(i, _)| i)
            .expect("a legal action names cards in hand");
        hand.remove(at)
    }

    fn play(&mut self, seat: Seat, rank: Option<Rank>, naturals: u8, jokers: u8) {
        let mut cards = Vec::new();
        if let Some(rank) = rank {
            for _ in 0..naturals {
                cards.push(self.take(seat, Face::Natural { rank }));
            }
        }
        for _ in 0..jokers {
            cards.push(self.take(seat, Face::Joker));
        }
        self.events.push(Event::Played {
            seat,
            rank,
            cards: cards.clone(),
        });
        // [W] Going out on the top card (❓3: the 3 during a revolution) or a
        // joker; ❓4 judged in the order in force when the cards are laid.
        let top = self.top_rank();
        let fouled = self.config.no_finish_on_two
            && cards
                .iter()
                .any(|&c| c == Card::Joker || c.rank() == Some(top));
        // [W] Four cards of a rank (❓7: jokers included) reverse the order
        // until the end of the round; ❓2 another four turn it back.
        if self.config.revolution && cards.len() == 4 {
            self.reversed = !self.reversed;
            self.events.push(Event::Revolution {
                reversed: self.reversed,
            });
        }
        self.trick = Some(Trick { seat, rank, cards });
        self.passes.clear();
        if self.hands[seat.index()].is_empty() {
            if fouled {
                self.fouled.push(seat);
            } else {
                self.out.push(seat);
            }
            self.events.push(Event::Finished { seat, fouled });
        }
        self.after_play(seat);
    }

    /// [W] The strongest natural rank: the 2, or the 3 during a revolution.
    pub(crate) fn top_rank(&self) -> Rank {
        if self.reversed {
            Rank::Three
        } else {
            Rank::Two
        }
    }

    fn after_play(&mut self, seat: Seat) {
        if self.active().len() <= 1 {
            self.end_round();
        } else {
            self.turn = Some(self.next_active(seat));
        }
    }

    /// [P] Passing keeps you in the trick. [W] Once everyone else still in
    /// the round has passed in a row, the last to play leads; if they are
    /// out, the player on their left.
    fn pass(&mut self, seat: Seat) {
        self.passes.push(seat);
        self.events.push(Event::Passed { seat });
        let owner = self.trick.as_ref().expect("a pass answers a play").seat;
        let owner_in = !self.hands[owner.index()].is_empty();
        if self.passes.len() + usize::from(owner_in) >= self.active().len() {
            let leader = if owner_in {
                owner
            } else {
                self.next_active(owner)
            };
            self.trick = None;
            self.passes.clear();
            self.events.push(Event::TrickClosed { leader });
            self.turn = Some(leader);
        } else {
            self.turn = Some(self.next_active(seat));
        }
    }

    /// [W] Titles and points: n−1 for the first out, one less per place, 0
    /// for the last. The player still holding cards comes after everyone
    /// out; those out on a forbidden card come after them, the first of them
    /// last (❓5).
    fn end_round(&mut self) {
        let order: Vec<Seat> = self
            .out
            .iter()
            .copied()
            .chain(self.active())
            .chain(self.fouled.iter().rev().copied())
            .collect();
        let n = order.len();
        let mut points = vec![0; n];
        let mut titles = vec![Title::Neutral; n];
        for (place, seat) in order.iter().enumerate() {
            points[seat.index()] = (n - 1 - place) as u32;
            titles[seat.index()] = title(place, n);
        }
        for (score, p) in self.scores.iter_mut().zip(&points) {
            *score += p;
        }
        self.events.push(Event::RoundScored {
            order,
            points,
            scores: self.scores.clone(),
        });
        self.titles = titles;
        self.trick = None;
        self.turn = None;
        if self.round_no >= self.config.rounds.clamp(1, MAX_ROUNDS) {
            self.phase = Phase::Over;
        } else {
            self.start_round();
        }
    }

    /// Orders choices weakest first: value, then jokers spent, then cards.
    pub(crate) fn cost(&self, action: &Action) -> (u32, u8, u8) {
        match action {
            Action::Play {
                rank,
                naturals,
                jokers,
            } => (u32::from(value(*rank, self.reversed)), *jokers, *naturals),
            Action::Give { faces } => (faces.iter().map(|&f| u32::from(face_value(f))).sum(), 0, 0),
            Action::Pass => (0, 0, 0),
        }
    }
}

/// [W] The best card of a hand, for the exchange: jokers first, then 2, ace…
fn take_best(hand: &mut Vec<Card>) -> Option<Card> {
    let at = hand
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| (exchange_value(**c), **c))
        .map(|(i, _)| i)?;
    Some(hand.remove(at))
}

impl Game for President {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cards-president",
            name: "Président",
            min_players: 4,
            max_players: 7,
            hidden_info: true,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![
            GameOption {
                key: "rounds",
                kind: OptionKind::Number {
                    min: 1,
                    max: u32::from(MAX_ROUNDS),
                    default: u32::from(DEFAULT_ROUNDS),
                },
            },
            GameOption {
                key: "revolution",
                kind: OptionKind::Toggle { default: false },
            },
            GameOption {
                key: "no_finish_on_two",
                kind: OptionKind::Toggle { default: false },
            },
            GameOption {
                key: "jokers",
                kind: OptionKind::Choice {
                    values: &["Off", "High", "Wild"],
                    default: "Off",
                },
            },
        ]
    }

    fn check_config(config: &Config, _players: u8) -> Result<(), Illegal> {
        if (1..=MAX_ROUNDS).contains(&config.rounds) {
            Ok(())
        } else {
            Err(Illegal::new(format!(
                "Le Président se joue en 1 à {MAX_ROUNDS} manches."
            )))
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            config: config.clone(),
            seed,
            round_no: 0,
            phase: Phase::Play,
            hands: vec![Vec::new(); n],
            set_aside: Vec::new(),
            titles: Vec::new(),
            reversed: false,
            trick: None,
            passes: Vec::new(),
            turn: None,
            out: Vec::new(),
            fouled: Vec::new(),
            transfers: Vec::new(),
            to_give: Vec::new(),
            scores: vec![0; n],
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Exchange => Pending(s.to_give.clone()),
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        match s.phase {
            Phase::Exchange if s.to_give.contains(&seat) => s.gifts(seat),
            Phase::Play if s.turn == Some(seat) => plays(
                &s.hands[seat.index()],
                s.trick.as_ref(),
                s.config.jokers,
                s.reversed,
            ),
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
        match action {
            Action::Play {
                rank,
                naturals,
                jokers,
            } => s.play(seat, *rank, *naturals, *jokers),
            Action::Pass => s.pass(seat),
            Action::Give { faces } => s.give(seat, faces),
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        (s.phase == Phase::Over).then(|| Outcome::Ranked(standings(&s.scores)))
    }

    /// Pass when allowed; otherwise lead, or hand back, the weakest cards.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        if legal.contains(&Action::Pass) {
            return Some(Action::Pass);
        }
        legal.into_iter().min_by_key(|a| s.cost(a))
    }
}
