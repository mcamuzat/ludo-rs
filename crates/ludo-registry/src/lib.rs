//! Every game of the platform. Adding a game: one `static` and one entry in `GAMES`.

use ludo_core::{AnyGame, GameEntry, LudoError};

static YAVALATH: GameEntry<yavalath::Yavalath> = GameEntry::new();
static RIICHI: GameEntry<riichi::Riichi> = GameEntry::new();
static PAIRS_CONTINUOUS: GameEntry<pairs::ContinuousPairs> = GameEntry::new();
static PAIRS_AUBERGE: GameEntry<pairs::Auberge> = GameEntry::new();
static PAIRS_GHOST_TOWN: GameEntry<pairs::GhostTown> = GameEntry::new();
static PAIRS_ZOMBIE_LIFE: GameEntry<pairs::ZombieLife> = GameEntry::new();
static PAIRS_GOBLIN_POKER: GameEntry<pairs::GoblinPoker> = GameEntry::new();
static PAIRS_TO_THE_MOON: GameEntry<pairs::ToTheMoon> = GameEntry::new();
static PAIRS_DEADFALL: GameEntry<pairs::Deadfall> = GameEntry::new();
static PAIRS_CAPTAINS_ORDERS: GameEntry<pairs::CaptainsOrders> = GameEntry::new();

pub static GAMES: &[&dyn AnyGame] = &[
    &YAVALATH,
    &RIICHI,
    &PAIRS_CONTINUOUS,
    &PAIRS_AUBERGE,
    &PAIRS_GHOST_TOWN,
    &PAIRS_ZOMBIE_LIFE,
    &PAIRS_GOBLIN_POKER,
    &PAIRS_TO_THE_MOON,
    &PAIRS_DEADFALL,
    &PAIRS_CAPTAINS_ORDERS,
];

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
