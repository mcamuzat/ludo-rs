//! The best poker hand of any number of cards ([L] p. 10). A hand of fewer
//! than five cards plays what it has: no straight nor flush, fewer kickers.

use serde::{Deserialize, Serialize};

use crate::deck::{Card, Rank, Suit};

/// [L] From worst to best. Five of a kind needs wild cards and a royal flush
/// is the best straight flush: neither has its own variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Category {
    HighCard,
    Pair,
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    Quads,
    StraightFlush,
}

/// A poker hand: its category and its (up to) five cards, most significant
/// first (the groups, then the kickers, high to low; the ace last in a
/// five-high straight). Hands compare by category, then rank by rank:
/// "a kicker is better than nothing" ([L] p. 12).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PokerHand {
    pub category: Category,
    pub cards: Vec<Card>,
}

impl PokerHand {
    fn key(&self) -> (Category, Vec<Rank>) {
        (
            self.category,
            self.cards.iter().filter_map(|c| c.rank()).collect(),
        )
    }
}

/// Equal when they rank the same: suits never break a tie.
impl PartialEq for PokerHand {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for PokerHand {}

impl PartialOrd for PokerHand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PokerHand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key().cmp(&other.key())
    }
}

fn rank_index(rank: Rank) -> usize {
    rank as usize
}

/// The highest straight among `ranks` (indexed by `Rank as usize`), as its
/// five ranks from the top; the ace also plays low ([L] A-2-3-4-5).
fn straight(present: &[bool; 13]) -> Option<[Rank; 5]> {
    // The k-th card down from `high`; below the two comes the ace.
    let at = |high: usize, k: usize| high.checked_sub(k).unwrap_or(12);
    // High card from the ace (12) down to the five (3).
    (3..13).rev().find_map(|high: usize| {
        (0..5)
            .all(|k| present[at(high, k)])
            .then(|| std::array::from_fn(|k| Rank::ALL[at(high, k)]))
    })
}

/// The cards of `cards` of rank `rank`, best suit first (spades, hearts,
/// diamonds, clubs: the order of `Suit`).
fn of_rank(cards: &[Card], rank: Rank) -> Vec<Card> {
    let mut out: Vec<Card> = cards
        .iter()
        .copied()
        .filter(|c| c.rank() == Some(rank))
        .collect();
    out.sort_by_key(|c| c.suit());
    out
}

