use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{strength, Action, Kind, Octopus, Phase, State};
use crate::deck::Rank;

/// A shown hand whose highest card is at most this is worth calling.
const SHOW_BELOW: u8 = 7;

/// The kind `hand` ranks best under, and how it ranks.
fn best_reading(hand: &[Rank]) -> (Kind, (u8, Vec<Rank>)) {
    [Kind::Clean, Kind::Set]
        .into_iter()
        .map(|kind| (kind, strength(hand, kind)))
        .min_by(|a, b| a.1.cmp(&b.1))
        .expect("two kinds")
}

/// The card whose loss leaves `hand` strongest.
fn worst_card(hand: &[Rank]) -> Option<Rank> {
    let mut cards = hand.to_vec();
    cards.sort_unstable();
    cards.dedup();
    cards.into_iter().min_by_key(|&card| {
        let mut rest = hand.to_vec();
        if let Some(at) = rest.iter().position(|&c| c == card) {
            rest.remove(at);
        }
        (best_reading(&rest).1, std::cmp::Reverse(card))
    })
}

impl Heuristics for Octopus {
    /// From what `seat` sees only. Stay in; play and pass the card whose
    /// loss leaves the hand strongest; call the showdown, in the kind its
    /// hand ranks best under, when that hand is clean or set and low.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let hand = &s.hands[seat.index()];
        let chosen = match s.phase {
            Phase::Declare => Some(Action::Stay),
            Phase::Play => worst_card(hand).map(|card| Action::Play { card }),
            Phase::Pass => worst_card(hand).map(|card| Action::Pass { card }),
            Phase::Call => {
                let (kind, (order, cards)) = best_reading(hand);
                let high = cards.first().map_or(0, |c| c.value());
                Some(if order == 0 && high <= SHOW_BELOW {
                    Action::Show { best: kind }
                } else {
                    Action::CallPass
                })
            }
            Phase::Over => None,
        };
        chosen
            .filter(|a| Self::legal_actions(s, seat).contains(a))
            .or_else(|| Self::legal_actions(s, seat).choose(rng).cloned())
            .unwrap_or(Action::Stay)
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use super::worst_card;
    use crate::octopus::testing::table;
    use crate::octopus::{Action, Config, Octopus};
    use crate::testing::{r, ranks};

    #[test]
    fn drops_the_card_that_spoils_the_hand() {
        // 2-3-5-5: dropping a 5 leaves a clean hand.
        assert_eq!(worst_card(&ranks(&[2, 3, 5, 5])), Some(r(5)));
        // 8-8-9-9-10: dropping the 10 leaves a set hand.
        assert_eq!(worst_card(&ranks(&[8, 8, 9, 9, 10])), Some(r(10)));
    }

    #[test]
    fn plays_the_spoiling_card() {
        let s = table(&[&[2, 3, 5, 5], &[1, 4], &[6, 7], &[8, 9]], &[]);
        let action = Octopus::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(action, Action::Play { card: r(5) });
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 4..=6u8 {
            for _ in 0..5 {
                let mut s = Octopus::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = Octopus::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = Octopus::rollout_action(&s, seat, &mut rng);
                    assert!(
                        Octopus::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    Octopus::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
