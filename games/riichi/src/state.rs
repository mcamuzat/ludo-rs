use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use ludo_core::Seat;
use riichienv_core::action::{Action as EngineAction, Phase};
use riichienv_core::rule::GameRule;
use riichienv_core::state::legal_actions::GameStateLegalActions;
use riichienv_core::state::GameState;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::action::{self, Action};
use crate::scoring;
use crate::view::{DrawKind, HandResult};
use crate::Tile;

/// riichienv's game mode 2: hanchan (East and South, Mahjong Soul extension).
const HANCHAN: u8 = 2;

/// One engine step: every answer given for one decision.
type Step = Vec<(Seat, Action)>;

/// Accepted steps as a persistent list: the AI clones states constantly,
/// and a clone shares the history instead of copying it.
#[derive(Clone, Debug, Default)]
struct History(Option<Arc<Node>>);

#[derive(Debug)]
struct Node {
    step: Step,
    prev: History,
}

impl History {
    fn push(&mut self, step: Step) {
        let prev = std::mem::take(self);
        self.0 = Some(Arc::new(Node { step, prev }));
    }

    fn to_vec(&self) -> Vec<Step> {
        let mut steps = Vec::new();
        let mut at = &self.0;
        while let Some(node) = at {
            steps.push(node.step.clone());
            at = &node.prev.0;
        }
        steps.reverse();
        steps
    }
}

#[derive(Clone, Debug)]
pub struct State {
    seed: u64,
    /// Imposed first-hand wall (tests), in draw order.
    wall: Option<Vec<u8>>,
    history: History,
    /// Answers received for the current decision, not yet given to the engine.
    pub(crate) answers: BTreeMap<Seat, Action>,
    pub(crate) inner: GameState,
    /// Not saved: the replay recomputes them.
    pub(crate) hands_played: u32,
    pub(crate) last_hand: Option<HandResult>,
}

/// What is saved: the engine is rebuilt by replaying it.
#[derive(Serialize, Deserialize)]
struct Saved {
    seed: u64,
    wall: Option<Vec<u8>>,
    steps: Vec<Step>,
    answers: Vec<(Seat, Action)>,
}

impl State {
    pub fn start(seed: u64, wall: Option<Vec<u8>>) -> Self {
        let mut inner = GameState::new(HANCHAN, true, Some(seed), 0, GameRule::default_mjsoul());
        if let Some(w) = &wall {
            assert_eq!(w.len(), 136, "a wall has 136 tiles");
            inner._initialize_round(0, 0, 0, 0, Some(w.clone()), None);
        }
        Self {
            seed,
            wall,
            history: History::default(),
            answers: BTreeMap::new(),
            inner,
            hands_played: 0,
            last_hand: None,
        }
    }

    pub fn hands_played(&self) -> u32 {
        self.hands_played
    }

    /// Who let the tile go and which tile, while players may call it (a
    /// discard, or the tile added to a kan for chankan).
    pub(crate) fn claim_source(&self) -> Option<(u8, u8)> {
        if !self.in_claim_window() {
            return None;
        }
        match &self.inner.pending_kan {
            Some((pid, kan)) => kan
                .tile
                .or(kan.consume_tiles.first().copied())
                .map(|t| (*pid, t)),
            None => self.inner.last_discard,
        }
    }

    pub fn scores(&self) -> [i32; 4] {
        std::array::from_fn(|i| self.inner.players[i].score)
    }

    /// Identifies the hand being played: it changes when a hand ends.
    pub(crate) fn hand_key(&self) -> (u8, u8, u8) {
        (
            self.inner.round_wind,
            self.inner.kyoku_idx,
            self.inner.honba,
        )
    }

    pub(crate) fn is_pending(&self, seat: Seat) -> bool {
        !self.inner.is_done
            && self.inner.active_players.contains(&seat.0)
            && !self.answers.contains_key(&seat)
    }

