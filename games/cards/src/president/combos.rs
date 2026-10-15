//! What may be played on what, under the order in force. [W][P]

use super::{Action, Jokers, Trick};
use crate::deck::{Card, Face, Rank};

/// Jokers played alone outrank everything, revolution or not.
pub const JOKERS_ALONE: u8 = 13;

/// [W] 3 lowest, then 4 … king, ace, and 2 highest; a revolution reverses it.
pub fn strength(rank: Rank, reversed: bool) -> u8 {
    let normal = match rank {
        Rank::Two => 12,
        other => other as u8 - 1,
    };
    if reversed {
        12 - normal
    } else {
        normal
    }
}

/// The value of a play of `rank` (`None`: jokers alone).
pub fn value(rank: Option<Rank>, reversed: bool) -> u8 {
    rank.map_or(JOKERS_ALONE, |r| strength(r, reversed))
}

/// For the exchange, always in the normal order: a joker is the best card.
pub fn face_value(face: Face) -> u8 {
    match face {
        Face::Joker => JOKERS_ALONE,
        Face::Natural { rank } => strength(rank, false),
    }
}

pub fn exchange_value(card: Card) -> u8 {
    face_value(card.face())
}

/// Where a card sorts in a hand under the order in force: weakest first.
pub fn hand_order(card: Card, reversed: bool) -> (u8, Card) {
    let v = match card.face() {
        Face::Joker => JOKERS_ALONE,
        Face::Natural { rank } => strength(rank, reversed),
    };
    (v, card)
}

/// Every play from `hand`. [W] A lead is one to four cards of a rank; on a
/// trick, as many cards of a strictly higher value, or a pass. Jokers:
/// [W][P] strong ones play alone or as a pair; [P] wild ones join a rank,
/// four cards at most, or play alone.
pub fn plays(hand: &[Card], trick: Option<&Trick>, jokers: Jokers, reversed: bool) -> Vec<Action> {
    let held_jokers = hand.iter().filter(|&&c| c == Card::Joker).count() as u8;
    let mut plays = Vec::new();
    for rank in Rank::ALL {
        let held = hand.iter().filter(|c| c.rank() == Some(rank)).count() as u8;
        for naturals in 1..=held {
            let extra = if jokers == Jokers::Wild {
                held_jokers.min(4 - naturals)
            } else {
                0
            };
            for j in 0..=extra {
                plays.push(Action::Play {
                    rank: Some(rank),
                    naturals,
                    jokers: j,
                });
            }
        }
    }
    if jokers != Jokers::Off {
        for j in 1..=held_jokers {
            plays.push(Action::Play {
                rank: None,
                naturals: 0,
                jokers: j,
            });
        }
    }
    let Some(trick) = trick else {
        return plays;
    };
    let count = trick.cards.len() as u8;
    let to_beat = value(trick.rank, reversed);
    plays.retain(|p| {
        matches!(p, Action::Play { rank, naturals, jokers }
            if naturals + jokers == count && value(*rank, reversed) > to_beat)
    });
    plays.push(Action::Pass);
    plays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_is_highest_and_a_revolution_reverses_the_order() {
        assert!(strength(Rank::Two, false) > strength(Rank::Ace, false));
        assert!(strength(Rank::Ace, false) > strength(Rank::Three, false));
        assert_eq!(strength(Rank::Three, false), 0);
        assert_eq!(strength(Rank::Three, true), 12);
        assert_eq!(strength(Rank::Two, true), 0);
        assert!(value(None, true) > strength(Rank::Three, true));
    }
}
