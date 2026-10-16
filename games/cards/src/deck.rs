//! The French-suited pack shared by the classic card games. Each game ranks
//! the cards its own way: this module knows no order of strength.

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
}

/// In the usual order of a pack, two first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Card {
    Natural { rank: Rank, suit: Suit },
    Joker,
}

/// What a card shows when suits do not matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Face {
    Natural { rank: Rank },
    Joker,
}

impl Card {
    pub fn face(self) -> Face {
        match self {
            Card::Natural { rank, .. } => Face::Natural { rank },
            Card::Joker => Face::Joker,
        }
    }

    pub fn rank(self) -> Option<Rank> {
        match self {
            Card::Natural { rank, .. } => Some(rank),
            Card::Joker => None,
        }
    }

    pub fn suit(self) -> Option<Suit> {
        match self {
            Card::Natural { suit, .. } => Some(suit),
            Card::Joker => None,
        }
    }
}

pub fn pack_52() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| {
            Rank::ALL
                .iter()
                .map(move |&rank| Card::Natural { rank, suit })
        })
        .collect()
}

/// The piquet pack: seven to ace.
pub fn pack_32() -> Vec<Card> {
    pack_52()
        .into_iter()
        .filter(|c| c.rank() >= Some(Rank::Seven))
        .collect()
}

pub fn pack_54() -> Vec<Card> {
    let mut pack = pack_52();
    pack.extend([Card::Joker, Card::Joker]);
    pack
}