    pub(crate) fn pending_seats(&self) -> Vec<Seat> {
        let mut seats: Vec<Seat> = (0..4u8).map(Seat).filter(|&s| self.is_pending(s)).collect();
        seats.sort();
        seats
    }

    pub(crate) fn in_claim_window(&self) -> bool {
        self.inner.phase == Phase::WaitResponse
    }

    /// Legal actions of a pending seat, each with the engine action behind it.
    pub(crate) fn engine_legal(&self, seat: Seat) -> Vec<(Action, EngineAction)> {
        if !self.is_pending(seat) {
            return Vec::new();
        }
        action::legal(&self.inner._get_legal_actions_internal(seat.0))
    }

    /// Records `seat`'s answer; runs the engine once every asked seat has answered.
    /// The caller has checked that the action is legal.
    pub(crate) fn answer(&mut self, seat: Seat, action: Action) {
        self.answers.insert(seat, action);
        if self
            .inner
            .active_players
            .iter()
            .all(|p| self.answers.contains_key(&Seat(*p)))
        {
            self.flush();
        }
    }

    fn flush(&mut self) {
        let step: Step = std::mem::take(&mut self.answers).into_iter().collect();
        let mut engine_actions = HashMap::new();
        for (seat, a) in &step {
            let legal = action::legal(&self.inner._get_legal_actions_internal(seat.0));
            let (_, e) = legal
                .into_iter()
                .find(|(ours, _)| ours == a)
                .expect("answers are validated");
            engine_actions.insert(seat.0, e);
        }
        // The engine forgets a hand's result once the next hand starts: read it now.
        let hand = self.hand_key();
        let scores = self.scores();
        let wins = scoring::preview(&self.inner, self.claim_source(), &step);
        let ura = if wins
            .iter()
            .any(|w| self.inner.players[w.seat.index()].riichi_declared)
        {
            scoring::ura_indicators(&self.inner)
        } else {
            Vec::new()
        };
        let exhausted = self.inner.wall.drawable_count == 0;
        let nine_terminals = step.iter().any(|(_, a)| *a == Action::KyushuKyuhai);

        self.inner.step(&engine_actions);
        if let Some(e) = self.inner.last_error.take() {
            panic!("riichienv refused validated answers {step:?}: {e}");
        }
        self.history.push(step);

        if self.inner.is_done || self.hand_key() != hand {
            self.hands_played += 1;
            let after = self.scores();
            let draw = wins.is_empty().then_some(if nine_terminals {
                DrawKind::NineTerminals
            } else if exhausted {
                DrawKind::Exhaustive
            } else {
                DrawKind::Abortive
            });
            self.last_hand = Some(HandResult {
                number: self.hands_played,
                wins,
                draw,
                deltas: (0..4).map(|i| after[i] - scores[i]).collect(),
                ura_indicators: ura.into_iter().map(Tile::from_id).collect(),
            });
        }
    }

    fn replay(saved: Saved) -> Result<Self, String> {
        let mut s = Self::start(saved.seed, saved.wall);
        for (i, step) in saved.steps.into_iter().enumerate() {
            for (seat, a) in step {
                if !s.engine_legal(seat).iter().any(|(ours, _)| *ours == a) {
                    return Err(format!(
                        "étape {i} : {a:?} illégal pour le siège {}",
                        seat.0
                    ));
                }
                s.answer(seat, a);
            }
        }
        for (seat, a) in saved.answers {
            if !s.engine_legal(seat).iter().any(|(ours, _)| *ours == a) {
                return Err(format!(
                    "réponse en attente {a:?} illégale pour le siège {}",
                    seat.0
                ));
            }
            s.answers.insert(seat, a);
        }
        Ok(s)
    }
}

impl Serialize for State {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Saved {
            seed: self.seed,
            wall: self.wall.clone(),
            steps: self.history.to_vec(),
            answers: self.answers.iter().map(|(s, a)| (*s, a.clone())).collect(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        State::replay(Saved::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
