use serde::{Deserialize, Serialize};

use crate::{Game, Illegal, Seat};

/// Everything needed to rebuild a match: the engine is deterministic, so the
/// setup plus the accepted actions, in order, determine the state.
#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct MatchLog<G: Game> {
    pub game_id: String,
    pub config: G::Config,
    pub players: u8,
    pub seed: u64,
    pub actions: Vec<(Seat, G::Action)>,
}

impl<G: Game> Clone for MatchLog<G> {
    fn clone(&self) -> Self {
        Self {
            game_id: self.game_id.clone(),
            config: self.config.clone(),
            players: self.players,
            seed: self.seed,
            actions: self.actions.clone(),
        }
    }
}

/// A match in progress: its state and the log that produced it.
pub struct Match<G: Game> {
    state: G::State,
    log: MatchLog<G>,
}

impl<G: Game> Match<G> {
    pub fn new(config: G::Config, players: u8, seed: u64) -> Result<Self, Illegal> {
        let meta = G::meta();
        if !(meta.min_players..=meta.max_players).contains(&players) {
            return Err(Illegal::new(format!(
                "{} se joue de {} à {} joueurs.",
                meta.name, meta.min_players, meta.max_players
            )));
        }
        G::check_config(&config, players)?;
        let state = G::new(&config, players, seed);
        let log = MatchLog {
            game_id: meta.id.to_owned(),
            config,
            players,
            seed,
            actions: Vec::new(),
        };
        Ok(Self { state, log })
    }

    pub fn from_log(log: MatchLog<G>) -> Result<Self, Illegal> {
        if log.game_id != G::meta().id {
            return Err(Illegal::new(format!(
                "Journal d'un autre jeu : {}.",
                log.game_id
            )));
        }
        let MatchLog {
            config,
            players,
            seed,
            actions,
            ..
        } = log;
        let mut m = Self::new(config, players, seed)?;
        for (i, (seat, action)) in actions.into_iter().enumerate() {
            m.apply(seat, action)
                .map_err(|e| Illegal::new(format!("action {i} : {e}")))?;
        }
        Ok(m)
    }

    /// Applies and logs `action`; a refused action changes nothing.
    pub fn apply(&mut self, seat: Seat, action: G::Action) -> Result<(), Illegal> {
        G::apply(&mut self.state, seat, &action)?;
        self.log.actions.push((seat, action));
        Ok(())
    }

    pub fn state(&self) -> &G::State {
        &self.state
    }

    pub fn log(&self) -> &MatchLog<G> {
        &self.log
    }

    /// Number of accepted actions: identifies a state for staleness checks.
    pub fn rev(&self) -> u32 {
        self.log.actions.len() as u32
    }
}

/// Rebuilds the state described by `log`, refusing it at the first illegal action.
pub fn replay<G: Game>(log: &MatchLog<G>) -> Result<G::State, Illegal> {
    Match::from_log(log.clone()).map(|m| m.state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testgames::{Nim, NimConfig};

    fn played() -> Match<Nim> {
        let mut m = Match::<Nim>::new(NimConfig { pile: 5 }, 2, 42).unwrap();
        m.apply(Seat(0), 2).unwrap();
        m.apply(Seat(1), 3).unwrap();
        m
    }

    #[test]
    fn replay_rebuilds_the_same_state() {
        let m = played();
        assert_eq!(m.rev(), 2);
        assert_eq!(replay::<Nim>(m.log()).unwrap(), *m.state());
    }

    #[test]
    fn a_refused_action_is_not_logged() {
        let mut m = played();
        assert!(m.apply(Seat(0), 1).is_err());
        assert_eq!(m.rev(), 2);
    }

    #[test]
    fn a_tampered_log_is_refused() {
        let mut log = played().log().clone();
        log.actions.push((Seat(0), 1));
        let err = replay::<Nim>(&log).unwrap_err();
        assert!(err.0.starts_with("action 2"), "{}", err.0);
    }

    #[test]
    fn a_log_from_another_game_is_refused() {
        let mut log = played().log().clone();
        log.game_id = "chess".into();
        assert!(replay::<Nim>(&log).is_err());
    }

    #[test]
    fn check_config_guards_new_matches() {
        assert!(Match::<Nim>::new(NimConfig { pile: 30 }, 2, 1).is_ok());
        for pile in [0, 31] {
            let e = Match::<Nim>::new(NimConfig { pile }, 2, 1)
                .err()
                .expect("refused");
            assert_eq!(e.0, "Nim se joue avec un tas de 1 à 30.");
        }
    }

    #[test]
    fn player_count_is_checked() {
        assert!(Match::<Nim>::new(NimConfig::default(), 1, 0).is_err());
        assert!(Match::<Nim>::new(NimConfig::default(), 5, 0).is_err());
    }

    #[test]
    fn the_log_round_trips_through_json() {
        let m = played();
        let json = serde_json::to_string(m.log()).unwrap();
        let back: MatchLog<Nim> = serde_json::from_str(&json).unwrap();
        assert_eq!(replay::<Nim>(&back).unwrap(), *m.state());
    }
}
