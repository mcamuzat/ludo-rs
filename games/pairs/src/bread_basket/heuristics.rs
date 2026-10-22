use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::cards::{completes, filling_points, full_deck, hand_penalty, Card, Completion};
use super::{Action, BreadBasket, Phase, State};

/// How often the rollout plays a random legal action instead.
const WANDER: f64 = 0.1;
/// What a drawn card is expected to cost in hand: the deck's mean penalty.
const DRAW_COST: f64 = 300.0 / 55.0;

/// The best filling any card could pick up from `line`: what it offers the
/// next player.
fn exposure(line: &[Card]) -> i32 {
    full_deck()
        .into_iter()
        .filter_map(|card| match completes(line, card) {
            Completion::Sandwich { start } => Some(filling_points(&line[start + 1..])),
            _ => None,
        })
        .max()
        .unwrap_or(0)
        .max(0)
}

/// From what `seat` sees: the filling a play picks up, plus the penalty it
/// sheds, minus what the line it leaves offers the next player.
fn play_value(line: &[Card], card: Card) -> f64 {
    let mut after = line.to_vec();
    let gain = match completes(line, card) {
        Completion::Sandwich { start } => {
            let gain = filling_points(&line[start + 1..]);
            after.truncate(start);
            f64::from(gain)
        }
        Completion::Air => {
            after.pop();
            -DRAW_COST
        }
        Completion::Nothing => {
            after.push(card);
            0.0
        }
    };
    gain + f64::from(hand_penalty(card)) - f64::from(exposure(&after))
}

impl Heuristics for BreadBasket {
    /// Complete the best sandwich, else shed the card that offers the next
    /// player least; draw when every play costs more than a drawn card.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Play || rng.gen_bool(WANDER) {
            return legal.choose(rng).cloned().unwrap_or(Action::Draw);
        }
        let draw = -DRAW_COST - f64::from(exposure(&s.line));
        s.playable(seat)
            .into_iter()
            .map(|card| (play_value(&s.line, card), card))
            .filter(|(value, _)| *value > draw)
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(Action::Draw, |(_, card)| Action::Play { card })
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::bread_basket::testing::{card, table};
    use crate::bread_basket::{Action, BreadBasket, Config};

    /// What the rollout plays most often over 50 tries (it wanders).
    fn usual(s: &crate::bread_basket::State, seat: u8) -> Action {
        let mut rng = SmallRng::seed_from_u64(9);
        let mut counts: Vec<(Action, u32)> = Vec::new();
        for _ in 0..50 {
            let a = BreadBasket::rollout_action(s, Seat(seat), &mut rng);
            match counts.iter_mut().find(|(b, _)| *b == a) {
                Some((_, n)) => *n += 1,
                None => counts.push((a, 1)),
            }
        }
        counts.into_iter().max_by_key(|(_, n)| *n).unwrap().0
    }

    #[test]
    fn completes_the_best_sandwich() {
        let s = table(&[&[9, 6, 2], &[1]], &[9, 8, 6], &[3, 4]);
        assert_eq!(usual(&s, 0), Action::Play { card: card(9) });
    }

    #[test]
    fn sheds_an_onion_rather_than_feed_a_sandwich() {
        let s = table(&[&[-5, 8], &[1]], &[10, 9], &[3, 4]);
        assert_eq!(usual(&s, 0), Action::Play { card: card(-5) });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=6u8 {
            for _ in 0..5 {
                let mut s = BreadBasket::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = BreadBasket::pending(&s).0.first() {
                    let action = BreadBasket::rollout_action(&s, seat, &mut rng);
                    assert!(
                        BreadBasket::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    BreadBasket::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
