//! Tiny games that exercise the platform in tests.

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::{
    Determinize, Game, GameMeta, GameOption, Heuristics, Illegal, OptionKind, Outcome, Pending,
    Seat, Viewer,
};

/// Take 1 to 3 stones; whoever takes the last one wins, everyone else ties second.
pub struct Nim;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NimConfig {
    pub pile: u8,
}

impl Default for NimConfig {
    fn default() -> Self {
        Self { pile: 10 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NimState {
    pub pile: u8,
    pub players: u8,
    pub current: Seat,
    pub winner: Option<Seat>,
}

impl Game for Nim {
    type State = NimState;
    type Action = u8;
    type View = NimState;
    type Config = NimConfig;

    fn meta() -> GameMeta {
        GameMeta {
            id: "nim",
            name: "Nim",
            min_players: 2,
            max_players: 4,
            hidden_info: false,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "pile",
            kind: OptionKind::Number {
                min: 1,
                max: 30,
                default: 10,
            },
        }]
    }

    fn check_config(config: &NimConfig, _players: u8) -> Result<(), Illegal> {
        if (1..=30).contains(&config.pile) {
            Ok(())
        } else {
            Err(Illegal::new("Nim se joue avec un tas de 1 à 30."))
        }
    }

    fn new(config: &NimConfig, players: u8, _seed: u64) -> NimState {
        NimState {
            pile: config.pile,
            players,
            current: Seat(0),
            winner: None,
        }
    }

    fn pending(s: &NimState) -> Pending {
        if s.winner.is_some() {
            Pending::none()
        } else {
            Pending::one(s.current)
        }
    }

    fn legal_actions(s: &NimState, seat: Seat) -> Vec<u8> {
        if s.winner.is_some() || seat != s.current {
            return Vec::new();
        }
        (1..=3.min(s.pile)).collect()
    }

    fn apply(s: &mut NimState, seat: Seat, take: &u8) -> Result<(), Illegal> {
        if !Self::legal_actions(s, seat).contains(take) {
            return Err(Illegal::new("coup interdit"));
        }
        s.pile -= take;
        if s.pile == 0 {
            s.winner = Some(seat);
        } else {
            s.current = Seat((seat.0 + 1) % s.players);
        }
        Ok(())
    }

    fn view(s: &NimState, _viewer: Viewer) -> NimState {
        s.clone()
    }

    fn outcome(s: &NimState) -> Option<Outcome> {
        s.winner
            .map(|w| Outcome::Ranked((0..s.players).map(|i| u8::from(i != w.0)).collect()))
    }
    fn timeout_action(s: &NimState, seat: Seat) -> Option<u8> {
        Self::pending(s).contains(seat).then_some(1)
    }
}

impl Determinize for Nim {
    fn determinize<R: Rng + ?Sized>(s: &NimState, _observer: Seat, _rng: &mut R) -> NimState {
        s.clone()
    }
}

impl Heuristics for Nim {
    fn rollout_action<R: Rng + ?Sized>(s: &NimState, seat: Seat, rng: &mut R) -> u8 {
        *Self::legal_actions(s, seat)
            .choose(rng)
            .expect("pending seat has a move")
    }
}

/// A game that never ends: seat 0 keeps incrementing. Only an `evaluate`
/// cutoff lets a simulation stop.
pub struct Counter;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterState {
    pub n: u32,
}

impl Game for Counter {
    type State = CounterState;
    type Action = u8;
    type View = CounterState;
    type Config = ();

    fn meta() -> GameMeta {
        GameMeta {
            id: "counter",
            name: "Counter",
            min_players: 1,
            max_players: 1,
            hidden_info: false,
            clock: false,
        }
    }
    fn new(_config: &(), _players: u8, _seed: u64) -> CounterState {
        CounterState { n: 0 }
    }
    fn pending(_s: &CounterState) -> Pending {
        Pending::one(Seat(0))
    }
    fn legal_actions(_s: &CounterState, seat: Seat) -> Vec<u8> {
        if seat == Seat(0) {
            vec![1, 2]
        } else {
            Vec::new()
        }
    }
    fn apply(s: &mut CounterState, seat: Seat, a: &u8) -> Result<(), Illegal> {
        if seat != Seat(0) || !(1..=2).contains(a) {
            return Err(Illegal::new("coup interdit"));
        }
        s.n += u32::from(*a);
        Ok(())
    }
    fn view(s: &CounterState, _viewer: Viewer) -> CounterState {
        s.clone()
    }
    fn outcome(_s: &CounterState) -> Option<Outcome> {
        None
    }
}

impl Determinize for Counter {
    fn determinize<R: Rng + ?Sized>(
        s: &CounterState,
        _observer: Seat,
        _rng: &mut R,
    ) -> CounterState {
        s.clone()
    }
}

impl Heuristics for Counter {
    fn rollout_action<R: Rng + ?Sized>(_s: &CounterState, _seat: Seat, rng: &mut R) -> u8 {
        rng.gen_range(1..=2)
    }
    fn evaluate(root: &CounterState, sim: &CounterState) -> Option<Vec<f64>> {
        (sim.n >= root.n + 5).then(|| vec![1.0])
    }
    /// The search only ever considers adding 1.
    fn search_actions(_s: &CounterState, _seat: Seat) -> Vec<u8> {
        vec![1]
    }
}

/// Nim whose timeout action is illegal once the pile is below 3.
pub struct BadClock;

impl Game for BadClock {
    type State = NimState;
    type Action = u8;
    type View = NimState;
    type Config = ();

    fn meta() -> GameMeta {
        GameMeta {
            id: "badclock",
            ..Nim::meta()
        }
    }
    fn new(_config: &(), players: u8, seed: u64) -> NimState {
        Nim::new(&NimConfig::default(), players, seed)
    }
    fn pending(s: &NimState) -> Pending {
        Nim::pending(s)
    }
    fn legal_actions(s: &NimState, seat: Seat) -> Vec<u8> {
        Nim::legal_actions(s, seat)
    }
    fn apply(s: &mut NimState, seat: Seat, a: &u8) -> Result<(), Illegal> {
        Nim::apply(s, seat, a)
    }
    fn view(s: &NimState, viewer: Viewer) -> NimState {
        Nim::view(s, viewer)
    }
    fn outcome(s: &NimState) -> Option<Outcome> {
        Nim::outcome(s)
    }
    fn timeout_action(s: &NimState, seat: Seat) -> Option<u8> {
        Nim::pending(s).contains(seat).then_some(3)
    }
}

impl Determinize for BadClock {
    fn determinize<R: Rng + ?Sized>(s: &NimState, _observer: Seat, _rng: &mut R) -> NimState {
        s.clone()
    }
}

/// Nim announcing a default pile that `NimConfig::default()` does not have.
pub struct BadOptions;

impl Game for BadOptions {
    type State = NimState;
    type Action = u8;
    type View = NimState;
    type Config = NimConfig;

    fn meta() -> GameMeta {
        GameMeta {
            id: "badoptions",
            ..Nim::meta()
        }
    }
    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "pile",
            kind: OptionKind::Number {
                min: 1,
                max: 30,
                default: 7,
            },
        }]
    }
    fn new(config: &NimConfig, players: u8, seed: u64) -> NimState {
        Nim::new(config, players, seed)
    }
    fn pending(s: &NimState) -> Pending {
        Nim::pending(s)
    }
    fn legal_actions(s: &NimState, seat: Seat) -> Vec<u8> {
        Nim::legal_actions(s, seat)
    }
    fn apply(s: &mut NimState, seat: Seat, a: &u8) -> Result<(), Illegal> {
        Nim::apply(s, seat, a)
    }
    fn view(s: &NimState, viewer: Viewer) -> NimState {
        Nim::view(s, viewer)
    }
    fn outcome(s: &NimState) -> Option<Outcome> {
        Nim::outcome(s)
    }
}

