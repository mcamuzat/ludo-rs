use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;
use riichienv_core::action::ActionType;
use riichienv_core::state::legal_actions::GameStateLegalActions;

use crate::{Riichi, State};

impl State {
    /// Wall positions `observer` has seen: the revealed dora indicators.
    fn revealed_wall_positions(&self) -> Vec<usize> {
        let shift = usize::from(self.inner.wall.rinshan_draw_count);
        (0..self.inner.wall.dora_indicators.len())
            .map(|k| (4 + 2 * k).saturating_sub(shift))
            .collect()
    }

    /// In a call window, who may call depends on the (redrawn) hands.
    fn recompute_claims(&mut self, observer: Seat) {
        let Some((discarder, tile)) = self.claim_source() else {
            return;
        };
        let chankan = self.inner.pending_kan.is_some();
        let mut active = Vec::new();
        for i in 0..4u8 {
            if i == discarder {
                continue;
            }
            if i == observer.0 {
                if self.inner.active_players.contains(&i) {
                    active.push(i);
                }
                continue;
            }
            let (mut claims, _) = self.inner._get_claim_actions_for_player(i, discarder, tile);
            if chankan {
                // Only a win can rob a kan.
                claims.retain(|a| a.action_type == ActionType::Ron);
            }
            if claims.is_empty() {
                self.inner.current_claims.remove(&i);
            } else {
                self.inner.current_claims.insert(i, claims);
                active.push(i);
            }
        }
        self.inner.active_players = active;
    }
}

