use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{rank_value, Action, Collusion, Phrase, State};
use crate::deck::{Card, Suit};
use crate::standings;

impl State {
    /// The tricks `seat` expects to end with: those taken plus its cards
    /// above every card of their suit still out in other hands. From what
    /// `seat` sees: the cards still in play are the pack minus those played.
    fn estimated_tricks(&self, seat: Seat) -> u8 {
        let mine = &self.hands[seat.index()];
        let strength = |c: &Card| c.rank().map(rank_value);
        let masters: usize = Suit::ALL
            .iter()
            .map(|&suit| {
                let best_out = self
                    .hands
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i != seat.index())
                    .flat_map(|(_, h)| h.iter())
                    .filter(|c| c.suit() == Some(suit))
                    .filter_map(|c| c.rank())
                    .map(rank_value)
                    .max();
                mine.iter()
                    .filter(|c| c.suit() == Some(suit) && strength(c) > best_out)
                    .count()
            })
            .sum();
        let range = self.aim_range(seat);
        (self.taken[seat.index()] + masters as u8).clamp(*range.start(), *range.end())
    }

    /// The seat's latest "I'm aiming for" this deal.
    fn last_aim(&self, seat: Seat) -> Option<u8> {
        self.talk
            .iter()
            .rev()
            .find_map(|&(s, phrase)| match phrase {
                Phrase::Aim { n } if s == seat => Some(n),
                _ => None,
            })
    }

    fn plays(&self, seat: Seat) -> Vec<Action> {
        if self.turn != Some(seat) {
            return Vec::new();
        }
        self.playable(seat)
            .into_iter()
            .map(|card| Action::Play { card })
            .collect()
    }
}

impl Heuristics for Collusion {
    /// Any playable card: the targets shift too much for a fixed policy.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        *s.plays(seat)
            .choose(rng)
            .expect("a pending seat has a card to play")
    }

    /// A simulation stops once the deal it started in is scored: the next
    /// deal is chance.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.deal_no > root.deal_no).then(|| Outcome::Ranked(standings(&sim.scores)).rewards())
    }

    /// The search only picks cards: talk changes no score.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.plays(seat)
    }

    /// Sincere but free: "I'm aiming for N" whenever the estimate moved
    /// since the seat last said it.
    fn scripted_action(s: &State, seat: Seat) -> Option<Action> {
        let n = s.estimated_tricks(seat);
        let aim = Action::Say {
            phrase: Phrase::Aim { n },
        };
        (s.last_aim(seat) != Some(n) && Self::legal_actions(s, seat).contains(&aim)).then_some(aim)
    }

    fn random_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.plays(seat)
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    use crate::collusion::testing::table;
    use crate::collusion::{Action, Collusion, Config, Phrase};

    #[test]
    fn ai_announces_when_its_estimate_changes() {
        // Seat 0 holds the two top spades and nothing else that wins.
        let mut s = table(
            ["As Ks 2h 3h", "Qs 4h 5h 6h", "Js 7h 8h 9h", "Ts Th Jh Qh"],
            0,
        );
        let said = Collusion::scripted_action(&s, Seat(0));
        assert_eq!(
            said,
            Some(Action::Say {
                phrase: Phrase::Aim { n: 2 }
            })
        );
        Collusion::apply(&mut s, Seat(0), &said.unwrap()).unwrap();
        assert_eq!(Collusion::scripted_action(&s, Seat(0)), None, "said once");
        Collusion::apply(
            &mut s,
            Seat(0),
            &Action::Play {
                card: crate::collusion::testing::c("2h"),
            },
        )
        .unwrap();
        // Seat 1's estimate: no master card.
        assert_eq!(
            Collusion::scripted_action(&s, Seat(1)),
            Some(Action::Say {
                phrase: Phrase::Aim { n: 0 }
            })
        );
    }

    #[test]
    fn ai_stays_quiet_when_its_estimate_holds() {
        let mut s = table(
            ["As Ks 2h 3h", "Qs 4h 5h 6h", "Js 7h 8h 9h", "Ts Th Jh Qh"],
            0,
        );
        s.talk.push((Seat(0), Phrase::Aim { n: 2 }));
        assert_eq!(Collusion::scripted_action(&s, Seat(0)), None);
    }

    #[test]
    fn rollout_and_random_moves_never_talk() {
        let mut rng = SmallRng::seed_from_u64(5);
        let mut s = Collusion::new(&Config::default(), 4, 11);
        for _ in 0..200 {
            let Some(seat) = s.turn else { break };
            let rollout = Collusion::rollout_action(&s, seat, &mut rng);
            assert!(matches!(rollout, Action::Play { .. }));
            assert!(Collusion::random_actions(&s, seat)
                .iter()
                .all(|a| matches!(a, Action::Play { .. })));
            assert!(Collusion::search_actions(&s, seat)
                .iter()
                .all(|a| matches!(a, Action::Play { .. })));
            let any = *Collusion::legal_actions(&s, seat).choose(&mut rng).unwrap();
            Collusion::apply(&mut s, seat, &any).unwrap();
        }
    }
}
