use ludo_core::Viewer;
use serde::Serialize;

use crate::State;

/// What a seat sees: everything but the seed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: u8,
}

impl View {
    pub fn of(s: &State, _viewer: Viewer) -> Self {
        Self { players: s.players }
    }
}