/// `cards` shuffled from `seed`. ChaCha, not `SmallRng`: the latter is a
/// different generator on wasm32 and on 64-bit targets, and a log recorded
/// in the browser must replay natively.
pub fn shuffled(mut cards: Vec<Card>, seed: u64) -> Vec<Card> {
    cards.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
    cards
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deal {
    pub hands: Vec<Vec<Card>>,
    /// The cards left over, shown to everyone and kept out of the round.
    pub set_aside: Vec<Card>,
}

/// Shuffles and deals the same number of cards to each of `players`.
pub fn deal_evenly(cards: Vec<Card>, players: u8, seed: u64) -> Deal {
    let mut cards = shuffled(cards, seed);
    let n = usize::from(players);
    let set_aside = cards.split_off(cards.len() - cards.len() % n);
    let mut hands = vec![Vec::new(); n];
    for (i, card) in cards.into_iter().enumerate() {
        hands[i % n].push(card);
    }
    Deal { hands, set_aside }
}

/// For determinization: the cards of `hands` are shuffled among them, each
/// hand keeping its size.
pub fn redeal<R: Rng + ?Sized>(hands: &mut [&mut Vec<Card>], rng: &mut R) {
    let sizes: Vec<usize> = hands.iter().map(|h| h.len()).collect();
    let mut pool = Vec::new();
    for hand in hands.iter_mut() {
        pool.append(hand);
    }
    pool.shuffle(rng);
    for (hand, size) in hands.iter_mut().zip(sizes) {
        let at = pool.len() - size;
        hand.extend(pool.drain(at..));
    }
}

/// Like `redeal`, but each card may only go to a hand `i` with
/// `accepts(i, card)` (a player known to be out of a suit gets none of it).
/// The hands given must already satisfy `accepts`: after a few dead ends
/// they are kept as they are.
pub fn redeal_where<R: Rng + ?Sized>(
    hands: &mut [&mut Vec<Card>],
    accepts: &dyn Fn(usize, Card) -> bool,
    rng: &mut R,
) {
    const TRIES: usize = 20;
    let sizes: Vec<usize> = hands.iter().map(|h| h.len()).collect();
    let mut pool: Vec<Card> = hands.iter().flat_map(|h| h.iter().copied()).collect();
    for _ in 0..TRIES {
        pool.shuffle(rng);
        // The most constrained cards first, so that they still find a place.
        pool.sort_by_key(|&card| (0..sizes.len()).filter(|&i| accepts(i, card)).count());
        let mut dealt: Vec<Vec<Card>> = sizes.iter().map(|&n| Vec::with_capacity(n)).collect();
        let complete = pool.iter().all(|&card| {
            let open: Vec<usize> = (0..sizes.len())
                .filter(|&i| dealt[i].len() < sizes[i] && accepts(i, card))
                .collect();
            if open.is_empty() {
                return false;
            }
            let pick = rng.gen_range(0..open.len() as u32) as usize;
            dealt[open[pick]].push(card);
            true
        });
        if complete {
            for (hand, cards) in hands.iter_mut().zip(dealt) {
                **hand = cards;
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn packs_hold_the_right_cards() {
        let full = pack_52();
        assert_eq!(full.len(), 52);
        assert_eq!(full.iter().collect::<HashSet<_>>().len(), 52);
        let piquet = pack_32();
        assert_eq!(piquet.len(), 32);
        assert!(piquet.iter().all(|c| c.rank() >= Some(Rank::Seven)));
        let with_jokers = pack_54();
        assert_eq!(with_jokers.len(), 54);
        assert_eq!(with_jokers.iter().filter(|&&c| c == Card::Joker).count(), 2);
    }

    #[test]
    fn an_even_deal_sets_the_rest_aside() {
        let deal = deal_evenly(pack_52(), 5, 1);
        assert!(deal.hands.iter().all(|h| h.len() == 10));
        assert_eq!(deal.set_aside.len(), 2);
        let mut all: Vec<Card> = deal.hands.concat();
        all.extend(&deal.set_aside);
        all.sort();
        let mut pack = pack_52();
        pack.sort();
        assert_eq!(all, pack);
        assert!(deal_evenly(pack_52(), 4, 1).set_aside.is_empty());
    }

    #[test]
    fn the_deal_of_a_seed_is_pinned() {
        let deal = deal_evenly(pack_52(), 4, 42);
        assert_eq!(deal, deal_evenly(pack_52(), 4, 42));
        assert_ne!(deal.hands[0], deal_evenly(pack_52(), 4, 43).hands[0]);
        // Pinned: a change of shuffle would make browser logs unreadable.
        let pinned = vec![
            Card::Natural {
                rank: Rank::Five,
                suit: Suit::Hearts,
            },
            Card::Natural {
                rank: Rank::Ace,
                suit: Suit::Clubs,
            },
            Card::Natural {
                rank: Rank::Jack,
                suit: Suit::Hearts,
            },
            Card::Natural {
                rank: Rank::Six,
                suit: Suit::Hearts,
            },
        ];
        assert_eq!(deal.hands[0][..4].to_vec(), pinned);
    }

    #[test]
    fn redeal_keeps_the_sizes_and_the_cards() {
        let mut a = pack_52()[..5].to_vec();
        let mut b = pack_52()[5..12].to_vec();
        let before: HashSet<Card> = a.iter().chain(&b).copied().collect();
        redeal(&mut [&mut a, &mut b], &mut SmallRng::seed_from_u64(3));
        assert_eq!((a.len(), b.len()), (5, 7));
        assert_eq!(a.iter().chain(&b).copied().collect::<HashSet<_>>(), before);
    }

    #[test]
    fn a_card_shows_its_face() {
        let seven = Card::Natural {
            rank: Rank::Seven,
            suit: Suit::Hearts,
        };
        assert_eq!(seven.face(), Face::Natural { rank: Rank::Seven });
        assert_eq!(seven.rank(), Some(Rank::Seven));
        assert_eq!(Card::Joker.face(), Face::Joker);
        assert_eq!(Card::Joker.rank(), None);
        assert_eq!(
            serde_json::to_value(seven).unwrap(),
            serde_json::json!({ "kind": "Natural", "rank": "Seven", "suit": "Hearts" })
        );
        assert_eq!(
            serde_json::to_value(Card::Joker).unwrap(),
            serde_json::json!({ "kind": "Joker" })
        );
    }

    fn three_suits() -> (Vec<Card>, Vec<Card>, Vec<Card>) {
        let pack = pack_52();
        (
            pack[..5].to_vec(),
            pack[13..18].to_vec(),
            pack[26..31].to_vec(),
        )
    }

    fn is_heart(card: Card) -> bool {
        matches!(
            card,
            Card::Natural {
                suit: Suit::Hearts,
                ..
            }
        )
    }

    #[test]
    fn redeal_where_keeps_sizes_and_cards() {
        let (mut a, mut b, mut c) = three_suits();
        a.truncate(3);
        let before: HashSet<Card> = a.iter().chain(&b).chain(&c).copied().collect();
        redeal_where(
            &mut [&mut a, &mut b, &mut c],
            &|i, card| i != 0 || !is_heart(card),
            &mut SmallRng::seed_from_u64(1),
        );
        assert_eq!((a.len(), b.len(), c.len()), (3, 5, 5));
        let after: HashSet<Card> = a.iter().chain(&b).chain(&c).copied().collect();
        assert_eq!(after, before);
    }

    #[test]
    fn redeal_where_respects_the_constraint() {
        for seed in 0..200 {
            let (mut a, mut b, mut c) = three_suits();
            redeal_where(
                &mut [&mut a, &mut b, &mut c],
                &|i, card| i != 0 || !is_heart(card),
                &mut SmallRng::seed_from_u64(seed),
            );
            assert!(!a.iter().any(|&card| is_heart(card)), "seed {seed}");
        }
    }

    #[test]
    fn redeal_where_still_shuffles() {
        let (a0, b0, c0) = three_suits();
        let moved = (0..20).any(|seed| {
            let (mut a, mut b, mut c) = three_suits();
            redeal_where(
                &mut [&mut a, &mut b, &mut c],
                &|i, card| i != 0 || !is_heart(card),
                &mut SmallRng::seed_from_u64(seed),
            );
            let sorted = |mut h: Vec<Card>| {
                h.sort();
                h
            };
            (sorted(a), sorted(b), sorted(c))
                != (sorted(a0.clone()), sorted(b0.clone()), sorted(c0.clone()))
        });
        assert!(moved);
    }
}