impl Determinize for Riichi {
    /// Redraws everything `observer` cannot see: the other hands, the live
    /// wall and the unrevealed dead wall (sizes and revealed indicators kept),
    /// then who may call; the answers others already gave are forgotten.
    /// Approximation: a redrawn hand of a player in riichi is not forced tenpai.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        if d.inner.is_done {
            return d;
        }
        let me = observer.index();
        let revealed = d.revealed_wall_positions();
        let mut pool: Vec<u8> = (0..4)
            .filter(|&p| p != me)
            .flat_map(|p| d.inner.players[p].hand.clone())
            .collect();
        pool.extend(
            d.inner
                .wall
                .tiles
                .iter()
                .enumerate()
                .filter(|(i, _)| !revealed.contains(i))
                .map(|(_, &t)| t),
        );
        // Sorted first: the same knowledge must give the same world, whatever the truth.
        pool.sort_unstable();
        pool.shuffle(rng);
        let mut fresh = pool.into_iter();

        let drawer = usize::from(d.inner.current_player);
        let old_drawn = d.inner.drawn_tile;
        let mut new_drawn = None;
        for p in (0..4).filter(|&p| p != me) {
            for slot in d.inner.players[p].hand.iter_mut() {
                let tile = fresh.next().expect("pool size matches");
                if p == drawer && Some(*slot) == old_drawn && new_drawn.is_none() {
                    new_drawn = Some(tile);
                }
                *slot = tile;
            }
            d.inner.players[p].hand.sort_unstable();
        }
        for (i, slot) in d.inner.wall.tiles.iter_mut().enumerate() {
            if !revealed.contains(&i) {
                *slot = fresh.next().expect("pool size matches");
            }
        }
        if drawer != me && new_drawn.is_some() {
            d.inner.drawn_tile = new_drawn;
        }

        if d.in_claim_window() {
            d.recompute_claims(observer);
        }
        // Others' answers are hidden, and so is whether they have answered:
        // every other asked seat decides again in the simulation.
        d.answers.retain(|&seat, _| seat == observer);
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::testing::wall;
    use crate::{Action, Riichi, State, Tile};

    fn t(text: &str) -> Tile {
        Tile::parse(text).unwrap()
    }

    fn after_first_discard() -> State {
        let mut s = State::start(
            1,
            Some(wall(
                [
                    "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                    "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                    "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
                    "5p 5pr S S S W W N N P P F F",
                ],
                &["N"],
            )),
        );
        Riichi::apply(&mut s, Seat(0), &Action::Discard { tile: t("5p") }).unwrap();
        s
    }

    fn hidden_pool(s: &State, observer: usize) -> Vec<u8> {
        let shift = usize::from(s.inner.wall.rinshan_draw_count);
        let revealed: Vec<usize> = (0..s.inner.wall.dora_indicators.len())
            .map(|k| (4 + 2 * k).saturating_sub(shift))
            .collect();
        let mut pool: Vec<u8> = (0..4)
            .filter(|&p| p != observer)
            .flat_map(|p| s.inner.players[p].hand.clone())
            .collect();
        pool.extend(
            s.inner
                .wall
                .tiles
                .iter()
                .enumerate()
                .filter(|(i, _)| !revealed.contains(i))
                .map(|(_, &t)| t),
        );
        pool.sort_unstable();
        pool
    }

    #[test]
    fn what_i_know_is_kept() {
        let s = after_first_discard();
        for seed in 0..10 {
            let d = Riichi::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(
                serde_json::to_value(Riichi::view(&d, Viewer::Seat(Seat(1)))).unwrap(),
                serde_json::to_value(Riichi::view(&s, Viewer::Seat(Seat(1)))).unwrap()
            );
            assert_eq!(d.inner.wall.tiles.len(), s.inner.wall.tiles.len());
            assert_eq!(d.inner.wall.dora_indicators, s.inner.wall.dora_indicators);
            assert_eq!(
                hidden_pool(&d, 1),
                hidden_pool(&s, 1),
                "the same tiles, redistributed"
            );
        }
    }

    #[test]
    fn hidden_tiles_really_move() {
        let s = after_first_discard();
        let moved = (0..10).any(|seed| {
            let d = Riichi::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            d.inner.players[3].hand != s.inner.players[3].hand
        });
        assert!(moved);
    }

    #[test]
    fn who_may_call_is_recomputed_from_the_new_hands() {
        let s = after_first_discard();
        let mut seat3_asked = 0;
        for seed in 0..40 {
            let d = Riichi::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            let pending = Riichi::pending(&d).0;
            assert!(pending.contains(&Seat(1)), "I still decide");
            assert!(Riichi::legal_actions(&d, Seat(1)).contains(&Action::Ron));
            for seat in pending {
                assert!(!Riichi::legal_actions(&d, seat).is_empty());
            }
            seat3_asked += usize::from(Riichi::pending(&d).contains(Seat(3)));
        }
        assert!(
            seat3_asked < 40,
            "seat 3 no longer always holds the pair of 5p"
        );
    }

    #[test]
    fn whether_others_have_answered_is_not_known() {
        // Seat 1 cannot tell whether seat 3 has already answered.
        let asked = after_first_discard();
        let mut answered = asked.clone();
        Riichi::apply(&mut answered, Seat(3), &Action::Pass).unwrap();
        let mut seat3_asked = 0;
        for seed in 0..300 {
            let a = Riichi::determinize(&asked, Seat(1), &mut SmallRng::seed_from_u64(seed));
            let b = Riichi::determinize(&answered, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Riichi::pending(&a), Riichi::pending(&b), "seed {seed}");
            assert_eq!(a.answers, b.answers, "seed {seed}");
            seat3_asked += usize::from(Riichi::pending(&a).contains(Seat(3)));
        }
        assert!(seat3_asked > 0, "the case where seat 3 may call occurs");
    }

    #[test]
    fn the_same_knowledge_gives_the_same_world() {
        // Seats 2 and 3 swap hands: seat 1 cannot tell the two states apart.
        let deal = |a: &str, b: &str| {
            let mut s = State::start(
                1,
                Some(wall(
                    [
                        "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                        "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                        a,
                        b,
                    ],
                    &["N"],
                )),
            );
            Riichi::apply(&mut s, Seat(0), &Action::Discard { tile: t("5p") }).unwrap();
            s
        };
        let x = deal(
            "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
            "5p 5pr S S S W W N N P P F F",
        );
        let y = deal(
            "5p 5pr S S S W W N N P P F F",
            "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
        );
        let dx = Riichi::determinize(&x, Seat(1), &mut SmallRng::seed_from_u64(5));
        let dy = Riichi::determinize(&y, Seat(1), &mut SmallRng::seed_from_u64(5));
        assert_eq!(dx.inner.players[2].hand, dy.inner.players[2].hand);
        assert_eq!(dx.inner.players[3].hand, dy.inner.players[3].hand);
        assert_eq!(dx.inner.wall.tiles, dy.inner.wall.tiles);
    }
}