impl Determinize for BadOptions {
    fn determinize<R: Rng + ?Sized>(s: &NimState, _observer: Seat, _rng: &mut R) -> NimState {
        s.clone()
    }
}

/// Each seat is dealt one hidden card from 0..=6, then every seat answers
/// Stay or Fold at the same time. Highest card among stayers wins; folders
/// tie behind the winner; other stayers finish last. Nobody stays: all tie.
pub struct HighCard;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Choice {
    Stay,
    Fold,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighCardState {
    pub cards: Vec<u8>,
    /// Undealt cards, hidden from everyone.
    pub deck: Vec<u8>,
    pub answers: Vec<Option<Choice>>,
}

#[derive(Debug, Serialize)]
pub struct HighCardView {
    pub my_card: Option<u8>,
    pub my_answer: Option<Choice>,
    pub answered: Vec<bool>,
    /// Cards and answers, once everyone has answered.
    pub revealed: Option<(Vec<u8>, Vec<Choice>)>,
}

impl HighCard {
    fn finished(s: &HighCardState) -> Option<Vec<Choice>> {
        s.answers.iter().copied().collect()
    }
}

impl Game for HighCard {
    type State = HighCardState;
    type Action = Choice;
    type View = HighCardView;
    type Config = ();

    fn meta() -> GameMeta {
        GameMeta {
            id: "highcard",
            name: "HighCard",
            min_players: 2,
            max_players: 4,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &(), players: u8, seed: u64) -> HighCardState {
        let mut deck: Vec<u8> = (0..=6).collect();
        deck.shuffle(&mut SmallRng::seed_from_u64(seed));
        let cards = deck.split_off(deck.len() - usize::from(players));
        HighCardState {
            cards,
            deck,
            answers: vec![None; usize::from(players)],
        }
    }

    fn pending(s: &HighCardState) -> Pending {
        Pending(
            (0..s.answers.len() as u8)
                .map(Seat)
                .filter(|x| s.answers[x.index()].is_none())
                .collect(),
        )
    }

    fn legal_actions(s: &HighCardState, seat: Seat) -> Vec<Choice> {
        match s.answers.get(seat.index()) {
            Some(None) => vec![Choice::Stay, Choice::Fold],
            _ => Vec::new(),
        }
    }

    fn apply(s: &mut HighCardState, seat: Seat, choice: &Choice) -> Result<(), Illegal> {
        match s.answers.get_mut(seat.index()) {
            Some(slot @ None) => {
                *slot = Some(*choice);
                Ok(())
            }
            _ => Err(Illegal::new("déjà répondu")),
        }
    }

    fn view(s: &HighCardState, viewer: Viewer) -> HighCardView {
        let mine = match viewer {
            Viewer::Seat(seat) => Some(seat.index()),
            Viewer::Spectator => None,
        };
        HighCardView {
            my_card: mine.map(|i| s.cards[i]),
            my_answer: mine.and_then(|i| s.answers[i]),
            answered: s.answers.iter().map(Option::is_some).collect(),
            revealed: Self::finished(s).map(|answers| (s.cards.clone(), answers)),
        }
    }

    fn outcome(s: &HighCardState) -> Option<Outcome> {
        let answers = Self::finished(s)?;
        let n = answers.len();
        let stayers: Vec<usize> = (0..n).filter(|&i| answers[i] == Choice::Stay).collect();
        let Some(&best) = stayers.iter().max_by_key(|&&i| s.cards[i]) else {
            return Some(Outcome::Ranked(vec![0; n]));
        };
        let folders = (n - stayers.len()) as u8;
        Some(Outcome::Ranked(
            (0..n)
                .map(|i| match answers[i] {
                    _ if i == best => 0,
                    Choice::Fold => 1,
                    Choice::Stay => 1 + folders,
                })
                .collect(),
        ))
    }
}

impl Determinize for HighCard {
    fn determinize<R: Rng + ?Sized>(
        s: &HighCardState,
        observer: Seat,
        rng: &mut R,
    ) -> HighCardState {
        if Self::finished(s).is_some() {
            return s.clone();
        }
        let me = observer.index();
        let mut pool: Vec<u8> = (0..=6).filter(|&c| c != s.cards[me]).collect();
        pool.shuffle(rng);
        let mut d = s.clone();
        for i in (0..s.cards.len()).filter(|&i| i != me) {
            d.cards[i] = pool.pop().expect("enough cards");
            if d.answers[i].is_some() {
                d.answers[i] = Some(if rng.gen_bool(0.5) {
                    Choice::Stay
                } else {
                    Choice::Fold
                });
            }
        }
        d.deck = pool;
        d
    }
}

impl Heuristics for HighCard {
    fn rollout_action<R: Rng + ?Sized>(_s: &HighCardState, _seat: Seat, rng: &mut R) -> Choice {
        if rng.gen_bool(0.5) {
            Choice::Stay
        } else {
            Choice::Fold
        }
    }
}

/// HighCard with a bug: every view shows every card.
pub struct Leaky;

impl Game for Leaky {
    type State = HighCardState;
    type Action = Choice;
    type View = (HighCardView, Vec<u8>);
    type Config = ();

