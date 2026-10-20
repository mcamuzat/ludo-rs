use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{card_points, Action, HornOfPlenty, Phase, State};
use crate::deck::Rank;

/// How often the rollout declares the true rank.
const TRUTHFUL: f64 = 0.5;
/// How often the rollout leaves an answer to the next player.
const PASSES: f64 = 0.2;

impl State {
    /// Per rank, the copies `seat` cannot place: not face up, thrown away,
    /// nor in its own hand (nor on the table when it played that card).
    fn unseen_for(&self, seat: Seat) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mine = self
            .table
            .as_ref()
            .filter(|t| t.poser == seat || self.phase == Phase::Place)
            .map(|t| t.card);
        for card in self
            .shown
            .iter()
            .flatten()
            .chain(self.supply.seen())
            .chain(&self.hands[seat.index()])
            .chain(mine.iter())
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// What the card is worth in the face-up hand of `to`: its rank, or
    /// minus the rank it would throw away with its pair.
    fn gain(&self, to: Seat, card: Rank) -> i32 {
        let value = i32::from(card.value());
        if self.shown[to.index()].contains(&card) {
            -value
        } else {
            value
        }
    }

    /// The best place for `card` as `seat` sees it: its own gain, or the
    /// loss of another seat (counted more against the stronger hands).
    fn best_recipient(&self, seat: Seat, card: Rank, recipients: &[Seat]) -> Option<Seat> {
        let lead = |to: Seat| {
            card_points(&self.shown[to.index()]) as i32 + 10 * i32::from(self.scores[to.index()])
        };
        recipients.iter().copied().max_by_key(|&to| {
            if to == seat {
                (self.gain(to, card), 0)
            } else {
                (-self.gain(to, card), lead(to))
            }
        })
    }
}

impl Heuristics for HornOfPlenty {
    /// From what `seat` sees only, never the card on the table it did not
    /// play. Declare: a random card, the true rank half of the time, else a
    /// rank with unseen copies. Answer: disagree with an impossible rank,
    /// otherwise agree, disagree or pass at random. Place: where the card
    /// helps `seat` most or hurts the leader most.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        match s.phase {
            Phase::Declare => {
                let Some(&card) = s.hands[seat.index()].choose(rng) else {
                    return Action::Pass;
                };
                if rng.gen_bool(TRUTHFUL) {
                    return Action::Play {
                        card,
                        declared: card,
                    };
                }
                let unseen = s.unseen_for(seat);
                let plausible: Vec<Rank> = Rank::all()
                    .filter(|&r| r != card && unseen[r.index()] > 0)
                    .collect();
                let declared = plausible.choose(rng).copied().unwrap_or(card);
                Action::Play { card, declared }
            }
            Phase::Answer => {
                let Some(table) = s.table.as_ref() else {
                    return Action::Pass;
                };
                if s.unseen_for(seat)[table.declared.index()] == 0 {
                    return Action::Disagree;
                }
                if rng.gen_bool(PASSES) {
                    Action::Pass
                } else if rng.gen_bool(0.5) {
                    Action::Agree
                } else {
                    Action::Disagree
                }
            }
            Phase::Place => {
                let card = s.table.as_ref().map(|t| t.card);
                card.and_then(|card| s.best_recipient(seat, card, &s.recipients))
                    .map_or(Action::Pass, |to| Action::Give { to })
            }
            Phase::Over => Action::Pass,
        }
    }

    /// Declare: for each card, the truth and one bluff (the rank with the
    /// most unseen copies): the search cannot tell bluffs apart anyway.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        if s.phase != Phase::Declare {
            return legal;
        }
        let unseen = s.unseen_for(seat);
        let mut cards = s.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        let narrowed: Vec<Action> = cards
            .into_iter()
            .flat_map(|card| {
                let bluff = Rank::all()
                    .filter(|&r| r != card)
                    .max_by_key(|&r| (unseen[r.index()], r))
                    .unwrap_or(card);
                [
                    Action::Play {
                        card,
                        declared: card,
                    },
                    Action::Play {
                        card,
                        declared: bluff,
                    },
                ]
            })
            .collect();
        if narrowed.is_empty() {
            legal
        } else {
            narrowed
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::horn_of_plenty::testing::table;
    use crate::horn_of_plenty::{Action, Config, HornOfPlenty, State};
    use crate::testing::r;

    fn rollout(s: &State, seat: u8) -> Action {
        HornOfPlenty::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(1))
    }

    fn played(card: u8, declared: u8) -> State {
        let mut s = table(
            &[&[card, 9], &[5, 6], &[7, 8]],
            &[&[2], &[3], &[4]],
            &[10, 10],
        );
        HornOfPlenty::apply(
            &mut s,
            Seat(0),
            &Action::Play {
                card: r(card),
                declared: r(declared),
            },
        )
        .unwrap();
        s
    }

    #[test]
    fn disagrees_with_a_rank_it_can_see_is_gone() {
        // The only 1 is face up at seat 2.
        let mut s = played(6, 1);
        s.shown[2] = vec![r(1)];
        assert_eq!(rollout(&s, 1), Action::Disagree);
    }

    #[test]
    fn keeps_a_good_card_and_gives_a_pairing_one() {
        let mut s = played(6, 6);
        HornOfPlenty::apply(&mut s, Seat(1), &Action::Agree).unwrap();
        // Seat 1 was right: it takes the 6, which pairs nobody.
        assert_eq!(rollout(&s, 1), Action::Give { to: Seat(1) });
        let mut s = played(3, 3);
        HornOfPlenty::apply(&mut s, Seat(1), &Action::Agree).unwrap();
        // The 3 would pair seat 1's 3: it goes to seat 0.
        assert_eq!(rollout(&s, 1), Action::Give { to: Seat(0) });
    }

    #[test]
    fn search_considers_the_truth_and_one_bluff_per_card() {
        let s = table(&[&[3, 9], &[5, 6]], &[&[2], &[4]], &[10]);
        let actions = HornOfPlenty::search_actions(&s, Seat(0));
        assert_eq!(actions.len(), 4);
        assert!(actions.contains(&Action::Play {
            card: r(3),
            declared: r(3)
        }));
        assert!(actions.contains(&Action::Play {
            card: r(9),
            declared: r(9)
        }));
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 2..=6u8 {
            for _ in 0..5 {
                let mut s = HornOfPlenty::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = HornOfPlenty::pending(&s).0.first() {
                    let action = HornOfPlenty::rollout_action(&s, seat, &mut rng);
                    assert!(
                        HornOfPlenty::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    for a in HornOfPlenty::search_actions(&s, seat) {
                        assert!(HornOfPlenty::legal_actions(&s, seat).contains(&a), "{a:?}");
                    }
                    HornOfPlenty::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
