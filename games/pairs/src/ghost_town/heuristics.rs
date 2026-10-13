use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, GhostTown, Phase, State};
use crate::deck::lowest;

/// Draw whenever a draw is expected to pay at least this many chips.
const DRAW: f64 = 0.5;
/// Below this expected loss, never draw.
const GAMBLE: f64 = -2.0;

impl Heuristics for GhostTown {
    /// From what `seat` sees only. Draw while a draw pays; otherwise load
    /// the left neighbour's house, stall by playing at home, gamble on a
    /// mildly bad draw, or fold.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if s.phase == Phase::Setup {
            return legal.choose(rng).cloned().unwrap_or(Action::Fold);
        }
        let can_draw = legal.contains(&Action::Draw);
        let value = s.draw_value(seat);
        if can_draw && value >= DRAW {
            return Action::Draw;
        }
        let (mine, theirs): (Vec<&Action>, Vec<&Action>) = legal
            .iter()
            .filter(|a| matches!(a, Action::Play { .. }))
            .partition(|a| matches!(a, Action::Play { to, .. } if *to == seat));
        let roll: f64 = rng.gen();
        if roll < 0.4 {
            if let Some(&play) = theirs.choose(rng) {
                return play.clone();
            }
        }
        if roll < 0.65 {
            if let Some(&play) = mine.choose(rng) {
                return play.clone();
            }
        }
        if can_draw && value > GAMBLE && roll < 0.85 {
            return Action::Draw;
        }
        Action::Fold
    }

    /// The card for the street is not worth a search (and a table of AIs
    /// would think one after the other): the lowest, to move first while
    /// the pot is full. In play, every legal action.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Setup || legal.is_empty() {
            return legal;
        }
        match lowest(&s.hands[seat.index()]) {
            Some(card) => vec![Action::Place { card }],
            None => legal,
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::ghost_town::testing::{setup, table};
    use crate::ghost_town::{Action, Config, GhostTown, State};
    use crate::testing::r;

    fn rollout(s: &State, seat: u8, seed: u64) -> Action {
        GhostTown::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(seed))
    }

    /// Every card not on the table, as the draw pile.
    fn rest(taken: &[u8]) -> Vec<u8> {
        let mut deck: Vec<u8> = (1..=10u8).flat_map(|v| vec![v; usize::from(v)]).collect();
        for card in taken {
            let at = deck.iter().position(|c| c == card).expect("card left");
            deck.remove(at);
        }
        deck
    }

    #[test]
    fn draws_while_a_draw_pays() {
        // An empty house and a low street: nearly every card wins chips.
        let s = table(&[&[6], &[8]], &[&[], &[]], &[1], &rest(&[6, 8, 1]));
        assert!(s.draw_value(Seat(0)) > 5.0);
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Draw);
        }
    }

    #[test]
    fn stops_drawing_when_a_draw_costs_more_than_it_pays() {
        // A house and a street full of high cards: most cards bust.
        let s = table(
            &[&[], &[8]],
            &[&[10, 9, 7, 6], &[]],
            &[8, 5],
            &rest(&[10, 9, 7, 6, 8, 5, 8]),
        );
        assert!(s.draw_value(Seat(0)) < -2.0);
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Fold);
        }
    }

    #[test]
    fn the_draw_value_is_capped_by_the_pot() {
        let mut s = table(&[&[6], &[8]], &[&[], &[]], &[1], &rest(&[6, 8, 1]));
        let rich = s.draw_value(Seat(0));
        s.pot = 1;
        assert!(s.draw_value(Seat(0)) <= 1.0);
        assert!(rich > 1.0);
    }

    #[test]
    fn the_search_does_not_deliberate_over_the_card_to_place() {
        let s = setup(&[&[9, 4, 9, 6, 7], &[2, 8, 3, 5, 5]], &[1]);
        assert_eq!(
            GhostTown::search_actions(&s, Seat(0)),
            vec![Action::Place { card: r(4) }]
        );
        let p = table(&[&[6, 7], &[8]], &[&[6], &[]], &[2], &[1]);
        assert_eq!(
            GhostTown::search_actions(&p, Seat(0)),
            GhostTown::legal_actions(&p, Seat(0))
        );
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=6u8 {
            for _ in 0..5 {
                let mut s = GhostTown::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = GhostTown::pending(&s).0.first() {
                    let action = GhostTown::rollout_action(&s, seat, &mut rng);
                    assert!(
                        GhostTown::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    GhostTown::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
