//! What the seats' public moves say about who the Captain is, for the AI.
//! Only the kind of each move and who made it is read, never a hidden card.

use ludo_core::Seat;

use super::{Event, State};

/// A seat's moves this round that betray its role. A Captain passes cards
/// to run out of them; a Mutineer asks for cards and orders, to gather
/// points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tells {
    pub captain: u32,
    pub mutineer: u32,
}

impl State {
    pub fn tells(&self, seat: Seat) -> Tells {
        let mut tells = Tells::default();
        for event in &self.log {
            match *event {
                Event::Passed { from, .. } if from == seat => tells.captain += 1,
                Event::Asked { from, .. } if from == seat => tells.mutineer += 1,
                Event::Ordered {
                    seat: by,
                    belay: false,
                    ..
                } if by == seat => tells.mutineer += 1,
                _ => {}
            }
        }
        tells
    }

    /// How likely the tells of every seat but `observer` are, were the
    /// hands as they are: each move that does not fit a seat's role (a
    /// Captain who asked or ordered, a Mutineer who passed) counts against.
    pub fn tells_likelihood(&self, observer: Seat) -> f64 {
        let captains = self.captains();
        (0..self.players)
            .map(Seat)
            .filter(|&x| x != observer)
            .map(|x| {
                let tells = self.tells(x);
                let misfits = if captains.contains(&x) {
                    tells.mutineer
                } else {
                    tells.captain
                };
                MISFIT.powi(misfits as i32)
            })
            .product()
    }
}

/// The weight of each move that does not fit a seat's role.
const MISFIT: f64 = 0.3;

#[cfg(test)]
mod tests {
    use ludo_core::Seat;

    use crate::captains_orders::testing::table;
    use crate::captains_orders::Event;
    use crate::testing::r;

    use super::Tells;

    #[test]
    fn passing_betrays_a_captain_asking_and_ordering_a_mutineer() {
        let mut s = table(&[&[2, 9], &[5, 6], &[7, 8], &[3, 4]], &[]);
        s.log = vec![
            Event::Passed {
                from: Seat(1),
                to: Seat(2),
                card: Some(r(5)),
            },
            Event::Asked {
                from: Seat(2),
                to: Seat(3),
            },
            Event::Ordered {
                seat: Seat(2),
                card: r(8),
                belay: false,
            },
            // A belay says nothing: both roles have reasons to.
            Event::Ordered {
                seat: Seat(3),
                card: r(3),
                belay: true,
            },
        ];
        assert_eq!(
            s.tells(Seat(1)),
            Tells {
                captain: 1,
                mutineer: 0
            }
        );
        assert_eq!(
            s.tells(Seat(2)),
            Tells {
                captain: 0,
                mutineer: 2
            }
        );
        assert_eq!(s.tells(Seat(3)), Tells::default());
        // Seat 1 passed but is no Captain (seat 0 is): one misfit.
        assert!((s.tells_likelihood(Seat(0)) - 0.3).abs() < 1e-9);
        // Were seat 1 the Captain, its pass would fit.
        s.hands[1] = vec![r(1), r(6)];
        assert!((s.tells_likelihood(Seat(0)) - 1.0).abs() < 1e-9);
    }
}
