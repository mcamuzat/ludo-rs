use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{HornOfPlenty, Phase, State};
use crate::deck::Rank;

/// How often the redrawn card on the table is made to match its
/// declaration (when a hidden copy allows it): a uniform redraw would make
/// every declaration look like a bluff.
const TRUTH_PRIOR: f64 = 0.5;

impl Determinize for HornOfPlenty {
    /// The other face-down cards and the undealt cards are redrawn among
    /// the cards `observer` cannot place, with the card on the table while
    /// it is unrevealed and not the observer's: that card then matches its
    /// declaration half of the time, when it can.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let hidden_card =
            d.phase == Phase::Answer && d.table.as_ref().is_some_and(|t| t.poser != observer);
        let mut on_table: Vec<Rank> = match (&d.table, hidden_card) {
            (Some(t), true) => vec![t.card],
            _ => Vec::new(),
        };
        {
            let mut others: Vec<&mut Vec<Rank>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, hand)| hand)
                .collect();
            others.push(&mut on_table);
            d.supply.redeal_hidden_with(&[], &mut others, rng);
        }
        if let (Some(table), Some(&card)) = (d.table.as_mut(), on_table.first()) {
            table.card = card;
            if card != table.declared && rng.gen_bool(TRUTH_PRIOR) {
                // Swap with a hidden copy of the declared rank, in another
                // hand or else among the undealt cards.
                let copy = d
                    .hands
                    .iter_mut()
                    .enumerate()
                    .filter(|(i, _)| *i != o)
                    .find_map(|(_, hand)| hand.iter_mut().find(|c| **c == table.declared));
                let swapped = match copy {
                    Some(copy) => {
                        *copy = card;
                        true
                    }
                    None => d.supply.swap_hidden(table.declared, card),
                };
                if swapped {
                    table.card = table.declared;
                }
            }
        }
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::horn_of_plenty::testing::table;
    use crate::horn_of_plenty::{Action, Config, HornOfPlenty, Phase};
    use crate::testing::r;

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = HornOfPlenty::new(&Config::default(), 4, 3);
        let view = HornOfPlenty::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = HornOfPlenty::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(HornOfPlenty::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert_eq!(d.supply.deck_len(), s.supply.deck_len());
            assert_ne!(d.seed, s.seed);
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn the_card_on_the_table_is_redrawn_for_the_others_only() {
        let mut s = table(
            &[&[3, 9], &[5, 6], &[7, 8]],
            &[&[1], &[2], &[4]],
            &[10, 10, 10],
        );
        HornOfPlenty::apply(
            &mut s,
            Seat(0),
            &Action::Play {
                card: r(3),
                declared: r(9),
            },
        )
        .unwrap();
        assert_eq!(s.phase, Phase::Answer);
        let mut seen = Vec::new();
        for seed in 0..40 {
            let mine = HornOfPlenty::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(mine.table.as_ref().unwrap().card, r(3));
            let theirs = HornOfPlenty::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            seen.push(theirs.table.as_ref().unwrap().card);
        }
        assert!(seen.contains(&r(9)), "never true");
        assert!(seen.iter().any(|&c| c != r(9)), "always true");
    }

    #[test]
    fn a_declaration_can_come_true_from_the_undealt_cards() {
        // The only hidden 10 is undealt: a uniform redraw would show it on
        // the table a third of the time, the prior two thirds.
        let mut s = table(&[&[3], &[5], &[]], &[&[1], &[2], &[4]], &[10, 1]);
        HornOfPlenty::apply(
            &mut s,
            Seat(0),
            &Action::Play {
                card: r(3),
                declared: r(10),
            },
        )
        .unwrap();
        let true_ones = (0..300)
            .filter(|&seed| {
                let d = HornOfPlenty::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
                d.table.as_ref().unwrap().card == r(10)
            })
            .count();
        assert!(true_ones > 160, "{true_ones}");
    }
}
