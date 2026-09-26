//! Checks every game must pass (called from each game's tests).

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use serde_json::Value;

use crate::{replay, Determinize, Game, Match, MatchLog, Outcome, Pending, Seat, Viewer};

fn json<T: Serialize + ?Sized>(value: &T) -> Value {
    serde_json::to_value(value).expect("engine types serialize to JSON")
}

/// Plays `games` random matches to the end and panics at the first breach
/// of the `Game` contract.
pub fn check<G: Game + Determinize>(config: &G::Config, players: u8, games: usize, seed: u64) {
    let mut rng = SmallRng::seed_from_u64(seed);
    for g in 0..games {
        let mut m = Match::<G>::new(config.clone(), players, rng.gen())
            .unwrap_or_else(|e| panic!("partie {g} : création refusée : {e}"));
        let mut step = 0usize;
        loop {
            let s = m.state();
            let pending = G::pending(s);
            let outcome = G::outcome(s);
            assert_eq!(
                pending.is_empty(),
                outcome.is_some(),
                "partie {g}, coup {step} : `pending` vide ⇔ `outcome` défini"
            );
            // Reading a state back may replay the whole game: sample it.
            if step < 20 || step.is_multiple_of(25) {
                check_serde::<G>(s, g, step);
            }
            check_visible_pending::<G>(s, &pending, players, g, step);
            check_timeouts::<G>(s, &pending, g, step);
            if G::meta().hidden_info {
                check_hidden::<G>(s, players, &mut rng, g, step);
            }
            if let Some(Outcome::Ranked(ranks)) = outcome {
                assert_eq!(
                    ranks.len(),
                    usize::from(players),
                    "partie {g} : un rang par siège"
                );
                break;
            }
            let seat = *pending.0.choose(&mut rng).expect("non-empty");
            let legal = G::legal_actions(s, seat);
            assert!(
                !legal.is_empty(),
                "partie {g}, coup {step} : le siège {} est en attente sans action légale",
                seat.0
            );
            for other in (0..players).map(Seat).filter(|&x| !pending.contains(x)) {
                assert!(
                    G::legal_actions(s, other).is_empty(),
                    "partie {g}, coup {step} : le siège {} n'est pas en attente mais a des actions",
                    other.0
                );
                let mut copy = s.clone();
                assert!(
                    G::apply(&mut copy, other, &legal[0]).is_err(),
                    "partie {g}, coup {step} : le siège {} a joué hors de son tour",
                    other.0
                );
                assert_eq!(
                    json(&copy),
                    json(s),
                    "partie {g}, coup {step} : un `apply` refusé a modifié l'état"
                );
            }
            let action = legal.choose(&mut rng).expect("non-empty").clone();
            m.apply(seat, action.clone()).unwrap_or_else(|e| {
                panic!("partie {g}, coup {step} : action légale {action:?} refusée : {e}")
            });
            step += 1;
            assert!(step < 100_000, "partie {g} : la partie ne se termine pas");
        }
        check_log::<G>(&m, g);
    }
}

fn check_serde<G: Game>(s: &G::State, g: usize, step: usize) {
    let text = serde_json::to_string(s).expect("state serializes");
    let back: G::State = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("partie {g}, coup {step} : l'état ne se relit pas : {e}"));
    assert_eq!(
        json(&back),
        json(s),
        "partie {g}, coup {step} : aller-retour serde de l'état"
    );
}

fn check_visible_pending<G: Game>(
    s: &G::State,
    pending: &Pending,
    players: u8,
    g: usize,
    step: usize,
) {
    for seat in (0..players).map(Seat) {
        let visible = G::visible_pending(s, Viewer::Seat(seat));
        assert!(
            visible.0.iter().all(|&x| pending.contains(x)),
            "partie {g}, coup {step} : `visible_pending` du siège {} contient un siège qui n'est pas en attente",
            seat.0
        );
        if pending.contains(seat) {
            assert!(
                visible.contains(seat),
                "partie {g}, coup {step} : le siège {} ne se voit pas en attente",
                seat.0
            );
        }
    }
}

fn check_timeouts<G: Game>(s: &G::State, pending: &Pending, g: usize, step: usize) {
    for &seat in &pending.0 {
        match G::timeout_action(s, seat) {
            Some(action) => assert!(
                G::legal_actions(s, seat).contains(&action),
                "partie {g}, coup {step} : l'action par défaut {action:?} du siège {} est illégale",
                seat.0
            ),
            None => assert!(
                !G::meta().clock,
                "partie {g}, coup {step} : le jeu a un chrono mais pas d'action par défaut pour le siège {}",
                seat.0
            ),
        }
    }
}

fn check_hidden<G: Game + Determinize>(
    s: &G::State,
    players: u8,
    rng: &mut SmallRng,
    g: usize,
    step: usize,
) {
    for seat in (0..players).map(Seat) {
        let expected = json(&G::view(s, Viewer::Seat(seat)));
        for _ in 0..2 {
            let d = G::determinize(s, seat, rng);
            assert_eq!(
                json(&G::view(&d, Viewer::Seat(seat))),
                expected,
                "partie {g}, coup {step} : la vue du siège {} dépend de données cachées",
                seat.0
            );
            assert_eq!(
                G::visible_pending(&d, Viewer::Seat(seat)),
                G::visible_pending(s, Viewer::Seat(seat)),
                "partie {g}, coup {step} : l'attente visible du siège {} dépend de données cachées",
                seat.0
            );
        }
    }
    let spectator = json(&G::view(s, Viewer::Spectator));
    let d = G::determinize(s, Seat(0), rng);
    assert_eq!(
        json(&G::view(&d, Viewer::Spectator)),
        spectator,
        "partie {g}, coup {step} : la vue spectateur dépend de données cachées"
    );
}

fn check_log<G: Game>(m: &Match<G>, g: usize) {
    let final_state = json(m.state());
    let replayed = replay::<G>(m.log())
        .unwrap_or_else(|e| panic!("partie {g} : le journal ne se rejoue pas : {e}"));
    assert_eq!(
        json(&replayed),
        final_state,
        "partie {g} : le rejeu ne redonne pas l'état final"
    );
    let text = serde_json::to_string(m.log()).expect("log serializes");
    let back: MatchLog<G> = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("partie {g} : le journal ne se relit pas : {e}"));
    assert_eq!(
        json(&replay::<G>(&back).expect("replays")),
        final_state,
        "partie {g} : aller-retour serde du journal"
    );
    if let Some(last) = m.log().actions.last().cloned() {
        let mut tampered = m.log().clone();
        tampered.actions.push(last);
        assert!(
            replay::<G>(&tampered).is_err(),
            "partie {g} : un journal avec une action après la fin est accepté"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::testgames::{HighCard, Leaky, Nim, NimConfig};

    #[test]
    #[should_panic(expected = "action par défaut")]
    fn an_illegal_timeout_action_is_caught() {
        check::<crate::testgames::BadClock>(&(), 2, 5, 4);
    }

    #[test]
    fn nim_conforms() {
        check::<Nim>(&NimConfig::default(), 3, 200, 1);
    }

    #[test]
    fn highcard_conforms() {
        check::<HighCard>(&(), 4, 200, 2);
    }

    #[test]
    #[should_panic(expected = "dépend de données cachées")]
    fn a_leaking_view_is_caught() {
        check::<Leaky>(&(), 3, 20, 3);
    }
}
