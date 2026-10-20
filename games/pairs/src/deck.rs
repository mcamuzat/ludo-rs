//! The Pairs deck: one 1, two 2s, … ten 10s (55 cards), and the draw and
//! discard piles every Pairs game shares.

use ludo_core::Seat;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const DECK_SIZE: usize = 55;

/// A card. Cards of one rank are interchangeable: there are no suits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Rank(u8);

impl Rank {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 10;

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }

    /// Position in a per-rank table: 0 for the 1, 9 for the 10.
    pub fn index(self) -> usize {
        usize::from(self.0 - Self::MIN)
    }

    pub fn all() -> impl Iterator<Item = Rank> {
        (Self::MIN..=Self::MAX).map(Rank)
    }
}

impl TryFrom<u8> for Rank {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, String> {
        Rank::new(value).ok_or_else(|| format!("rang hors de 1–10 : {value}"))
    }
}

impl From<Rank> for u8 {
    fn from(rank: Rank) -> u8 {
        rank.0
    }
}

/// The 55 cards, in rank order.
pub fn full_deck() -> Vec<Rank> {
    Rank::all()
        .flat_map(|r| std::iter::repeat_n(r, usize::from(r.value())))
        .collect()
}

pub fn has_rank(cards: &[Rank], rank: Rank) -> bool {
    cards.contains(&rank)
}

pub fn lowest(cards: &[Rank]) -> Option<Rank> {
    cards.iter().copied().min()
}

/// How many cards of each rank (see `Rank::index`).
pub fn count_by_rank(cards: &[Rank]) -> [u8; 10] {
    let mut counts = [0u8; 10];
    for card in cards {
        counts[card.index()] += 1;
    }
    counts
}

/// The highest rank present at least twice.
pub fn pair_in(cards: &[Rank]) -> Option<Rank> {
    let counts = count_by_rank(cards);
    Rank::all().filter(|r| counts[r.index()] >= 2).max()
}

/// A card of an opening deal (see `open_round`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opening {
    Dealt {
        seat: Seat,
        card: Rank,
    },
    /// A tie-break card that would pair: discarded and replaced.
    Replaced {
        seat: Seat,
        card: Rank,
    },
    /// The discards became the draw pile.
    Reshuffled,
}

/// Draws a card, noting a reshuffle in `log`.
pub fn draw_logged(supply: &mut Supply, log: &mut Vec<Opening>) -> Option<Rank> {
    let before = supply.shuffles();
    let card = supply.draw();
    if supply.shuffles() != before {
        log.push(Opening::Reshuffled);
    }
    card
}

/// [R] A card for `seat` that does not pair `stack`: a pairing card is
/// discarded face up and replaced. `None` when the supply has none left.
pub fn draw_unpaired(
    supply: &mut Supply,
    stack: &mut Vec<Rank>,
    seat: Seat,
    log: &mut Vec<Opening>,
) -> Option<Rank> {
    for _ in 0..DECK_SIZE {
        let card = draw_logged(supply, log)?;
        if has_rank(stack, card) {
            supply.discard([card]);
            log.push(Opening::Replaced { seat, card });
        } else {
            stack.push(card);
            log.push(Opening::Dealt { seat, card });
            return Some(card);
        }
    }
    None
}

/// [R] The opening of a Pairs round: one face-up card to each of `seats`
/// (in `stacks`, indexed by seat); the lowest card goes first. Tied
/// players get one more card each, compared among themselves, until the
/// tie breaks; the extra cards stay in the stacks. Out of cards, the first
/// of the tied players goes first.
pub fn open_round(
    supply: &mut Supply,
    stacks: &mut [Vec<Rank>],
    seats: &[Seat],
    log: &mut Vec<Opening>,
) -> Seat {
    let lowest = |cards: &[(Seat, Rank)]| -> Vec<Seat> {
        let low = cards.iter().map(|&(_, card)| card).min();
        cards
            .iter()
            .filter(|&&(_, card)| Some(card) == low)
            .map(|&(seat, _)| seat)
            .collect()
    };
    let mut drawn: Vec<(Seat, Rank)> = Vec::new();
    for &seat in seats {
        if let Some(card) = draw_unpaired(supply, &mut stacks[seat.index()], seat, log) {
            drawn.push((seat, card));
        }
    }
    let mut tied = lowest(&drawn);
    while tied.len() > 1 {
        let mut round: Vec<(Seat, Rank)> = Vec::new();
        for &seat in &tied {
            match draw_unpaired(supply, &mut stacks[seat.index()], seat, log) {
                Some(card) => round.push((seat, card)),
                None => break,
            }
        }
        if round.len() < tied.len() {
            break;
        }
        tied = lowest(&round);
    }
    tied.first().or(seats.first()).copied().unwrap_or(Seat(0))
}

