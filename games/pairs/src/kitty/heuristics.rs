use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Kitty, Phase, State};
use crate::deck::{count_by_rank, Rank};

impl State {
    /// How much `seat` wants one more card of `rank`, from its own hand:
    /// the rank's value, more when it already holds some.
    fn want(&self, seat: Seat, rank: Rank) -> u32 {
        let held = count_by_rank(&self.hands[seat.index()])[rank.index()];
        u32::from(rank.value()) * (1 + u32::from(held))
    }
}

impl Heuristics for Kitty {
    /// From what `seat` sees only. Pass the card it wants least; bid its
    /// lowest card on a target it wants, a random one of its least wanted
    /// otherwise; take the card it wants most; give away the least wanted.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let hand = &s.hands[seat.index()];
        let least_wanted = || hand.iter().copied().min_by_key(|&c| (s.want(seat, c), c));
        let chosen = match s.phase {
            Phase::Pass => least_wanted().map(|card| Action::Pass { card }),
            Phase::Bid => {
                let prize: u32 = s.target.iter().map(|&c| s.want(seat, c)).max().unwrap_or(0);
                if prize >= 12 {
                    hand.iter().copied().min().map(|card| Action::Bid { card })
                } else {
                    let mut cards: Vec<Rank> = hand.clone();
                    cards.sort_by_key(|&c| (s.want(seat, c), c));
                    cards.truncate(3);
                    cards.choose(rng).map(|&card| Action::Bid { card })
                }
            }
            Phase::Claim => legal
                .iter()
                .max_by_key(|a| match a {
                    Action::Take { card } => s.want(seat, *card),
                    Action::Give { card, .. } => u32::MAX - u32::from(card.value()),
                    _ => 0,
                })
                .cloned(),
            Phase::Over => None,
        };
        match chosen {
            Some(Action::Give { card, .. }) => {
                let tos: Vec<&Action> = legal
                    .iter()
                    .filter(|a| matches!(a, Action::Give { card: c, .. } if *c == card))
                    .collect();
                tos.choose(rng).map(|&a| a.clone())
            }
            other => other,
        }
        .or_else(|| legal.choose(rng).cloned())
        .unwrap_or(Action::Pass {
            card: Rank::new(1).expect("1 is a rank"),
        })
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::kitty::{Config, Kitty};

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        for players in 3..=6u8 {
            for _ in 0..10 {
                let mut s = Kitty::new(&Config::default(), players, rng.gen());
                loop {
                    let pending = Kitty::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = Kitty::rollout_action(&s, seat, &mut rng);
                    assert!(
                        Kitty::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    Kitty::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
