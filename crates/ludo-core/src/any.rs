//! Type erasure: every game seen through JSON, for the WASM bridge and the CLI.

use std::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::ai::{self, Level, Search};
use crate::{
    Determinize, Game, GameMeta, GameOption, Heuristics, Illegal, Match, MatchLog, Outcome, Seat,
    Viewer,
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
    fn visible_pending(&self, viewer: Viewer) -> Vec<Seat>;
    fn timeout_action_json(&self, seat: Seat) -> Option<Value>;
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

    fn visible_pending(&self, viewer: Viewer) -> Vec<Seat> {
        G::visible_pending(self.state(), viewer).0
    }

    fn timeout_action_json(&self, seat: Seat) -> Option<Value> {
        G::timeout_action(self.state(), seat).map(|a| to_json(&a))
    }
}

/// Reads a config, refusing a field the game does not know: serde would
/// silently drop it, and a mistyped option would quietly play the default.
fn known_config<G: Game>(given: Value) -> Result<G::Config, LudoError> {
    let config: G::Config = from_json(given.clone())?;
    if let (Value::Object(given), Value::Object(kept)) = (&given, to_json(&config)) {
        if let Some(key) = given.keys().find(|k| !kept.contains_key(*k)) {
            return Err(LudoError::Illegal(format!("Option inconnue : {key}.")));
        }
    }
    Ok(config)
}

/// What the lobby needs to know of a game: its metadata and its options.
#[derive(Clone, Debug, Serialize)]
pub struct GameInfo {
    #[serde(flatten)]
    pub meta: GameMeta,
    pub options: Vec<GameOption>,
}

pub trait AnyGame: Sync {
    fn meta(&self) -> GameMeta;
    fn options(&self) -> Vec<GameOption>;

    fn info(&self) -> GameInfo {
        GameInfo {
            meta: self.meta(),
            options: self.options(),
        }
    }
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
        level: Level,
        timed: bool,
        seed: u64,
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

    fn options(&self) -> Vec<GameOption> {
        G::options()
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
            known_config::<G>(config)?
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
        level: Level,
        timed: bool,
        seed: u64,
        now_ms: &dyn Fn() -> f64,
    ) -> Result<Value, LudoError> {
        let state: G::State = from_json(state)?;
        let search = Search::for_game::<G>(level, timed, seed);
        Ok(to_json(&ai::choose::<G>(&state, seat, &search, now_ms)?))
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
    fn info_lists_meta_and_options() {
        let info = serde_json::to_value(find(GAMES, "nim").unwrap().info()).unwrap();
        assert_eq!(info["id"], json!("nim"));
        assert_eq!(info["minPlayers"], json!(2));
        assert_eq!(
            info["options"],
            json!([{ "key": "pile", "kind": { "type": "number", "min": 1, "max": 30, "default": 10 } }])
        );
    }

    #[test]
    fn new_match_refuses_a_config_out_of_bounds() {
        let game = find(GAMES, "nim").unwrap();
        assert!(matches!(
            game.new_match(json!({ "pile": 0 }), 2, 1),
            Err(LudoError::Illegal(_))
        ));
    }

    #[test]
    fn new_match_refuses_an_unknown_option() {
        let game = find(GAMES, "nim").unwrap();
        match game.new_match(json!({ "pile": 5, "piles": 3 }), 2, 1) {
            Err(LudoError::Illegal(message)) => assert_eq!(message, "Option inconnue : piles."),
            other => panic!(
                "expected a refusal, got {:?}",
                other.map(|m| m.state_json())
            ),
        }
        assert!(game.new_match(json!({ "pile": 5 }), 2, 1).is_ok());
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
            .ai_move(m.state_json(), Seat(0), Level::Easy, false, 5, &|| 0.0)
            .unwrap();
        m.apply_json(Seat(0), action).unwrap();
    }

    #[test]
    fn timeout_and_visible_pending_through_json() {
        let m = find(GAMES, "nim")
            .unwrap()
            .new_match(Value::Null, 2, 1)
            .unwrap();
        assert_eq!(m.timeout_action_json(Seat(0)), Some(json!(1)));
        assert_eq!(m.timeout_action_json(Seat(1)), None);
        assert_eq!(m.visible_pending(Viewer::Seat(Seat(1))), vec![Seat(0)]);
    }
}
