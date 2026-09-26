//! Japanese Riichi Mahjong, Mahjong Soul rules, on top of riichienv-core.
//! Rules and what ludo adds: RULES.md.

mod action;
mod scoring;
mod state;
pub mod testing;
mod tile;
mod view;

pub use action::Action;
pub use state::State;
pub use tile::Tile;
pub use view::{
    DiscardView, DrawKind, HandResult, Help, MeldKind, MeldView, PlayerView, RoundView, View,
    WinView, Wind, YakuName,
};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

pub struct Riichi;

/// Hanchan with Mahjong Soul rules: no options.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

impl Game for Riichi {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "riichi",
            name: "Mahjong Riichi",
            min_players: 4,
            max_players: 4,
            hidden_info: true,
            clock: true,
        }
    }

    fn new(_config: &Config, _players: u8, seed: u64) -> State {
        State::start(seed, None)
    }

    fn pending(s: &State) -> Pending {
        Pending(s.pending_seats())
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.engine_legal(seat).into_iter().map(|(a, _)| a).collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        let legal = s.engine_legal(seat);
        if legal.is_empty() {
            return Err(Illegal::new("Ce n'est pas à toi de jouer."));
        }
        if !legal.iter().any(|(a, _)| a == action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        s.answer(seat, action.clone());
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// Final ranking by score; ties go to the earlier seat (Mahjong Soul).
    fn outcome(s: &State) -> Option<Outcome> {
        if !s.inner.is_done {
            return None;
        }
        let scores = s.scores();
        let mut order: Vec<usize> = (0..4).collect();
        order.sort_by_key(|&i| (std::cmp::Reverse(scores[i]), i));
        let mut ranks = vec![0u8; 4];
        for (rank, &seat) in order.iter().enumerate() {
            ranks[seat] = rank as u8;
        }
        Some(Outcome::Ranked(ranks))
    }

    /// Pass a call; otherwise discard the drawn tile; otherwise the first
    /// legal discard; otherwise the first legal action.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return None;
        }
        if legal.contains(&Action::Pass) {
            return Some(Action::Pass);
        }
        if let Some(drawn) = s.inner.drawn_tile.map(Tile::from_id) {
            let tsumogiri = Action::Discard { tile: drawn };
            if legal.contains(&tsumogiri) {
                return Some(tsumogiri);
            }
        }
        legal
            .iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .or(legal.first())
            .cloned()
    }

    /// During a call window, being asked is secret: each seat only sees itself.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        if !s.in_claim_window() {
            return Self::pending(s);
        }
        match viewer {
            Viewer::Seat(me) if s.is_pending(me) => Pending::one(me),
            _ => Pending::none(),
        }
    }
}