/// [R] The lowest of the face-up `cards` goes first. Tied seats draw one
/// more card each, compared among themselves, until the tie breaks; these
/// tie-break cards are discarded face up (unlike `open_round`, which keeps
/// them). Returns the leader and the tie-break cards, in draw order. Out of
/// cards, the first of the tied seats leads.
pub fn lowest_discarding_ties(
    supply: &mut Supply,
    cards: &[(Seat, Rank)],
) -> (Seat, Vec<(Seat, Rank)>) {
    let lowest = |cards: &[(Seat, Rank)]| -> Vec<Seat> {
        let low = cards.iter().map(|&(_, card)| card).min();
        cards
            .iter()
            .filter(|&&(_, card)| Some(card) == low)
            .map(|&(seat, _)| seat)
            .collect()
    };
    let mut drawn = Vec::new();
    let mut tied = lowest(cards);
    while tied.len() > 1 {
        let mut round = Vec::new();
        for &seat in &tied {
            match supply.draw() {
                Some(card) => round.push((seat, card)),
                None => break,
            }
        }
        supply.discard(round.iter().map(|&(_, card)| card));
        drawn.extend_from_slice(&round);
        if round.len() < tied.len() {
            break;
        }
        tied = lowest(&round);
    }
    let leader = tied
        .first()
        .or(cards.first().map(|(seat, _)| seat))
        .copied();
    (leader.unwrap_or(Seat(0)), drawn)
}

/// The draw pile and the discard pile. An empty draw pile is rebuilt from the
/// discards; `burn` cards then go face down to the discards.
///
/// The seed lives here so that a reshuffle replays identically from a match
/// log. Never put a `Supply` in a view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supply {
    /// Draw pile; the top is the last element.
    deck: Vec<Rank>,
    /// Discards everyone has seen.
    seen: Vec<Rank>,
    /// Discards nobody has seen (burned cards).
    unseen: Vec<Rank>,
    burn: u8,
    seed: u64,
    shuffles: u32,
}

impl Supply {
    /// Shuffles `cards` and burns.
    pub fn new(cards: Vec<Rank>, burn: u8, seed: u64) -> Self {
        let mut supply = Self::from_parts(Vec::new(), Vec::new(), cards, burn, seed);
        supply.restock();
        supply
    }

    /// Piles as given, without shuffling. The top of `deck` is its last element.
    pub fn from_parts(
        deck: Vec<Rank>,
        seen: Vec<Rank>,
        unseen: Vec<Rank>,
        burn: u8,
        seed: u64,
    ) -> Self {
        Self {
            deck,
            seen,
            unseen,
            burn,
            seed,
            shuffles: 0,
        }
    }

    fn restock(&mut self) {
        self.deck.append(&mut self.seen);
        self.deck.append(&mut self.unseen);
        // One independent stream per shuffle. ChaCha, not `SmallRng`: the
        // latter is a different generator on wasm32 and on 64-bit targets,
        // and a log recorded in the browser must replay natively.
        let stream = self.seed ^ u64::from(self.shuffles).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.deck.shuffle(&mut ChaCha8Rng::seed_from_u64(stream));
        self.shuffles += 1;
        // ❓ Never burn the last card: a reshuffle always leaves one to draw.
        let burned = usize::from(self.burn).min(self.deck.len().saturating_sub(1));
        let keep = self.deck.len() - burned;
        self.unseen.extend(self.deck.drain(keep..));
    }

    /// Every discard back into the draw pile, shuffled (with the burn).
    pub fn reshuffle(&mut self) {
        self.restock();
    }

