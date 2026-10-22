//! The Bread Basket deck and what a played card completes.

use serde::{Deserialize, Serialize};

/// [B] Copies of each value in the deck.
const COPIES: usize = 5;

/// A card: an Onion (−5) or a rank from 1 to 10. Cards of one value are
/// interchangeable. The Onion is the lowest card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "i8", into = "i8")]
pub struct Card(i8);

impl Card {
    pub const ONION: Card = Card(-5);

    pub fn new(value: i8) -> Option<Self> {
        (value == Self::ONION.0 || (1..=10).contains(&value)).then_some(Self(value))
    }

    pub fn value(self) -> i8 {
        self.0
    }

    pub fn is_onion(self) -> bool {
        self == Self::ONION
    }
}

impl TryFrom<i8> for Card {
    type Error = String;

    fn try_from(value: i8) -> Result<Self, String> {
        Card::new(value).ok_or_else(|| format!("carte hors de −5 et 1–10 : {value}"))
    }
}

impl From<Card> for i8 {
    fn from(card: Card) -> i8 {
        card.0
    }
}

/// [B] Five copies of each rank from 1 to 10 and five Onions: 55 cards,
/// in value order.
pub fn full_deck() -> Vec<Card> {
    std::iter::once(Card::ONION)
        .chain((1..=10).map(Card))
        .flat_map(|card| std::iter::repeat_n(card, COPIES))
        .collect()
}

/// [B] What a card left in hand costs at the end of the hand: its rank,
/// and 5 for an Onion ("worth negative five points in the hand").
pub fn hand_penalty(card: Card) -> i32 {
    if card.is_onion() {
        5
    } else {
        i32::from(card.0)
    }
}

/// [B] A sandwich scores its middle cards at face value (an Onion −5);
/// the bread scores nothing.
pub fn filling_points(cards: &[Card]) -> i32 {
    cards.iter().map(|c| i32::from(c.0)).sum()
}

/// [B] Cards dealt to each player: 10, 9, 8, 7, 6 for 2 to 6 players.
pub fn hand_size(players: u8) -> usize {
    12usize.saturating_sub(usize::from(players)).max(1)
}

/// What a card played at the end of the line completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    Nothing,
    /// The same card just before it: an empty sandwich.
    Air,
    /// The bread opening the sandwich is `line[start]`.
    Sandwich {
        start: usize,
    },
}

/// [B] A sandwich runs from a card to the same card, with every card
/// between them lower; two matching cards in a row are an air sandwich.
/// Sandwiches are picked up at once, so looking back from the end, the
/// first card not lower than `card` decides.
pub fn completes(line: &[Card], card: Card) -> Completion {
    match line.iter().rposition(|&c| c >= card) {
        Some(i) if line[i] == card && i + 1 == line.len() => Completion::Air,
        Some(i) if line[i] == card => Completion::Sandwich { start: i },
        _ => Completion::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(v: i8) -> Card {
        Card::new(v).unwrap()
    }

    fn c(values: &[i8]) -> Vec<Card> {
        values.iter().map(|&v| card(v)).collect()
    }

    #[test]
    fn the_deck_has_five_of_each_and_five_onions() {
        let deck = full_deck();
        assert_eq!(deck.len(), 55);
        assert_eq!(deck.iter().filter(|c| c.is_onion()).count(), 5);
        for v in 1..=10 {
            assert_eq!(deck.iter().filter(|c| c.value() == v).count(), 5, "{v}");
        }
    }

    #[test]
    fn card_rejects_other_values() {
        assert_eq!(Card::new(0), None);
        assert_eq!(Card::new(11), None);
        assert_eq!(Card::new(-4), None);
        assert_eq!(serde_json::from_str::<Card>("-5").unwrap(), Card::ONION);
        assert!(serde_json::from_str::<Card>("0").is_err());
        assert_eq!(serde_json::to_string(&card(7)).unwrap(), "7");
    }

    #[test]
    fn the_onion_is_the_lowest_card() {
        assert!(Card::ONION < card(1));
        assert!(Card::ONION.is_onion() && !card(1).is_onion());
    }

    #[test]
    fn completes_finds_simple_and_long_sandwiches() {
        assert_eq!(
            completes(&c(&[4, 3]), card(4)),
            Completion::Sandwich { start: 0 }
        );
        assert_eq!(
            completes(&c(&[2, 9, 8, 3]), card(9)),
            Completion::Sandwich { start: 1 }
        );
        assert_eq!(
            completes(&c(&[6, -5, 3]), card(6)),
            Completion::Sandwich { start: 0 }
        );
    }

    #[test]
    fn completes_stops_at_a_higher_card() {
        assert_eq!(completes(&c(&[5, 3, 7]), card(5)), Completion::Nothing);
        assert_eq!(completes(&c(&[1, 2]), card(3)), Completion::Nothing);
        assert_eq!(completes(&[], card(3)), Completion::Nothing);
    }

    #[test]
    fn completes_detects_air_including_onions() {
        assert_eq!(completes(&c(&[3, 8]), card(8)), Completion::Air);
        assert_eq!(completes(&c(&[-5]), card(-5)), Completion::Air);
    }

    #[test]
    fn penalties_and_filling() {
        assert_eq!(hand_penalty(card(7)), 7);
        assert_eq!(hand_penalty(Card::ONION), 5);
        assert_eq!(filling_points(&c(&[-5, 3])), -2);
        assert_eq!(filling_points(&[]), 0);
    }

    #[test]
    fn hand_sizes() {
        let sizes: Vec<usize> = (2..=6).map(hand_size).collect();
        assert_eq!(sizes, [10, 9, 8, 7, 6]);
    }
}
