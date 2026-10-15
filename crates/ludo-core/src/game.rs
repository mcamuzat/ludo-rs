use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;

use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::ai::{Budget, Level};

/// A place at the table, numbered from 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Seat(pub u8);

impl Seat {
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// Who a view is computed for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Viewer {
    Seat(Seat),
    Spectator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub min_players: u8,
    pub max_players: u8,
    pub hidden_info: bool,
    /// The game provides `timeout_action`: online play may use a clock.
    pub clock: bool,
}

/// A setting the lobby offers for a game. `key` is the name of a field of the
/// game's `Config`; `kind` gives the values it takes and its default, which
/// must be the field's value in `Config::default()` (checked by conformance).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GameOption {
    pub key: &'static str,
    pub kind: OptionKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OptionKind {
    Toggle {
        default: bool,
    },
    /// `values` are the serialized names of an enum's variants.
    Choice {
        values: &'static [&'static str],
        default: &'static str,
    },
    Number {
        min: u32,
        max: u32,
        default: u32,
    },
}

/// Seats that must decide now. Several seats when the game waits for
/// simultaneous answers; empty exactly when the game is over.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pending(pub Vec<Seat>);

impl Pending {
    pub fn none() -> Self {
        Self(Vec::new())
    }

    pub fn one(seat: Seat) -> Self {
        Self(vec![seat])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, seat: Seat) -> bool {
        self.0.contains(&seat)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Rank of each seat (0 = first); equal ranks are ties.
    Ranked(Vec<u8>),
}

impl Outcome {
    /// Each seat's reward in [0, 1]: the share of other seats it finished
    /// ahead of, a tie counting as half.
    pub fn rewards(&self) -> Vec<f64> {
        let Outcome::Ranked(ranks) = self;
        let n = ranks.len();
        if n <= 1 {
            return vec![1.0; n];
        }
        ranks
            .iter()
            .enumerate()
            .map(|(i, &mine)| {
                let beaten: f64 = ranks
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, &theirs)| match mine.cmp(&theirs) {
                        Ordering::Less => 1.0,
                        Ordering::Equal => 0.5,
                        Ordering::Greater => 0.0,
                    })
                    .sum();
                beaten / (n - 1) as f64
            })
            .collect()
    }
}

/// A refused action. The message is shown to the player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Illegal(pub String);

impl Illegal {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Illegal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Illegal {}

/// A game as a pure reducer: no I/O, no clock; randomness comes from the
/// seed given to `new` and kept in the state.
pub trait Game: 'static {
    type State: Clone + fmt::Debug + Serialize + DeserializeOwned;
    type Action: Clone + fmt::Debug + Eq + Hash + Serialize + DeserializeOwned;
    /// What one seat (or a spectator) may see. Never contains the seed nor
    /// another seat's hidden data.
    type View: Serialize;
    type Config: Clone + fmt::Debug + Default + Serialize + DeserializeOwned;

    fn meta() -> GameMeta;
    /// `players` is within `meta()`'s bounds (checked by `Match::new`).
    fn new(config: &Self::Config, players: u8, seed: u64) -> Self::State;
    fn pending(s: &Self::State) -> Pending;
    /// Empty for a seat that is not pending.
    fn legal_actions(s: &Self::State, seat: Seat) -> Vec<Self::Action>;
    /// On error the state is left unchanged.
    fn apply(s: &mut Self::State, seat: Seat, action: &Self::Action) -> Result<(), Illegal>;
    fn view(s: &Self::State, viewer: Viewer) -> Self::View;
    /// `Some` exactly when `pending` is empty.
    fn outcome(s: &Self::State) -> Option<Outcome>;

    /// What the host plays for `seat` when its time runs out. `None` when the
    /// game has no clock or `seat` has nothing to decide.
    fn timeout_action(_s: &Self::State, _seat: Seat) -> Option<Self::Action> {
        None
    }

    /// The pending seats `viewer` may know about (a subset of `pending`
    /// that always contains `viewer` when it is pending). Games where being
    /// asked is itself information (who can call a discard) narrow it.
    fn visible_pending(s: &Self::State, _viewer: Viewer) -> Pending {
        Self::pending(s)
    }

    /// The settings the lobby offers. Default: none.
    fn options() -> Vec<GameOption> {
        Vec::new()
    }

    /// Refuses a config the game cannot be played with (a value out of the
    /// bounds `options` announces). Called by `Match::new`, so by every new
    /// match and every replayed log.
    fn check_config(_config: &Self::Config, _players: u8) -> Result<(), Illegal> {
        Ok(())
    }
}

pub trait Determinize: Game {
    /// A copy of `s` where everything `observer` cannot see (other hands,
    /// deck order, other seats' unrevealed simultaneous answers) is redrawn
    /// consistently with what `observer` knows. Identity for a
    /// perfect-information game.
    fn determinize<R: Rng + ?Sized>(s: &Self::State, observer: Seat, rng: &mut R) -> Self::State;
}

pub trait Heuristics: Game {
    /// A legal action for `seat` (which is pending), better than uniform random.
    fn rollout_action<R: Rng + ?Sized>(s: &Self::State, seat: Seat, rng: &mut R) -> Self::Action;

    /// Per-seat estimate in [0, 1] that ends a simulation early. Asked at every
    /// simulated step, with the state the search started from.
    fn evaluate(_root: &Self::State, _sim: &Self::State) -> Option<Vec<f64>> {
        None
    }

    /// Search budget for `level` (`timed`: in the browser, else reproducible
    /// iterations). `None`: play the rollout policy alone, without search.
    fn budget(level: Level, timed: bool) -> Option<Budget> {
        Some(level.default_budget(timed))
    }

    /// The actions the search considers for `seat`: a non-empty subset of
    /// `legal_actions`, depending only on what `seat` can see. Games narrow it
    /// to keep a wide search on sensible moves; default: every legal action.
    fn search_actions(s: &Self::State, seat: Seat) -> Vec<Self::Action> {
        Self::legal_actions(s, seat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewards_count_players_beaten_and_half_ties() {
        assert_eq!(Outcome::Ranked(vec![0, 1]).rewards(), vec![1.0, 0.0]);
        assert_eq!(Outcome::Ranked(vec![0, 0]).rewards(), vec![0.5, 0.5]);
        assert_eq!(
            Outcome::Ranked(vec![0, 1, 1]).rewards(),
            vec![1.0, 0.25, 0.25]
        );
        assert_eq!(
            Outcome::Ranked(vec![1, 0, 2]).rewards(),
            vec![0.5, 1.0, 0.0]
        );
    }

    #[test]
    fn pending_helpers() {
        assert!(Pending::none().is_empty());
        assert!(Pending::one(Seat(2)).contains(Seat(2)));
        assert!(!Pending::one(Seat(2)).contains(Seat(1)));
    }
}
