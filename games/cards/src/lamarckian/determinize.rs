use ludo_core::{Determinize, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{pack, Lamarckian, State};
use crate::deck::Card;

impl Determinize for Lamarckian {
    /// The other players' secret marks are forgotten; the cards of their
    /// hands that nobody saw them take, and the deck, are redrawn at their
    /// sizes among the cards `observer` cannot place.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let mut unseen = pack(s.players);
        let others_known = s
            .known
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != o)
            .flat_map(|(_, k)| k);
        let table = s
            .waiting
            .iter()
            .map(|(_, c)| c)
            .chain(s.picking.as_ref().map(|(_, c)| c));
        for seen in s.hands[o]
            .iter()
            .chain(&s.pool)
            .chain(&s.discard)
            .chain(others_known)
            .chain(table)
        {
            if let Some(at) = unseen.iter().position(|c| c == seen) {
                unseen.swap_remove(at);
            }
        }
        unseen.shuffle(rng);
        let mut deal = |n: usize| -> Vec<Card> { unseen.split_off(unseen.len().saturating_sub(n)) };
        for i in (0..usize::from(s.players)).filter(|&i| i != o) {
            let hidden = s.hands[i].len() - s.known[i].len();
            let mut hand = s.known[i].clone();
            hand.extend(deal(hidden));
            d.hands[i] = hand;
            d.chosen[i] = None;
        }
        d.deck = deal(d.deck.len());
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::lamarckian::testing::table;
    use crate::lamarckian::{Action, Config, Lamarckian};
    use crate::president::testing::c;

    #[test]
    fn hidden_cards_and_marks_change_but_not_what_the_observer_sees() {
        let mut s = Lamarckian::new(&Config::default(), 4, 3);
        let card = s.hands[0][0];
        Lamarckian::apply(&mut s, Seat(0), &Action::Mark { card }).unwrap();
        let view = Lamarckian::view(&s, Viewer::Seat(Seat(1)));
        let mut decks = Vec::new();
        for seed in 0..20 {
            let d = Lamarckian::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Lamarckian::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert_eq!(d.chosen[0], None);
            decks.push(d.deck.clone());
        }
        decks.dedup();
        assert!(decks.len() > 1);
    }

    #[test]
    fn cards_seen_taken_stay_in_their_hand() {
        // Seat 0's king of spades captures the spades of the pool.
        let mut s = table(&["Ks 2h 3h 4h", "5d 6d 7d 8d"], "As 9s 9c 9h", "");
        Lamarckian::apply(&mut s, Seat(0), &Action::Mark { card: c("Ks") }).unwrap();
        Lamarckian::apply(&mut s, Seat(1), &Action::Mark { card: c("5d") }).unwrap();
        for seed in 0..20 {
            let d = Lamarckian::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert!(d.hands[0].contains(&c("As")));
            assert!(d.hands[0].contains(&c("9s")));
            assert_eq!(d.hands[0].len(), s.hands[0].len());
        }
    }
}
