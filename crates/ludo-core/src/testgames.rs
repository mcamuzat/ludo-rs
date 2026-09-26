//! Tiny games that exercise the platform in tests.

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{Determinize, Game, GameMeta, Heuristics, Illegal, Outcome, Pending, Seat, Viewer};

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

#[cfg(test)]
mod nim_tests {
    use super::*;
    use crate::{Game, Outcome, Seat};

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
