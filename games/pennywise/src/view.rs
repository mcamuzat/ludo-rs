use ludo_core::Seat;
use serde::Serialize;

use crate::{Character, LastMove, Purse, Stack, State};

/// Everything is public in Pennywise: every seat and spectator sees the same.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub character: Character,
    pub purses: Vec<Vec<Stack>>,
    pub totals: Vec<u32>,
    pub pot: Vec<Stack>,
    pub pot_total: u32,
    /// `None` once the game is over.
    pub current: Option<Seat>,
    pub last: Option<LastMove>,
    pub ruined: Option<Seat>,
    /// Final ranks (0 = first), once the game is over.
    pub ranks: Option<Vec<u8>>,
}

impl View {
    pub fn of(s: &State) -> Self {
        let over = s.ruined.is_some();
        View {
            character: s.config.character,
            purses: s.purses.iter().map(Purse::stacks).collect(),
            totals: s.totals(),
            pot: s.pot.stacks(),
            pot_total: s.pot.total(),
            current: (!over).then_some(s.current),
            last: s.last.clone(),
            ruined: s.ruined,
            ranks: over.then(|| s.ranks()),
        }
    }
}
