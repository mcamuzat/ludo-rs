use riichienv_core::action::{Action as EngineAction, ActionType};
use serde::{Deserialize, Serialize};

use crate::Tile;

/// What a player can do, in the players' terms (tiles by notation).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    Discard {
        tile: Tile,
    },
    /// Declares riichi; the discard follows as the next action.
    Riichi,
    Chi {
        tile: Tile,
        with: [Tile; 2],
    },
    Pon {
        tile: Tile,
        with: [Tile; 2],
    },
    Daiminkan {
        tile: Tile,
    },
    Ankan {
        tile: Tile,
    },
    Kakan {
        tile: Tile,
    },
    Tsumo,
    Ron,
    KyushuKyuhai,
    Pass,
}

fn pair(ids: &[u8]) -> Option<[Tile; 2]> {
    match ids {
        [a, b] => Some([Tile::from_id(*a), Tile::from_id(*b)]),
        _ => None,
    }
}

/// Our view of an engine action; `None` for three-player actions (kita).
pub(crate) fn from_engine(a: &EngineAction) -> Option<Action> {
    let tile = || a.tile.map(Tile::from_id);
    Some(match a.action_type {
        ActionType::Discard => Action::Discard { tile: tile()? },
        ActionType::Riichi => Action::Riichi,
        ActionType::Chi => Action::Chi {
            tile: tile()?,
            with: pair(&a.consume_tiles)?,
        },
        ActionType::Pon => Action::Pon {
            tile: tile()?,
            with: pair(&a.consume_tiles)?,
        },
        ActionType::Daiminkan => Action::Daiminkan { tile: tile()? },
        ActionType::Ankan => Action::Ankan {
            tile: Tile::from_id(*a.consume_tiles.first()?),
        },
        ActionType::Kakan => Action::Kakan {
            tile: a
                .tile
                .or(a.consume_tiles.first().copied())
                .map(Tile::from_id)?,
        },
        ActionType::Tsumo => Action::Tsumo,
        ActionType::Ron => Action::Ron,
        ActionType::KyushuKyuhai => Action::KyushuKyuhai,
        ActionType::Pass => Action::Pass,
        ActionType::Kita => return None,
    })
}

/// The engine's legal actions in our terms, one per distinct `Action`
/// (copies of the same tile are the same move), each paired with the first
/// engine action that realises it.
pub(crate) fn legal(engine: &[EngineAction]) -> Vec<(Action, EngineAction)> {
    let mut out: Vec<(Action, EngineAction)> = Vec::new();
    for e in engine {
        if let Some(a) = from_engine(e) {
            if !out.iter().any(|(seen, _)| *seen == a) {
                out.push((a, e.clone()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use riichienv_core::action::{Action as EngineAction, ActionType};

    use super::*;

    fn engine(kind: ActionType, tile: Option<u8>, consume: Vec<u8>) -> EngineAction {
        EngineAction::new(kind, tile, consume, Some(0))
    }

    fn t(text: &str) -> Tile {
        Tile::parse(text).unwrap()
    }

    #[test]
    fn every_engine_action_converts() {
        let cases = [
            (
                engine(ActionType::Discard, Some(17), vec![]),
                Action::Discard { tile: t("5m") },
            ),
            (
                engine(ActionType::Discard, Some(16), vec![]),
                Action::Discard { tile: t("5mr") },
            ),
            (engine(ActionType::Riichi, None, vec![]), Action::Riichi),
            (
                engine(ActionType::Chi, Some(8), vec![0, 4]),
                Action::Chi {
                    tile: t("3m"),
                    with: [t("1m"), t("2m")],
                },
            ),
            (
                engine(ActionType::Pon, Some(108), vec![109, 110]),
                Action::Pon {
                    tile: t("E"),
                    with: [t("E"), t("E")],
                },
            ),
            (
                engine(ActionType::Daiminkan, Some(132), vec![133, 134, 135]),
                Action::Daiminkan { tile: t("C") },
            ),
            (
                engine(ActionType::Ankan, None, vec![16, 17, 18, 19]),
                Action::Ankan { tile: t("5mr") },
            ),
            (
                engine(ActionType::Kakan, Some(120), vec![]),
                Action::Kakan { tile: t("N") },
            ),
            (engine(ActionType::Tsumo, None, vec![]), Action::Tsumo),
            (engine(ActionType::Ron, Some(40), vec![]), Action::Ron),
            (
                engine(ActionType::KyushuKyuhai, None, vec![]),
                Action::KyushuKyuhai,
            ),
            (engine(ActionType::Pass, None, vec![]), Action::Pass),
        ];
        for (e, expected) in cases {
            assert_eq!(from_engine(&e), Some(expected.clone()), "{e:?}");
        }
        assert_eq!(
            from_engine(&engine(ActionType::Kita, Some(120), vec![])),
            None
        );
    }

    #[test]
    fn legal_merges_copies_of_the_same_tile_but_not_the_red_five() {
        let engine_legal = vec![
            engine(ActionType::Discard, Some(5), vec![]),
            engine(ActionType::Discard, Some(6), vec![]),
            engine(ActionType::Discard, Some(16), vec![]),
            engine(ActionType::Discard, Some(17), vec![]),
        ];
        let legal = legal(&engine_legal);
        let ours: Vec<Action> = legal.iter().map(|(a, _)| a.clone()).collect();
        assert_eq!(
            ours,
            vec![
                Action::Discard { tile: t("2m") },
                Action::Discard { tile: t("5mr") },
                Action::Discard { tile: t("5m") },
            ]
        );
        assert_eq!(legal[0].1.tile, Some(5), "the first engine action is kept");
    }

    #[test]
    fn json_shape() {
        assert_eq!(
            serde_json::to_value(Action::Discard { tile: t("5pr") }).unwrap(),
            serde_json::json!({ "type": "Discard", "tile": "5pr" })
        );
        assert_eq!(
            serde_json::to_value(Action::Pass).unwrap(),
            serde_json::json!({ "type": "Pass" })
        );
        assert_eq!(
            serde_json::to_value(Action::Chi {
                tile: t("3m"),
                with: [t("1m"), t("2m")]
            })
            .unwrap(),
            serde_json::json!({ "type": "Chi", "tile": "3m", "with": ["1m", "2m"] })
        );
    }
}
