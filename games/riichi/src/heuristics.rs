use ludo_core::ai::{Budget, Level};
use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;
use riichienv_core::shanten::calculate_shanten;

use crate::{standings, Action, Riichi, State, Tile};

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
    /// Win if possible; riichi when tenpai; on a call window pass, except a
    /// pon that gives a yaku; otherwise the discard that lowers shanten most,
    /// keeping the most valuable tiles.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if let Some(win) = legal
            .iter()
            .find(|a| matches!(a, Action::Tsumo | Action::Ron))
        {
            return win.clone();
        }
        if legal.contains(&Action::Riichi) {
            return Action::Riichi;
        }
        if legal.contains(&Action::Pass) {
            return legal
                .iter()
                .find(|a| matches!(a, Action::Pon { tile, .. } if s.is_yakuhai(seat, *tile)))
                .cloned()
                .unwrap_or(Action::Pass);
        }
        let discards: Vec<Tile> = legal
            .iter()
            .filter_map(|a| match a {
                Action::Discard { tile } => Some(*tile),
                _ => None,
            })
            .collect();
        match discards.iter().min_by_key(|&&tile| {
            (
                s.shanten_without(seat, tile),
                s.keep_value(seat, tile),
                rng.gen::<u16>(),
            )
        }) {
            Some(&tile) => Action::Discard { tile },
            None => legal
                .choose(rng)
                .expect("a pending seat has a legal action")
                .clone(),
        }
    }

    /// Simulate to the end of the current hand, then estimate the final
    /// ranking from the scores.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (!sim.inner.is_done && sim.hand_key() != root.hand_key())
            .then(|| standings(sim.scores()).rewards())
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