/// The best poker hand that `cards` holds. `None` for no card at all.
pub fn best_hand(cards: &[Card]) -> Option<PokerHand> {
    if cards.is_empty() {
        return None;
    }
    let size = cards.len().min(5);
    let mut counts = [0u8; 13];
    for card in cards {
        if let Some(rank) = card.rank() {
            counts[rank_index(rank)] += 1;
        }
    }

    // Straight flush and flush, suit by suit.
    let mut straight_flush: Option<PokerHand> = None;
    let mut flush: Option<PokerHand> = None;
    for suit in Suit::ALL {
        let suited: Vec<Card> = cards
            .iter()
            .copied()
            .filter(|c| c.suit() == Some(suit))
            .collect();
        if suited.len() < 5 {
            continue;
        }
        let mut present = [false; 13];
        for c in &suited {
            if let Some(r) = c.rank() {
                present[rank_index(r)] = true;
            }
        }
        if let Some(ranks) = straight(&present) {
            let hand = PokerHand {
                category: Category::StraightFlush,
                cards: ranks
                    .iter()
                    .map(|&rank| Card::Natural { rank, suit })
                    .collect(),
            };
            if straight_flush.as_ref().is_none_or(|best| hand > *best) {
                straight_flush = Some(hand);
            }
        }
        let mut top = suited;
        top.sort_by_key(|a| std::cmp::Reverse(a.rank()));
        top.truncate(5);
        let hand = PokerHand {
            category: Category::Flush,
            cards: top,
        };
        if flush.as_ref().is_none_or(|best| hand > *best) {
            flush = Some(hand);
        }
    }
    if let Some(hand) = straight_flush {
        return Some(hand);
    }

    // Ranks by group size, then rank: the highest groups first.
    let mut groups: Vec<(u8, Rank)> = Rank::ALL
        .iter()
        .filter(|&&r| counts[rank_index(r)] > 0)
        .map(|&r| (counts[rank_index(r)], r))
        .collect();
    groups.sort_by(|a, b| b.cmp(a));

    // Takes `n` cards of `rank`, then fills up to `size` with the highest
    // other cards.
    let build = |category: Category, parts: &[(Rank, usize)]| -> PokerHand {
        let mut chosen: Vec<Card> = Vec::new();
        for &(rank, n) in parts {
            chosen.extend(of_rank(cards, rank).into_iter().take(n));
        }
        let used: Vec<Rank> = parts.iter().map(|&(r, _)| r).collect();
        let mut rest: Vec<Card> = cards
            .iter()
            .copied()
            .filter(|c| c.rank().is_some_and(|r| !used.contains(&r)))
            .collect();
        rest.sort_by(|a, b| b.rank().cmp(&a.rank()).then(a.suit().cmp(&b.suit())));
        let room = size.saturating_sub(chosen.len());
        chosen.extend(rest.into_iter().take(room));
        PokerHand {
            category,
            cards: chosen,
        }
    };

    let (top_count, top_rank) = groups[0];
    if top_count == 4 {
        return Some(build(Category::Quads, &[(top_rank, 4)]));
    }
    if top_count == 3 {
        if let Some(&(_, pair)) = groups.iter().skip(1).find(|(n, _)| *n >= 2) {
            return Some(build(Category::FullHouse, &[(top_rank, 3), (pair, 2)]));
        }
    }
    if let Some(hand) = flush {
        return Some(hand);
    }
    let mut present = [false; 13];
    for (i, &n) in counts.iter().enumerate() {
        present[i] = n > 0;
    }
    if let Some(ranks) = straight(&present) {
        let parts: Vec<(Rank, usize)> = ranks.iter().map(|&r| (r, 1)).collect();
        return Some(build(Category::Straight, &parts));
    }
    if top_count == 3 {
        return Some(build(Category::Trips, &[(top_rank, 3)]));
    }
    if top_count == 2 {
        if let Some(&(2, second)) = groups.get(1) {
            return Some(build(Category::TwoPair, &[(top_rank, 2), (second, 2)]));
        }
        return Some(build(Category::Pair, &[(top_rank, 2)]));
    }
    Some(build(Category::HighCard, &[]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::president::testing::cards;

    fn best(text: &str) -> PokerHand {
        best_hand(&cards(text)).unwrap()
    }

    fn ranks(hand: &PokerHand) -> String {
        hand.cards
            .iter()
            .map(|c| match c.rank().unwrap() {
                Rank::Ten => 'T',
                Rank::Jack => 'J',
                Rank::Queen => 'Q',
                Rank::King => 'K',
                Rank::Ace => 'A',
                r => char::from(b'2' + r as u8),
            })
            .collect()
    }

    #[test]
    fn finds_each_category() {
        let cases = [
            ("2h 5s 9d Jc Kh 3c", Category::HighCard, "KJ953"),
            ("2h 2s 9d Jc Kh 3c", Category::Pair, "22KJ9"),
            ("2h 2s 9d 9c Kh 3c", Category::TwoPair, "9922K"),
            ("9h 9s 9d Jc Kh", Category::Trips, "999KJ"),
            ("5h 6s 7d 8c 9h Kd", Category::Straight, "98765"),
            ("Ah 2s 3d 4c 5h Kd", Category::Straight, "5432A"),
            ("2h 7h 9h Jh Kh As", Category::Flush, "KJ972"),
            ("9h 9s 9d Jc Jh", Category::FullHouse, "999JJ"),
            ("9h 9s 9d 9c Jh", Category::Quads, "9999J"),
            ("5h 6h 7h 8h 9h Th", Category::StraightFlush, "T9876"),
        ];
        for (text, category, expected) in cases {
            let hand = best(text);
            assert_eq!(hand.category, category, "{text}");
            assert_eq!(ranks(&hand), expected, "{text}");
        }
    }

    #[test]
    fn picks_the_best_of_many_cards() {
        // Two trips make a full house of the higher one.
        let hand = best("9h 9s 9d Kc Kh Ks 2c");
        assert_eq!(hand.category, Category::FullHouse);
        assert_eq!(ranks(&hand), "KKK99");
        // A flush beats the straight it also holds.
        let hand = best("5h 6s 7h 8h 9h 2h");
        assert_eq!(hand.category, Category::Flush);
        // Three pairs: the best two, and the best remaining kicker.
        let hand = best("2h 2s 9d 9c Kh Ks 3c");
        assert_eq!(ranks(&hand), "KK993");
        // A straight flush wins over four of a kind.
        assert_eq!(
            best("5h 6h 7h 8h 9h 9s 9d 9c").category,
            Category::StraightFlush
        );
    }

    #[test]
    fn short_hands_play_what_they_have() {
        let hand = best("Kh Ks 4d 2c");
        assert_eq!(hand.category, Category::Pair);
        assert_eq!(ranks(&hand), "KK42");
        assert_eq!(best("Ah 2h 3h 4h").category, Category::HighCard);
        assert!(best_hand(&[]).is_none());
    }

    #[test]
    fn hands_compare_by_category_then_rank() {
        assert!(best("2h 2s 3d 4c 5h") > best("Ah Ks Qd Jc 9h"));
        assert!(best("Kh Ks 4d 3c 2h") > best("Qh Qs Ad Kc Jh"));
        assert!(best("Kh Ks 5d 3c 2h") > best("Kd Kc 4d 3c 2h"));
        assert!(best("6h 5s 4d 3c 2h") > best("5h 4s 3d 2c Ah"));
        // A kicker is better than nothing.
        assert!(best("Kh Ks 2d") > best("Kd Kc"));
        // Suits never break a tie.
        assert_eq!(
            best("Kh Ks 5d 3c 2h").cmp(&best("Kd Kc 5h 3s 2d")),
            std::cmp::Ordering::Equal
        );
    }
}
