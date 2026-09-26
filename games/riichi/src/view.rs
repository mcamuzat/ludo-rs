use ludo_core::{Seat, Viewer};
use riichienv_core::hand_evaluator::HandEvaluator;
use riichienv_core::shanten::calculate_shanten;
use riichienv_core::types::{Meld, MeldType};
use serde::Serialize;

use crate::{State, Tile};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Wind {
    E,
    S,
    W,
    N,
}

impl Wind {
    fn from_index(i: u8) -> Self {
        [Wind::E, Wind::S, Wind::W, Wind::N][usize::from(i % 4)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MeldKind {
    Chi,
    Pon,
    Daiminkan,
    Ankan,
    Kakan,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    /// My concealed tiles, sorted, without the tile I just drew.
    pub hand: Vec<Tile>,
    pub drawn: Option<Tile>,
    /// Index = seat.
    pub players: Vec<PlayerView>,
    pub round: RoundView,
    pub help: Option<Help>,
    /// My answer to the current call window is recorded; others are deciding.
    pub answered: bool,
    /// The last finished hand, until the next one ends.
    pub last_hand: Option<HandResult>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    pub seat_wind: Wind,
    pub score: i32,
    pub riichi: bool,
    pub hand_size: usize,
    pub discards: Vec<DiscardView>,
    pub melds: Vec<MeldView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscardView {
    pub tile: Tile,
    pub tsumogiri: bool,
    pub riichi: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeldView {
    pub kind: MeldKind,
    pub tiles: Vec<Tile>,
    pub from: Option<Seat>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundView {
    pub wind: Wind,
    /// 1..=4 within the wind.
    pub hand: u8,
    pub honba: u8,
    pub riichi_sticks: u32,
    pub wall_left: u8,
    pub dora_indicators: Vec<Tile>,
    pub dealer: Seat,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Help {
    /// 0 = tenpai (with 13 tiles) ; -1 = complete hand.
    pub shanten: i32,
    /// Tiles that complete the hand, when tenpai with 13 tiles.
    pub waits: Vec<Tile>,
}

pub(crate) fn tiles(ids: &[u8]) -> Vec<Tile> {
    let mut out: Vec<Tile> = ids.iter().map(|&id| Tile::from_id(id)).collect();
    out.sort();
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DrawKind {
    /// The wall ran out.
    Exhaustive,
    /// Kyuushu kyuuhai: nine different terminals or honors declared.
    NineTerminals,
    /// Four winds, four riichi, four kans…
    Abortive,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YakuName {
    pub name: String,
    pub name_ja: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WinView {
    pub seat: Seat,
    /// Who dealt in (ron); `None` for tsumo.
    pub from: Option<Seat>,
    /// Concealed tiles including the winning tile, sorted.
    pub hand: Vec<Tile>,
    pub melds: Vec<MeldView>,
    pub win_tile: Tile,
    pub yaku: Vec<YakuName>,
    pub han: u32,
    pub fu: u32,
    /// Hand value paid by the others (honba included, riichi sticks excluded).
    pub points: u32,
    pub yakuman: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HandResult {
    /// 1 for the first hand of the game.
    pub number: u32,
    pub wins: Vec<WinView>,
    /// `Some` when nobody won.
    pub draw: Option<DrawKind>,
    /// Score change of each seat at the settlement (the winner collects the
    /// riichi sticks; a declarer's stick was paid when declaring).
    pub deltas: Vec<i32>,
    /// Revealed when a winner was in riichi.
    pub ura_indicators: Vec<Tile>,
}

pub(crate) fn meld_views(melds: &[Meld], owner: u8) -> Vec<MeldView> {
    melds
        .iter()
        .map(|m| MeldView {
            kind: match m.meld_type {
                MeldType::Chi => MeldKind::Chi,
                MeldType::Pon => MeldKind::Pon,
                MeldType::Daiminkan => MeldKind::Daiminkan,
                MeldType::Ankan => MeldKind::Ankan,
                MeldType::Kakan => MeldKind::Kakan,
            },
            tiles: tiles(&m.tiles),
            from: u8::try_from(m.from_who)
                .ok()
                .filter(|&w| w < 4 && w != owner)
                .map(Seat),
        })
        .collect()
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let g = &s.inner;
        let me = match viewer {
            Viewer::Seat(seat) => Some(seat),
            Viewer::Spectator => None,
        };
        let mut hand_ids: Vec<u8> = me
            .map(|m| g.players[m.index()].hand.clone())
            .unwrap_or_default();
        // The drawn tile is shown apart, while it is my turn to discard.
        let drawn = match (me, g.drawn_tile) {
            (Some(m), Some(d))
                if !s.in_claim_window() && g.current_player == m.0 && hand_ids.contains(&d) =>
            {
                let i = hand_ids.iter().position(|&x| x == d).expect("contained");
                hand_ids.remove(i);
                Some(Tile::from_id(d))
            }
            _ => None,
        };
        let players = (0..4u8)
            .map(|p| {
                let player = &g.players[usize::from(p)];
                PlayerView {
                    seat_wind: Wind::from_index(p + 4 - g.oya),
                    score: player.score,
                    riichi: player.riichi_declared,
                    hand_size: player.hand.len(),
                    discards: player
                        .discards
                        .iter()
                        .enumerate()
                        .map(|(j, &id)| DiscardView {
                            tile: Tile::from_id(id),
                            tsumogiri: !player.discard_from_hand.get(j).copied().unwrap_or(true),
                            riichi: player.discard_is_riichi.get(j).copied().unwrap_or(false),
                        })
                        .collect(),
                    melds: meld_views(&player.melds, p),
                }
            })
            .collect();
        let help = me.map(|m| {
            let player = &g.players[m.index()];
            let as_u32: Vec<u32> = player.hand.iter().map(|&id| u32::from(id)).collect();
            let waits = HandEvaluator::new(player.hand.clone(), player.melds.clone())
                .get_waits_u8()
                .into_iter()
                .map(|kind| Tile::new(kind, false))
                .collect();
            Help {
                shanten: calculate_shanten(&as_u32),
                waits,
            }
        });
        View {
            me,
            hand: tiles(&hand_ids),
            drawn,
            players,
            round: RoundView {
                wind: Wind::from_index(g.round_wind),
                hand: g.kyoku_idx + 1,
                honba: g.honba,
                riichi_sticks: g.riichi_sticks,
                wall_left: g.wall.drawable_count,
                dora_indicators: g
                    .wall
                    .dora_indicators
                    .iter()
                    .map(|&id| Tile::from_id(id))
                    .collect(),
                dealer: Seat(g.oya),
            },
            help,
            answered: me.is_some_and(|m| s.answers.contains_key(&m)),
            last_hand: s.last_hand.clone(),
        }
    }
}
