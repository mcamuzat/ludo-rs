//! riichienv starts the next hand inside the same `step` and forgets its
//! results, so winners are evaluated just before, with its public evaluator
//! and the same conditions it uses (state/mod.rs, Tsumo and Ron branches).

use ludo_core::Seat;
use riichienv_core::hand_evaluator::HandEvaluator;
use riichienv_core::state::GameState;
use riichienv_core::types::{Conditions, Wind as EngineWind};
use riichienv_core::yaku::get_yaku_by_id;

use crate::view::{meld_views, tiles, WinView, YakuName};
use crate::{Action, Tile};

const NP: u8 = 4;

/// Ura-dora indicators, next to each revealed dora indicator in the dead wall.
pub(crate) fn ura_indicators(g: &GameState) -> Vec<u8> {
    let shift = usize::from(g.wall.rinshan_draw_count);
    (0..g.wall.dora_indicators.len())
        .filter_map(|i| g.wall.tiles.get((5 + 2 * i).saturating_sub(shift)).copied())
        .collect()
}

fn evaluate(
    g: &GameState,
    pid: u8,
    from: Option<Seat>,
    win_tile: u8,
    cond: Conditions,
    shown: Vec<u8>,
) -> WinView {
    let p = &g.players[usize::from(pid)];
    let ura = if p.riichi_declared {
        ura_indicators(g)
    } else {
        Vec::new()
    };
    let res = HandEvaluator::new(p.hand.clone(), p.melds.clone()).calc(
        win_tile,
        g.wall.dora_indicators.clone(),
        ura,
        Some(cond),
    );
    // Mahjong Soul rules enable every double yakuman: riichienv's cap never applies.
    let points = if from.is_some() {
        res.ron_agari
    } else if pid == g.oya {
        res.tsumo_agari_ko * 3
    } else {
        res.tsumo_agari_oya + 2 * res.tsumo_agari_ko
    };
    WinView {
        seat: Seat(pid),
        from,
        hand: tiles(&shown),
        melds: meld_views(&p.melds, pid),
        win_tile: Tile::from_id(win_tile),
        yaku: res
            .yaku
            .iter()
            .filter_map(|&id| get_yaku_by_id(id))
            .map(|y| YakuName {
                name: y.name_en,
                name_ja: y.name,
            })
            .collect(),
        han: res.han,
        fu: res.fu,
        points,
        yakuman: res.yakuman,
    }
}

/// The winners in `step`, before the engine applies it.
pub(crate) fn preview(
    g: &GameState,
    claim: Option<(u8, u8)>,
    step: &[(Seat, Action)],
) -> Vec<WinView> {
    if let Some((seat, _)) = step.iter().find(|(_, a)| *a == Action::Tsumo) {
        let pid = seat.0;
        let p = &g.players[usize::from(pid)];
        let cond = Conditions {
            tsumo: true,
            riichi: p.riichi_declared,
            double_riichi: p.double_riichi_declared,
            ippatsu: p.ippatsu_cycle,
            haitei: g.wall.drawable_count == 0 && !g.is_rinshan_flag,
            rinshan: g.is_rinshan_flag,
            tsumo_first_turn: g.is_first_turn && g.players.iter().all(|q| q.melds.is_empty()),
            player_wind: EngineWind::from((pid + NP - g.oya) % NP),
            round_wind: EngineWind::from(g.round_wind),
            riichi_sticks: g.riichi_sticks,
            honba: u32::from(g.honba),
            ..Default::default()
        };
        let win_tile = g.drawn_tile.unwrap_or(0);
        return vec![evaluate(g, pid, None, win_tile, cond, p.hand.clone())];
    }
    let Some((discarder, win_tile)) = claim else {
        return Vec::new();
    };
    let mut ronners: Vec<u8> = step
        .iter()
        .filter(|(_, a)| *a == Action::Ron)
        .map(|(s, _)| s.0)
        .collect();
    // The first winner after the discarder takes the honba.
    ronners.sort_by_key(|&p| (p + NP - discarder) % NP);
    ronners
        .iter()
        .enumerate()
        .map(|(i, &pid)| {
            let p = &g.players[usize::from(pid)];
            let cond = Conditions {
                tsumo: false,
                riichi: p.riichi_declared,
                double_riichi: p.double_riichi_declared,
                ippatsu: p.ippatsu_cycle,
                haitei: false,
                houtei: g.wall.drawable_count == 0 && !g.is_rinshan_flag,
                rinshan: false,
                chankan: g.pending_kan.is_some(),
                tsumo_first_turn: false,
                player_wind: EngineWind::from((pid + NP - g.oya) % NP),
                round_wind: EngineWind::from(g.round_wind),
                riichi_sticks: g.riichi_sticks,
                honba: if i == 0 { u32::from(g.honba) } else { 0 },
                ..Default::default()
            };
            let mut shown = p.hand.clone();
            shown.push(win_tile);
            evaluate(g, pid, Some(Seat(discarder)), win_tile, cond, shown)
        })
        .collect()
}
