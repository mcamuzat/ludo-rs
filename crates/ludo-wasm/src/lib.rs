//! WASM bridge. Zero game logic: every function parses JSON, calls the
//! registry, and returns JSON. Matches live in WASM memory behind handles.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use ludo_core::ai::{Level, Search};
use ludo_core::{AnyMatch, LudoError, Outcome, Seat, Viewer};
use serde_json::Value;
use wasm_bindgen::prelude::*;

thread_local! {
    static MATCHES: RefCell<HashMap<u32, Box<dyn AnyMatch>>> = RefCell::new(HashMap::new());
    static NEXT_HANDLE: Cell<u32> = const { Cell::new(1) };
}

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
}

fn err(e: LudoError) -> JsError {
    JsError::new(&e.to_string())
}

fn parse(json: &str) -> Result<Value, JsError> {
    serde_json::from_str(json).map_err(|e| JsError::new(&format!("JSON invalide : {e}")))
}

fn text(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("engine types serialize to JSON")
}

fn store(m: Box<dyn AnyMatch>) -> u32 {
    let handle = NEXT_HANDLE.get();
    NEXT_HANDLE.set(handle + 1);
    MATCHES.with_borrow_mut(|all| all.insert(handle, m));
    handle
}

fn with_match<R>(
    handle: u32,
    f: impl FnOnce(&mut dyn AnyMatch) -> Result<R, JsError>,
) -> Result<R, JsError> {
    MATCHES.with_borrow_mut(|all| match all.get_mut(&handle) {
        Some(m) => f(m.as_mut()),
        None => Err(JsError::new("Partie inconnue.")),
    })
}

#[wasm_bindgen(js_name = listGames)]
pub fn list_games() -> String {
    text(
        &ludo_registry::GAMES
            .iter()
            .map(|g| g.meta())
            .collect::<Vec<_>>(),
    )
}

#[wasm_bindgen(js_name = newMatch)]
pub fn new_match(game_id: &str, config: &str, players: u8, seed: u32) -> Result<u32, JsError> {
    let game = ludo_registry::find(game_id).map_err(err)?;
    Ok(store(
        game.new_match(parse(config)?, players, seed.into())
            .map_err(err)?,
    ))
}

#[wasm_bindgen(js_name = importLog)]
pub fn import_log(game_id: &str, log: &str) -> Result<u32, JsError> {
    let game = ludo_registry::find(game_id).map_err(err)?;
    Ok(store(game.import_log(parse(log)?).map_err(err)?))
}

#[wasm_bindgen(js_name = dropMatch)]
pub fn drop_match(handle: u32) {
    MATCHES.with_borrow_mut(|all| all.remove(&handle));
}

#[wasm_bindgen]
pub fn pending(handle: u32) -> Result<Vec<u8>, JsError> {
    with_match(handle, |m| {
        Ok(m.pending().into_iter().map(|s| s.0).collect())
    })
}

#[wasm_bindgen]
pub fn rev(handle: u32) -> Result<u32, JsError> {
    with_match(handle, |m| Ok(m.rev()))
}

#[wasm_bindgen(js_name = legalActions)]
pub fn legal_actions(handle: u32, seat: u8) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.legal_actions_json(Seat(seat)))))
}

#[wasm_bindgen]
pub fn apply(handle: u32, seat: u8, action: &str) -> Result<(), JsError> {
    let action = parse(action)?;
    with_match(handle, |m| m.apply_json(Seat(seat), action).map_err(err))
}

#[wasm_bindgen]
pub fn view(handle: u32, seat: Option<u8>) -> Result<String, JsError> {
    let viewer = seat.map_or(Viewer::Spectator, |s| Viewer::Seat(Seat(s)));
    with_match(handle, |m| Ok(text(&m.view_json(viewer))))
}

#[wasm_bindgen]
pub fn outcome(handle: u32) -> Result<String, JsError> {
    with_match(handle, |m| {
        Ok(text(&m.outcome().map(|Outcome::Ranked(ranks)| ranks)))
    })
}

#[wasm_bindgen(js_name = exportState)]
pub fn export_state(handle: u32) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.state_json())))
}

#[wasm_bindgen(js_name = exportLog)]
pub fn export_log(handle: u32) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.log_json())))
}

/// The AI's action for `seat` in `state` (from `exportState`), within the
/// level's time budget.
#[wasm_bindgen(js_name = aiMove)]
pub fn ai_move(
    game_id: &str,
    state: &str,
    seat: u8,
    level: &str,
    seed: u32,
) -> Result<String, JsError> {
    let level: Level = serde_json::from_value(Value::from(level))
        .map_err(|_| JsError::new(&format!("Niveau inconnu : {level}.")))?;
    let game = ludo_registry::find(game_id).map_err(err)?;
    let search = Search::timed(level, seed.into());
    let action = game
        .ai_move(parse(state)?, Seat(seat), &search, &js_sys::Date::now)
        .map_err(err)?;
    Ok(text(&action))
}