    fn meta() -> GameMeta {
        GameMeta {
            id: "leaky",
            ..HighCard::meta()
        }
    }
    fn new(config: &(), players: u8, seed: u64) -> HighCardState {
        HighCard::new(config, players, seed)
    }
    fn pending(s: &HighCardState) -> Pending {
        HighCard::pending(s)
    }
    fn legal_actions(s: &HighCardState, seat: Seat) -> Vec<Choice> {
        HighCard::legal_actions(s, seat)
    }
    fn apply(s: &mut HighCardState, seat: Seat, a: &Choice) -> Result<(), Illegal> {
        HighCard::apply(s, seat, a)
    }
    fn view(s: &HighCardState, viewer: Viewer) -> Self::View {
        (HighCard::view(s, viewer), s.cards.clone())
    }
    fn outcome(s: &HighCardState) -> Option<Outcome> {
        HighCard::outcome(s)
    }
}

impl Determinize for Leaky {
    fn determinize<R: Rng + ?Sized>(
        s: &HighCardState,
        observer: Seat,
        rng: &mut R,
    ) -> HighCardState {
        HighCard::determinize(s, observer, rng)
    }
}

/// Nim whose AI is steered: it takes 3 from a pile of 10 without searching,
/// and its random moves always take 1.
pub struct SteeredNim;

impl Game for SteeredNim {
    type State = NimState;
    type Action = u8;
    type View = NimState;
    type Config = NimConfig;

