use cards::deck::redeal_where;
use ludo_core::{Determinize, Seat};
use rand::Rng;

use crate::{Card, Phase, State, Volltreffer};

impl Determinize for Volltreffer {
    /// The cards `observer` cannot see (the other hands and the deck) are
    /// redealt among them, sizes kept. Stay where they are: the cards just
    /// seen hired and those shown in phase 2. Nobody gets a tribe they
    /// failed to follow, nor more of a value they showed a whole group of.
    /// The seed of the next rounds is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let others: Vec<usize> = (0..s.n()).filter(|&i| i != observer.index()).collect();
        let mut pinned: Vec<Vec<Card>> = vec![Vec::new(); s.n()];
        for hire in s.recent_hires() {
            pinned[hire.seat.index()].push(hire.card);
        }
        for &i in &others {
            if let Some(p) = s.visible_publication(i) {
                for &card in &p.cards {
                    if !pinned[i].contains(&card) {
                        pinned[i].push(card);
                    }
                }
            }
        }
        let mut free: Vec<Vec<Card>> = others
            .iter()
            .map(|&i| {
                s.hands[i]
                    .iter()
                    .copied()
                    .filter(|c| !pinned[i].contains(c))
                    .collect()
            })
            .collect();
        let mut deck = std::mem::take(&mut d.deck);
        {
            let mut piles: Vec<&mut Vec<Card>> = free.iter_mut().collect();
            piles.push(&mut deck);
            let groups: Vec<Vec<u8>> = others
                .iter()
                .map(|&i| match (s.phase, &s.publications[i]) {
                    (Phase::Publish, Some(p)) => p.groups.clone(),
                    _ => Vec::new(),
                })
                .collect();
            let voids: Vec<[bool; 5]> = others.iter().map(|&i| s.voids[i]).collect();
            let k = others.len();
            redeal_where(
                &mut piles,
                &|pile, card: Card| {
                    pile == k
                        || (!voids[pile][card.tribe.index()] && !groups[pile].contains(&card.value))
                },
                rng,
            );
        }
        for (slot, &i) in others.iter().enumerate() {
            let mut hand = pinned[i].clone();
            hand.append(&mut free[slot]);
            hand.sort();
            d.hands[i] = hand;
        }
        d.deck = deck;
        d.seed = rng.gen();
        d
    }
}
