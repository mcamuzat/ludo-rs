use ludo_core::ai::{Budget, Level};
use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;
use riichienv_core::shanten::calculate_shanten;

use crate::{standings, Action, Riichi, State, Tile};

/// Points of a large hand (haneman ron): the scale of a hand's gains.
const HAND_SCALE: i32 = 12_000;

impl State {
    /// Dragons, the round wind and `seat`'s wind give a yaku as a triplet.
    fn is_yakuhai(&self, seat: Seat, tile: Tile) -> bool {
        let kind = tile.kind();
        let seat_wind = (seat.0 + 4 - self.inner.oya) % 4;
        kind >= 31 || kind == 27 + self.inner.round_wind || kind == 27 + seat_wind
    }

    fn shanten_without(&self, seat: Seat, tile: Tile) -> i32 {
        let mut hand: Vec<u32> = self.inner.players[seat.index()]
            .hand
            .iter()
            .map(|&id| u32::from(id))
            .collect();
        if let Some(i) = hand.iter().position(|&id| tile.matches_id(id as u8)) {
            hand.remove(i);
        }
        calculate_shanten(&hand)
    }

    /// How much a tile is worth keeping, all else equal (lower: discard first).
    fn keep_value(&self, seat: Seat, tile: Tile) -> u8 {
        let base = if tile.is_honor() {
            u8::from(self.is_yakuhai(seat, tile))
        } else if tile.is_terminal() {
            2
        } else if matches!(tile.kind() % 9, 1 | 7) {
            3
        } else {
            4
        };
        base + if tile.is_red() { 10 } else { 0 }
    }
}

impl Heuristics for Riichi {
    /// Among the sensible moves (`search_actions`): take a pon that gives a
    /// yaku, otherwise pass; discard the least valuable of the best discards.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let candidates = Self::search_actions(s, seat);
        if candidates.contains(&Action::Pass) {
            return candidates
                .iter()
                .find(|a| matches!(a, Action::Pon { .. }))
                .cloned()
                .unwrap_or(Action::Pass);
        }
        let discards = candidates.iter().filter_map(|a| match a {
            Action::Discard { tile } => Some(*tile),
            _ => None,
        });
        match discards.min_by_key(|&tile| (s.keep_value(seat, tile), rng.gen::<u16>())) {
            Some(tile) => Action::Discard { tile },
            None => candidates
                .choose(rng)
                .expect("a pending seat has a legal action")
                .clone(),
        }
    }

    /// Win if possible; riichi when tenpai; on a call window pass or a pon
    /// that gives a yaku (an open hand without one cannot win); otherwise the
    /// discards that lower shanten most, and any kan.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        if let Some(win) = legal
            .iter()
            .find(|a| matches!(a, Action::Tsumo | Action::Ron))
        {
            return vec![win.clone()];
        }
        if legal.contains(&Action::Riichi) {
            return vec![Action::Riichi];
        }
        if legal.contains(&Action::Pass) {
            return legal
                .into_iter()
                .filter(|a| match a {
                    Action::Pass => true,
                    Action::Pon { tile, .. } => s.is_yakuhai(seat, *tile),
                    _ => false,
                })
                .collect();
        }
        let shanten = |a: &Action| match a {
            Action::Discard { tile } => Some(s.shanten_without(seat, *tile)),
            _ => None,
        };
        let Some(best) = legal.iter().filter_map(shanten).min() else {
            return legal;
        };
        legal
            .into_iter()
            .filter(|a| shanten(a).is_none_or(|n| n == best))
            .collect()
    }

    /// Simulate to the end of the current hand, then score it: half the
    /// ranking the scores give, half the points won or lost in the hand (a
    /// single hand rarely changes the ranking, and the search needs a signal).
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        if sim.inner.is_done || sim.hand_key() == root.hand_key() {
            return None;
        }
        let (before, after) = (root.scores(), sim.scores());
        let ranks = standings(after).rewards();
        Some(
            (0..4)
                .map(|i| {
                    let gain = f64::from(after[i] - before[i]) / f64::from(2 * HAND_SCALE);
                    0.5 * ranks[i] + 0.5 * (0.5 + gain).clamp(0.0, 1.0)
                })
                .collect(),
        )
    }

    fn budget(level: Level, timed: bool) -> Option<Budget> {
        match (level, timed) {
            (Level::Easy, _) => None,
            (Level::Medium, true) => Some(Budget::Millis(300)),
            (Level::Medium, false) => Some(Budget::Iterations(30)),
            (Level::Hard, true) => Some(Budget::Millis(1000)),
            (Level::Hard, false) => Some(Budget::Iterations(300)),
        }
    }
}
