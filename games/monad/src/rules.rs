//! Transactions with the table: trades (wild cards and bonus included), buys
//! and leaps, and the condition under which a player may pass.

use ludo_core::Seat;

use crate::{Card, Color, Rank, State, Target};

/// [M] What two cards trade for, and whether a wild card stands in: one
/// warm and one cool card of the same rank, or a wild card (a Bi to Quint of
/// the seat's Identity colour) with a card of another rank, which sets the
/// rank of the trade. `None` when the pair does not trade.
pub(crate) fn trade_target(s: &State, seat: Seat, cards: [Card; 2]) -> Option<(Target, bool)> {
    let [a, b] = cards;
    if a.color.warm() == b.color.warm() {
        return None;
    }
    let identity = s.identities[seat.index()];
    let wild = |c: Card| c.color == identity && c.rank != Rank::Unit;
    let (target, joker) = if a.rank == b.rank {
        (a.rank.next(), false)
    } else if wild(a) {
        (b.rank.next(), true)
    } else if wild(b) {
        (a.rank.next(), true)
    } else {
        return None;
    };
    s.available(target).then_some((target, joker))
}

/// [M] The trades `seat` can make, each pair once, in sorted order.
pub(crate) fn trades(s: &State, seat: Seat) -> Vec<[Card; 2]> {
    let hand = &s.hands[seat.index()];
    let mut pairs: Vec<[Card; 2]> = Vec::new();
    for (i, &a) in hand.iter().enumerate() {
        for &b in &hand[i + 1..] {
            let pair = [a, b];
            if !pairs.contains(&pair) && trade_target(s, seat, pair).is_some() {
                pairs.push(pair);
            }
        }
    }
    pairs
}

/// [M] The rank whose bonus a trade opens: no wild card, Bi or higher, the
/// colours of a bonus combination, no bonus yet this turn, and (site
/// choice) at least one part of the bonus on the table.
pub(crate) fn bonus_of(s: &State, cards: [Card; 2], joker: bool) -> Option<Rank> {
    let [a, b] = cards;
    let rank = a.rank;
    if joker || rank == Rank::Unit || a.color.bonus_partner() != b.color || s.turn.bonus_used {
        return None;
    }
    let something = !s.deck.is_empty()
        || Rank::ALL[1..rank.index()]
            .iter()
            .any(|&r| s.available(Target::Card(r)));
    something.then_some(rank)
}

const TARGETS: [Target; 5] = [
    Target::Card(Rank::Bi),
    Target::Card(Rank::Tri),
    Target::Card(Rank::Quad),
    Target::Card(Rank::Quint),
    Target::Monad,
];

/// [M] The buys `seat` can make: two cards or more, all of a lower rank than
/// the target, worth at least its value. Site choice: no superfluous card,
/// so taking any card out would fall short.
pub(crate) fn buys(s: &State, seat: Seat) -> Vec<(Target, Vec<Card>)> {
    let hand = &s.hands[seat.index()];
    let mut out = Vec::new();
    for target in TARGETS.into_iter().filter(|&t| s.available(t)) {
        // Distinct cards, highest value first (the hand is sorted), with how
        // many are held.
        let mut groups: Vec<(Card, usize)> = Vec::new();
        for &card in hand.iter().rev().filter(|c| target.above(c.rank)) {
            match groups.iter_mut().find(|(c, _)| *c == card) {
                Some((_, n)) => *n += 1,
                None => groups.push((card, 1)),
            }
        }
        let mut chosen = Vec::new();
        minimal_payments(&groups, 0, target.value(), 0, &mut chosen, &mut |cards| {
            if cards.len() >= 2 {
                let mut cards = cards.to_vec();
                cards.sort();
                out.push((target, cards));
            }
        });
    }
    out
}

/// Enumerates the sub-multisets of `groups` (sorted by value, highest first)
/// that reach `value` only with their last, smallest card: exactly the
/// payments without a superfluous card.
fn minimal_payments(
    groups: &[(Card, usize)],
    from: usize,
    value: u32,
    total: u32,
    chosen: &mut Vec<Card>,
    found: &mut dyn FnMut(&[Card]),
) {
    let rest: u32 = groups[from..]
        .iter()
        .map(|&(c, n)| c.rank.value() * u32::try_from(n).expect("small counts"))
        .sum();
    if total + rest < value {
        return;
    }
    for (i, &(card, n)) in groups.iter().enumerate().skip(from) {
        // One copy of `card` or more, then only later groups.
        let mut sum = total;
        let mut taken = 0;
        while taken < n {
            chosen.push(card);
            taken += 1;
            sum += card.rank.value();
            if sum >= value {
                found(chosen);
                break;
            }
            minimal_payments(groups, i + 1, value, sum, chosen, found);
        }
        chosen.truncate(chosen.len() - taken);
    }
}

/// [M] Four, five or six Units of different colours for the top Tri, Quad or
/// Quint, only after a trade or a buy this turn.
pub(crate) fn leaps(s: &State, seat: Seat) -> Vec<Vec<Color>> {
    if !s.turn.prepared {
        return Vec::new();
    }
    let mut colors: Vec<Color> = s.hands[seat.index()]
        .iter()
        .filter(|c| c.rank == Rank::Unit)
        .map(|c| c.color)
        .collect();
    colors.dedup();
    let mut out = Vec::new();
    for (size, rank) in [(4, Rank::Tri), (5, Rank::Quad), (6, Rank::Quint)] {
        if colors.len() < size || !s.available(Target::Card(rank)) {
            continue;
        }
        // Every subset of `size` colours, in sorted order.
        for mask in 0u32..(1 << colors.len()) {
            if mask.count_ones() as usize == size {
                out.push(
                    colors
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| mask & (1 << i) != 0)
                        .map(|(_, &c)| c)
                        .collect(),
                );
            }
        }
    }
    out
}

/// The rank a leap of `n` Units brings.
pub(crate) fn leap_rank(n: usize) -> Option<Rank> {
    match n {
        4 => Some(Rank::Tri),
        5 => Some(Rank::Quad),
        6 => Some(Rank::Quint),
        _ => None,
    }
}

/// [M] Passing is allowed only when the seat can neither trade, draw nor
/// flip, nor buy a card or a Monad with two cards of the set just below and
/// enough others to reach its value.
pub(crate) fn may_pass(s: &State, seat: Seat) -> bool {
    if !s.deck.is_empty() || !s.discard.is_empty() || !trades(s, seat).is_empty() {
        return false;
    }
    let hand = &s.hands[seat.index()];
    !TARGETS.into_iter().filter(|&t| s.available(t)).any(|t| {
        let below = hand.iter().filter(|c| c.rank == t.below()).count();
        let worth: u32 = hand
            .iter()
            .filter(|c| t.above(c.rank))
            .map(|c| c.rank.value())
            .sum();
        below >= 2 && worth >= t.value()
    })
}