    fn meta() -> GameMeta {
        GameMeta {
            id: "steered-nim",
            name: "Steered Nim",
            ..Nim::meta()
        }
    }

    fn new(config: &NimConfig, players: u8, seed: u64) -> NimState {
        Nim::new(config, players, seed)
    }

    fn pending(s: &NimState) -> Pending {
        Nim::pending(s)
    }

    fn legal_actions(s: &NimState, seat: Seat) -> Vec<u8> {
        Nim::legal_actions(s, seat)
    }

    fn apply(s: &mut NimState, seat: Seat, take: &u8) -> Result<(), Illegal> {
        Nim::apply(s, seat, take)
    }

    fn view(s: &NimState, viewer: Viewer) -> NimState {
        Nim::view(s, viewer)
    }

    fn outcome(s: &NimState) -> Option<Outcome> {
        Nim::outcome(s)
    }
}

impl Determinize for SteeredNim {
    fn determinize<R: Rng + ?Sized>(s: &NimState, _observer: Seat, _rng: &mut R) -> NimState {
        s.clone()
    }
}

impl Heuristics for SteeredNim {
    fn rollout_action<R: Rng + ?Sized>(s: &NimState, seat: Seat, rng: &mut R) -> u8 {
        Nim::rollout_action(s, seat, rng)
    }

    fn scripted_action(s: &NimState, _seat: Seat) -> Option<u8> {
        (s.pile == 10).then_some(3)
    }

    fn policy_action<R: Rng + ?Sized>(
        s: &NimState,
        _seat: Seat,
        level: crate::ai::Level,
        _rng: &mut R,
    ) -> Option<u8> {
        (s.pile == 10 && level == crate::ai::Level::Easy).then_some(2)
    }

    fn random_actions(_s: &NimState, _seat: Seat) -> Vec<u8> {
        vec![1]
    }
}

#[cfg(test)]
mod nim_tests {
    use super::*;
    use crate::{Game, Outcome, Seat};

    #[test]
    fn nim_times_out_by_taking_one() {
        let s = Nim::new(&NimConfig { pile: 4 }, 2, 0);
        assert_eq!(Nim::timeout_action(&s, Seat(0)), Some(1));
        assert_eq!(Nim::timeout_action(&s, Seat(1)), None);
    }

    #[test]
    fn taking_the_last_stone_wins() {
        let mut s = Nim::new(&NimConfig { pile: 4 }, 2, 0);
        assert_eq!(Nim::legal_actions(&s, Seat(0)), vec![1, 2, 3]);
        Nim::apply(&mut s, Seat(0), &3).unwrap();
        assert_eq!(Nim::legal_actions(&s, Seat(1)), vec![1]);
        Nim::apply(&mut s, Seat(1), &1).unwrap();
        assert_eq!(Nim::outcome(&s), Some(Outcome::Ranked(vec![1, 0])));
        assert!(Nim::pending(&s).is_empty());
    }

    #[test]
    fn illegal_moves_are_refused_without_changing_the_state() {
        let mut s = Nim::new(&NimConfig { pile: 4 }, 2, 0);
        let before = s.clone();
        assert!(Nim::apply(&mut s, Seat(0), &4).is_err());
        assert!(Nim::apply(&mut s, Seat(1), &1).is_err());
        assert_eq!(s, before);
    }
}
