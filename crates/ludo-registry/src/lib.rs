//! Every game of the platform. Adding a game: one `static` and one entry in `GAMES`.

use ludo_core::{AnyGame, GameEntry, LudoError};

static YAVALATH: GameEntry<yavalath::Yavalath> = GameEntry::new();

pub static GAMES: &[&dyn AnyGame] = &[&YAVALATH];

pub fn find(id: &str) -> Result<&'static dyn AnyGame, LudoError> {
    ludo_core::find(GAMES, id)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use serde_json::Value;

    use super::*;

    #[test]
    fn ids_are_unique_and_every_game_starts() {
        let ids: HashSet<&str> = GAMES.iter().map(|g| g.meta().id).collect();
        assert_eq!(ids.len(), GAMES.len());
        for game in GAMES {
            let meta = game.meta();
            for players in meta.min_players..=meta.max_players {
                let m = game.new_match(Value::Null, players, 1).unwrap();
                assert!(!m.pending().is_empty(), "{} à {players}", meta.id);
            }
        }
    }
}
