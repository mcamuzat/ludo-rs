//! Type erasure: every game seen through JSON, for the WASM bridge and the CLI.

use std::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::ai::{self, Search};
use crate::{
    Determinize, Game, GameMeta, Heuristics, Illegal, Match, MatchLog, Outcome, Seat, Viewer,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LudoError {
    UnknownGame(String),
    BadJson(String),
    Illegal(String),
}

impl fmt::Display for LudoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LudoError::UnknownGame(id) => write!(f, "Jeu inconnu : {id}."),
            LudoError::BadJson(message) => write!(f, "Données invalides : {message}."),
            LudoError::Illegal(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for LudoError {}

impl From<Illegal> for LudoError {
    fn from(e: Illegal) -> Self {
        LudoError::Illegal(e.0)
    }
}

fn from_json<T: DeserializeOwned>(value: Value) -> Result<T, LudoError> {
    serde_json::from_value(value).map_err(|e| LudoError::BadJson(e.to_string()))
}

fn to_json<T: Serialize + ?Sized>(value: &T) -> Value {
    serde_json::to_value(value).expect("engine types serialize to JSON")
}

pub trait AnyMatch {
    fn game_id(&self) -> &'static str;
    fn rev(&self) -> u32;
    fn pending(&self) -> Vec<Seat>;
    fn legal_actions_json(&self, seat: Seat) -> Value;
    fn apply_json(&mut self, seat: Seat, action: Value) -> Result<(), LudoError>;
    fn view_json(&self, viewer: Viewer) -> Value;
    fn outcome(&self) -> Option<Outcome>;
    fn state_json(&self) -> Value;
    fn log_json(&self) -> Value;
}

impl<G: Game> AnyMatch for Match<G> {
    fn game_id(&self) -> &'static str {
        G::meta().id
    }

    fn rev(&self) -> u32 {
        Match::rev(self)
    }

    fn pending(&self) -> Vec<Seat> {
        G::pending(self.state()).0
    }

    fn legal_actions_json(&self, seat: Seat) -> Value {
        to_json(&G::legal_actions(self.state(), seat))
    }

    fn apply_json(&mut self, seat: Seat, action: Value) -> Result<(), LudoError> {
        let action: G::Action = from_json(action)?;
        Ok(self.apply(seat, action)?)
    }

    fn view_json(&self, viewer: Viewer) -> Value {
        to_json(&G::view(self.state(), viewer))
    }

    fn outcome(&self) -> Option<Outcome> {
        G::outcome(self.state())
    }

    fn state_json(&self) -> Value {
        to_json(self.state())
    }

    fn log_json(&self) -> Value {
        to_json(self.log())
    }
}

pub trait AnyGame: Sync {
    fn meta(&self) -> GameMeta;
    /// `config` = `Value::Null` for the game's default options.
    fn new_match(
        &self,
        config: Value,
        players: u8,
        seed: u64,
    ) -> Result<Box<dyn AnyMatch>, LudoError>;
    fn import_log(&self, log: Value) -> Result<Box<dyn AnyMatch>, LudoError>;
    fn ai_move(
        &self,
        state: Value,
        seat: Seat,
        search: &Search,
        now_ms: &dyn Fn() -> f64,
    ) -> Result<Value, LudoError>;
}

/// Registers game `G`: `static G: GameEntry<MyGame> = GameEntry::new();`.
pub struct GameEntry<G>(PhantomData<fn() -> G>);

impl<G> GameEntry<G> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<G> Default for GameEntry<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Game + Determinize + Heuristics> AnyGame for GameEntry<G> {
    fn meta(&self) -> GameMeta {
        G::meta()
    }

    fn new_match(
        &self,
        config: Value,
        players: u8,
        seed: u64,
    ) -> Result<Box<dyn AnyMatch>, LudoError> {
        let config = if config.is_null() {
            G::Config::default()
        } else {
            from_json(config)?
        };
        Ok(Box::new(Match::<G>::new(config, players, seed)?))
    }

    fn import_log(&self, log: Value) -> Result<Box<dyn AnyMatch>, LudoError> {
        let log: MatchLog<G> = from_json(log)?;
        Ok(Box::new(Match::from_log(log)?))
    }

    fn ai_move(
        &self,
        state: Value,
        seat: Seat,
        search: &Search,
        now_ms: &dyn Fn() -> f64,
    ) -> Result<Value, LudoError> {
        let state: G::State = from_json(state)?;
        Ok(to_json(&ai::choose::<G>(&state, seat, search, now_ms)?))
    }
}

pub fn find<'a>(games: &[&'a dyn AnyGame], id: &str) -> Result<&'a dyn AnyGame, LudoError> {
    games
        .iter()
        .copied()
        .find(|g| g.meta().id == id)
        .ok_or_else(|| LudoError::UnknownGame(id.to_owned()))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ai::Level;
    use crate::testgames::Nim;

    static NIM: GameEntry<Nim> = GameEntry::new();
    static GAMES: &[&dyn AnyGame] = &[&NIM];

    #[test]
    fn plays_through_json() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(json!({ "pile": 5 }), 2, 1).unwrap();
        assert_eq!(m.legal_actions_json(Seat(0)), json!([1, 2, 3]));
        m.apply_json(Seat(0), json!(2)).unwrap();
        assert_eq!(m.rev(), 1);
        assert_eq!(m.pending(), vec![Seat(1)]);
        assert_eq!(m.view_json(Viewer::Spectator)["pile"], json!(3));
    }

    #[test]
    fn null_config_means_default() {
        let m = find(GAMES, "nim")
            .unwrap()
            .new_match(Value::Null, 2, 1)
            .unwrap();
        assert_eq!(m.state_json()["pile"], json!(10));
    }

    #[test]
    fn errors_are_typed() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(Value::Null, 2, 1).unwrap();
        assert!(matches!(
            m.apply_json(Seat(0), json!("x")),
            Err(LudoError::BadJson(_))
        ));
        assert!(matches!(
            m.apply_json(Seat(0), json!(9)),
            Err(LudoError::Illegal(_))
        ));
        assert!(matches!(
            find(GAMES, "chess"),
            Err(LudoError::UnknownGame(_))
        ));
        assert!(matches!(
            game.new_match(Value::Null, 9, 1),
            Err(LudoError::Illegal(_))
        ));
    }

    #[test]
    fn log_import_resumes_the_match() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(Value::Null, 2, 1).unwrap();
        m.apply_json(Seat(0), json!(3)).unwrap();
        let resumed = game.import_log(m.log_json()).unwrap();
        assert_eq!(resumed.state_json(), m.state_json());
        assert_eq!(resumed.rev(), 1);
    }

    #[test]
    fn ai_move_is_applicable() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(Value::Null, 2, 1).unwrap();
        let action = game
            .ai_move(
                m.state_json(),
                Seat(0),
                &Search::counted(Level::Easy, 5),
                &|| 0.0,
            )
            .unwrap();
        m.apply_json(Seat(0), action).unwrap();
    }
}