    pub fn can_draw(&self) -> bool {
        !(self.deck.is_empty() && self.seen.is_empty() && self.unseen.is_empty())
    }

    /// The top card. `None` only when both piles are empty.
    pub fn draw(&mut self) -> Option<Rank> {
        if self.deck.is_empty() {
            if !self.can_draw() {
                return None;
            }
            self.restock();
        }
        self.deck.pop()
    }

    /// Face up: everyone has seen these cards.
    pub fn discard(&mut self, cards: impl IntoIterator<Item = Rank>) {
        self.seen.extend(cards);
    }

    /// Face down: nobody at the table has seen these cards (a player who
    /// has is the game's business, see `redeal_hidden_with`).
    pub fn discard_unseen(&mut self, cards: impl IntoIterator<Item = Rank>) {
        self.unseen.extend(cards);
    }

    /// For determinization: one copy of `wanted` in the draw pile or the
    /// face-down discards becomes `given`. `false` when there is none.
    pub fn swap_hidden(&mut self, wanted: Rank, given: Rank) -> bool {
        match self
            .deck
            .iter_mut()
            .chain(self.unseen.iter_mut())
            .find(|c| **c == wanted)
        {
            Some(card) => {
                *card = given;
                true
            }
            None => false,
        }
    }

    pub fn deck_len(&self) -> usize {
        self.deck.len()
    }

    pub fn discard_len(&self) -> usize {
        self.seen.len() + self.unseen.len()
    }

    pub fn seen(&self) -> &[Rank] {
        &self.seen
    }

    /// How many times the draw pile has been built (1 after `new`).
    pub fn shuffles(&self) -> u32 {
        self.shuffles
    }

    /// Per rank, the cards nobody can place: the draw pile and the burned cards.
    pub fn unlocated(&self) -> [u8; 10] {
        let mut counts = count_by_rank(&self.deck);
        for card in &self.unseen {
            counts[card.index()] += 1;
        }
        counts
    }

    /// Per rank, the cards the next draw may bring, from what everyone
    /// knows: `unlocated`, or the whole discard pile when the draw pile is
    /// empty and the next draw reshuffles it.
    pub fn draw_pool(&self) -> [u8; 10] {
        if !self.deck.is_empty() {
            return self.unlocated();
        }
        let mut counts = count_by_rank(&self.seen);
        for card in &self.unseen {
            counts[card.index()] += 1;
        }
        counts
    }

