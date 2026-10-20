use ludo_core::ai::{Budget, Level};
use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::knowledge::{knowledge, possible_trios, Column, Knowledge};
use crate::material::{suspects, Suspect};
use crate::{Action, BlackVienna, Phase, State};

/// [U] The AI accuses once its possible trios are this few: it takes its
/// chance at Easy and Medium, waits for certainty at Hard.
pub fn accuse_threshold(level: Level) -> usize {
    match level {
        Level::Easy => 3,
        Level::Medium => 2,
        Level::Hard => 1,
    }
}

/// The likeliest of `trios`: the one whose suspects the most seats are
/// known not to hold; ties at random.
fn likeliest<R: Rng + ?Sized>(
    s: &State,
    sheet: &Knowledge,
    trios: &[[Suspect; 3]],
    rng: &mut R,
) -> Option<[Suspect; 3]> {
    let ruled_out = |x: Suspect| {
        (0..s.players)
            .filter(|&p| sheet.get(Column::Seat(Seat(p)), x) == Some(false))
            .count()
    };
    let score = |t: &[Suspect; 3]| t.iter().map(|&x| ruled_out(x)).sum::<usize>();
    let best = trios.iter().map(score).max()?;
    let top: Vec<[Suspect; 3]> = trios.iter().copied().filter(|t| score(t) == best).collect();
    top.choose(rng).copied()
}

/// What `ask` may teach `seat`: the unknown cells, at the target, of the
/// suspects on its cards.
fn unknowns(s: &State, sheet: &Knowledge, ask: &Action) -> usize {
    let Action::Ask {
        pile,
        target,
        extra,
    } = *ask
    else {
        return 0;
    };
    let top = pile.and_then(|p| s.piles[usize::from(p)].last().copied());
    top.into_iter()
        .chain(extra)
        .flat_map(suspects)
        .filter(|&x| sheet.get(Column::Seat(target), x).is_none())
        .count()
}

impl Heuristics for BlackVienna {
    /// The policy decides every move: no search.
    fn budget(_level: Level, _timed: bool) -> Option<Budget> {
        None
    }

    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        Self::policy_action(s, seat, Level::Medium, rng)
            .or_else(|| Self::legal_actions(s, seat).into_iter().next())
            .unwrap_or(Action::Ask {
                pile: None,
                target: seat,
                extra: None,
            })
    }

    /// From what `seat` saw only. [U] Accuse once the possible trios are
    /// few enough for the level (or when the rules force it), naming the
    /// likeliest; otherwise ask what teaches the most (Easy: at random).
    fn policy_action<R: Rng + ?Sized>(
        s: &State,
        seat: Seat,
        level: Level,
        rng: &mut R,
    ) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return None;
        }
        let sheet = knowledge(s, seat, level);
        let trios = possible_trios(s, seat, level);
        if s.phase == Phase::Guess {
            let trio = likeliest(s, &sheet, &trios, rng)?;
            return Some(Action::Guess { trio });
        }
        let asks: Vec<Action> = legal
            .iter()
            .filter(|a| matches!(a, Action::Ask { .. }))
            .cloned()
            .collect();
        let can_accuse = legal.iter().any(|a| matches!(a, Action::Accuse { .. }));
        // The last player asks its final question unless already sure enough;
        // after it, only accusations are left (`asks` is empty).
        if can_accuse && (trios.len() <= accuse_threshold(level) || asks.is_empty()) {
            let trio = likeliest(s, &sheet, &trios, rng)?;
            return Some(Action::Accuse { trio });
        }
        if level == Level::Easy {
            return asks.choose(rng).cloned();
        }
        let best = asks.iter().map(|a| unknowns(s, &sheet, a)).max()?;
        let top: Vec<Action> = asks
            .into_iter()
            .filter(|a| unknowns(s, &sheet, a) == best)
            .collect();
        top.choose(rng).cloned()
    }

    /// A blunder never accuses.
    fn random_actions(s: &State, seat: Seat) -> Vec<Action> {
        let legal = Self::legal_actions(s, seat);
        let asks: Vec<Action> = legal
            .iter()
            .filter(|a| matches!(a, Action::Ask { .. }))
            .cloned()
            .collect();
        if asks.is_empty() {
            legal
        } else {
            asks
        }
    }
}
