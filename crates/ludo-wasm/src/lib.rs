//! WASM bridge. Zero game logic: every function parses JSON, calls the
//! registry, and returns JSON. Matches live in WASM memory behind handles.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use ludo_core::ai::Level;
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
            .map(|g| g.info())
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

/// Rewinds to just before `seat`'s last action.
#[wasm_bindgen(js_name = takeBack)]
pub fn take_back(handle: u32, seat: u8) -> Result<(), JsError> {
    with_match(handle, |m| m.take_back(Seat(seat)).map_err(err))
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
    let action = game
        .ai_move(
            parse(state)?,
            Seat(seat),
            level,
            true,
            seed.into(),
            &js_sys::Date::now,
        )
        .map_err(err)?;
    Ok(text(&action))
}

#[wasm_bindgen(js_name = visiblePending)]
pub fn visible_pending(handle: u32, seat: Option<u8>) -> Result<Vec<u8>, JsError> {
    let viewer = seat.map_or(Viewer::Spectator, |s| Viewer::Seat(Seat(s)));
    with_match(handle, |m| {
        Ok(m.visible_pending(viewer).into_iter().map(|s| s.0).collect())
    })
}

/// The action the host plays when `seat`'s time runs out (JSON, `null` if none).
#[wasm_bindgen(js_name = timeoutAction)]
pub fn timeout_action(handle: u32, seat: u8) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.timeout_action_json(Seat(seat)))))
}

/// A Cryptide riddle (`cryptide::riddle`): a table of AIs stopped as soon
/// as one of them knows the habitat.
#[wasm_bindgen(js_name = cryptideRiddle)]
pub fn cryptide_riddle(mode: &str, players: u8, seed: u32) -> Result<String, JsError> {
    let mode: cryptide::Mode = serde_json::from_value(Value::from(mode))
        .map_err(|_| JsError::new(&format!("Mode inconnu : {mode}.")))?;
    if !(3..=5).contains(&players) {
        return Err(JsError::new("Une énigme se joue à 3, 4 ou 5."));
    }
    let riddle = cryptide::riddle(mode, players, seed.into())
        .ok_or_else(|| JsError::new("Aucune énigme trouvée pour cette graine."))?;
    Ok(text(&riddle))
}

// TwixT's network and coach (games/twixt). The weights are fetched by the
// worker and installed once; every coach function takes the game's moves.

fn twixt_moves(moves: &str) -> Result<Vec<twixt::TwixtAction>, JsError> {
    serde_json::from_str(moves).map_err(|e| JsError::new(&format!("Coups invalides : {e}")))
}

fn twixt_net() -> Result<&'static twixt::net::Net, JsError> {
    twixt::net::net().ok_or_else(|| JsError::new("Poids de TwixT non chargés."))
}

fn illegal(e: ludo_core::Illegal) -> JsError {
    JsError::new(&e.to_string())
}

#[wasm_bindgen(js_name = twixtInstallNet)]
pub fn twixt_install_net(bytes: &[u8]) -> Result<(), JsError> {
    twixt::net::install(bytes).map_err(|e| JsError::new(&format!("Poids de TwixT invalides : {e}")))
}

#[wasm_bindgen(js_name = twixtNetReady)]
pub fn twixt_net_ready() -> bool {
    twixt::net::net().is_some()
}

/// The coach's advice for the side to move after `moves` (JSON).
#[wasm_bindgen(js_name = twixtAdvise)]
pub fn twixt_advise(moves: &str) -> Result<String, JsError> {
    let advice = twixt::coach::advise(twixt_net()?, &twixt_moves(moves)?).map_err(illegal)?;
    Ok(text(&advice))
}

/// The review's numbers for the position after the first `ply` moves (JSON).
#[wasm_bindgen(js_name = twixtReviewPoint)]
pub fn twixt_review_point(moves: &str, ply: u32) -> Result<String, JsError> {
    let point = twixt::coach::review_point(twixt_net()?, &twixt_moves(moves)?, ply as usize)
        .map_err(illegal)?;
    Ok(text(&point))
}

/// The review of a game from its `twixtReviewPoint` values (JSON).
#[wasm_bindgen(js_name = twixtReview)]
pub fn twixt_review(moves: &str, points: &str) -> Result<String, JsError> {
    let points: Vec<twixt::coach::ReviewPoint> = serde_json::from_str(points)
        .map_err(|e| JsError::new(&format!("Courbe invalide : {e}")))?;
    let review = twixt::coach::review(&twixt_moves(moves)?, &points).map_err(illegal)?;
    Ok(text(&review))
}

/// The view of the position after `moves` (for the review's board).
#[wasm_bindgen(js_name = twixtPosition)]
pub fn twixt_position(moves: &str) -> Result<String, JsError> {
    let s = twixt::TwixtState::from_moves(&twixt_moves(moves)?).map_err(illegal)?;
    Ok(text(&twixt::TwixtView::of(&s)))
}