    /// For determinization in a game without hidden hands.
    pub fn redeal_hidden<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.redeal_hidden_with(&[], &mut [], rng);
    }

    /// For determinization: everything an observer cannot place is redrawn —
    /// the order of the draw pile, the face-down discards, and the `hands`
    /// given (the other players'), each keeping its size. `keep` lists
    /// face-down discards the observer has seen: they stay where they are.
    /// The seed of future shuffles is redrawn too.
    pub fn redeal_hidden_with<R: Rng + ?Sized>(
        &mut self,
        keep: &[Rank],
        hands: &mut [&mut Vec<Rank>],
        rng: &mut R,
    ) {
        let mut kept = Vec::new();
        for &card in keep {
            if let Some(at) = self.unseen.iter().position(|&c| c == card) {
                kept.push(self.unseen.swap_remove(at));
            }
        }
        let face_down = self.unseen.len();
        self.deck.append(&mut self.unseen);
        let sizes: Vec<usize> = hands.iter().map(|hand| hand.len()).collect();
        for hand in hands.iter_mut() {
            self.deck.append(hand);
        }
        self.deck.shuffle(rng);
        for (hand, size) in hands.iter_mut().zip(sizes) {
            let at = self.deck.len() - size;
            hand.extend(self.deck.drain(at..));
        }
        let at = self.deck.len() - face_down;
        self.unseen.extend(self.deck.drain(at..));
        self.unseen.append(&mut kept);
        self.seed = rng.next_u64();
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::SmallRng;

    use super::*;

    fn ranks(values: &[u8]) -> Vec<Rank> {
        values.iter().map(|&v| Rank::new(v).unwrap()).collect()
    }

    #[test]
    fn tie_break_cards_are_discarded_until_the_tie_breaks() {
        // Top of the deck last: the tied seats 0 and 2 draw 4 and 4, then
        // 6 and 3.
        let mut supply = Supply::from_parts(ranks(&[3, 6, 4, 4]), Vec::new(), Vec::new(), 0, 0);
        let up = [(Seat(0), Rank(2)), (Seat(1), Rank(5)), (Seat(2), Rank(2))];
        let (leader, drawn) = lowest_discarding_ties(&mut supply, &up);
        assert_eq!(leader, Seat(2));
        let values: Vec<u8> = drawn.iter().map(|&(_, c)| c.value()).collect();
        assert_eq!(values, vec![4, 4, 6, 3]);
        assert_eq!(supply.seen().len(), 4);
        // No tie: nothing drawn.
        let (leader, drawn) = lowest_discarding_ties(&mut supply, &up[..2]);
        assert_eq!((leader, drawn.len()), (Seat(0), 0));
    }

    #[test]
    fn the_deck_has_n_cards_of_rank_n() {
        let deck = full_deck();
        assert_eq!(deck.len(), DECK_SIZE);
        let counts = count_by_rank(&deck);
        assert_eq!(counts, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn ranks_outside_1_to_10_do_not_deserialize() {
        assert!(Rank::new(0).is_none());
        assert!(Rank::new(11).is_none());
        assert!(serde_json::from_str::<Rank>("0").is_err());
        assert!(serde_json::from_str::<Rank>("11").is_err());
        assert_eq!(
            serde_json::from_str::<Rank>("7").unwrap(),
            Rank::new(7).unwrap()
        );
        assert_eq!(serde_json::to_string(&Rank::new(7).unwrap()).unwrap(), "7");
    }

    #[test]
    fn helpers_read_a_stack() {
        let stack = ranks(&[9, 3, 6]);
        assert!(has_rank(&stack, Rank::new(3).unwrap()));
        assert!(!has_rank(&stack, Rank::new(4).unwrap()));
        assert_eq!(lowest(&stack), Rank::new(3));
        assert_eq!(lowest(&[]), None);
    }

    #[test]
    fn a_new_supply_burns_five_cards() {
        let s = Supply::new(full_deck(), 5, 1);
        assert_eq!(s.deck_len(), 50);
        assert_eq!(s.discard_len(), 5);
        assert!(s.seen().is_empty());
        assert_eq!(s.shuffles(), 1);
    }

    #[test]
    fn the_same_seed_gives_the_same_order() {
        let mut a = Supply::new(full_deck(), 5, 42);
        let mut b = Supply::new(full_deck(), 5, 42);
        let mut c = Supply::new(full_deck(), 5, 43);
        let (da, db, dc): (Vec<_>, Vec<_>, Vec<_>) = (
            (0..50).map(|_| a.draw()).collect(),
            (0..50).map(|_| b.draw()).collect(),
            (0..50).map(|_| c.draw()).collect(),
        );
        assert_eq!(da, db);
        assert_ne!(da, dc);
    }

    /// The order comes from ChaCha8, identical on every platform. A change
    /// here breaks the replay of every recorded match.
    #[test]
    fn the_deal_of_a_seed_is_pinned() {
        let mut s = Supply::new(full_deck(), 5, 42);
        let dealt: Vec<u8> = (0..10).map(|_| s.draw().unwrap().value()).collect();
        assert_eq!(dealt, vec![7, 8, 8, 5, 8, 5, 10, 10, 7, 8]);
    }

    #[test]
    fn an_empty_deck_is_rebuilt_from_the_discards() {
        let mut s = Supply::new(full_deck(), 5, 7);
        let drawn: Vec<Rank> = (0..50).map(|_| s.draw().unwrap()).collect();
        s.discard(drawn);
        // 55 cards in the discard: reshuffle, burn 5, draw 1.
        assert!(s.draw().is_some());
        assert_eq!(s.shuffles(), 2);
        assert_eq!(s.deck_len(), 49);
        assert_eq!(s.discard_len(), 5);
    }

    #[test]
    fn a_small_discard_still_yields_a_card() {
        let mut s = Supply::from_parts(Vec::new(), ranks(&[7, 7]), Vec::new(), 5, 1);
        assert_eq!(s.draw(), Rank::new(7));
        assert_eq!(s.deck_len() + s.discard_len(), 1);
    }

    #[test]
    fn nothing_left_means_no_card() {
        let mut s = Supply::new(Vec::new(), 5, 1);
        assert!(!s.can_draw());
        assert_eq!(s.draw(), None);
    }

    #[test]
    fn from_parts_draws_the_last_card_first() {
        let mut s = Supply::from_parts(ranks(&[2, 5, 9]), Vec::new(), Vec::new(), 0, 0);
        assert_eq!(s.draw(), Rank::new(9));
        assert_eq!(s.draw(), Rank::new(5));
    }

    #[test]
    fn unlocated_counts_the_deck_and_the_burned_cards_only() {
        let mut s = Supply::new(full_deck(), 5, 3);
        assert_eq!(s.unlocated(), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let card = s.draw().unwrap();
        s.discard([card]);
        let mut expected = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        expected[card.index()] -= 1;
        assert_eq!(s.unlocated(), expected);
        assert_eq!(s.seen(), &[card]);
    }

    #[test]
    fn redealing_keeps_what_everyone_knows() {
        let mut s = Supply::new(full_deck(), 5, 3);
        let card = s.draw().unwrap();
        s.discard([card]);
        let before = s.clone();
        let mut moved = false;
        for seed in 0..10 {
            let mut d = before.clone();
            d.redeal_hidden(&mut SmallRng::seed_from_u64(seed));
            assert_eq!(d.deck_len(), before.deck_len());
            assert_eq!(d.discard_len(), before.discard_len());
            assert_eq!(d.seen(), before.seen());
            assert_eq!(d.unlocated(), before.unlocated());
            assert_eq!(d.shuffles(), before.shuffles());
            moved |= d != before;
        }
        assert!(moved, "the hidden cards never moved");
    }

    #[test]
    fn pair_in_finds_the_highest_pair() {
        assert_eq!(pair_in(&ranks(&[3, 7, 3, 9, 9])), Rank::new(9));
        assert_eq!(pair_in(&ranks(&[3, 7, 9])), None);
        assert_eq!(pair_in(&[]), None);
    }

    #[test]
    fn face_down_discards_stay_unlocated() {
        let mut s = Supply::from_parts(ranks(&[2]), Vec::new(), Vec::new(), 0, 0);
        s.discard_unseen(ranks(&[7, 7]));
        assert!(s.seen().is_empty());
        assert_eq!(s.discard_len(), 2);
        assert_eq!(s.unlocated()[6], 2);
    }

    #[test]
    fn redealing_with_hands_keeps_sizes_and_cards() {
        let before =
            Supply::from_parts(ranks(&[1, 2, 3, 4]), ranks(&[9]), ranks(&[10, 10, 5]), 0, 0);
        let mut moved = false;
        for seed in 0..20 {
            let mut s = before.clone();
            let mut a = ranks(&[6, 7]);
            let mut b = ranks(&[8]);
            s.redeal_hidden_with(
                &ranks(&[10, 10]),
                &mut [&mut a, &mut b],
                &mut SmallRng::seed_from_u64(seed),
            );
            assert_eq!((a.len(), b.len()), (2, 1));
            assert_eq!(s.deck_len(), 4);
            assert_eq!(s.discard_len(), 4);
            assert_eq!(s.seen(), before.seen());
            // The two 10s the observer saw are still face down in the discards.
            assert_eq!(s.unseen.iter().filter(|c| c.value() == 10).count(), 2);
            assert!(!a.iter().chain(&b).any(|c| c.value() == 10));
            let mut all: Vec<Rank> = s
                .deck
                .iter()
                .chain(&s.unseen)
                .chain(&a)
                .chain(&b)
                .copied()
                .collect();
            all.sort_unstable();
            assert_eq!(all, ranks(&[1, 2, 3, 4, 5, 6, 7, 8, 10, 10]));
            moved |= a != ranks(&[6, 7]);
        }
        assert!(moved, "the hands never changed");
    }
}
