use ludo_core::ai::{choose, Budget, Level, Search};
use ludo_core::{Game, Heuristics, Seat, Viewer};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use riichi::testing::wall;
use riichi::{Action, Riichi, State, Tile};

fn t(text: &str) -> Tile {
    Tile::parse(text).unwrap()
}

fn deal() -> State {
    State::start(
        1,
        Some(wall(
            [
                "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
                "5p 5pr S S S W W N N P P F F",
            ],
            &["N"],
        )),
    )
}

#[test]
fn the_policy_wins_when_it_can() {
    let mut s = deal();
    Riichi::apply(&mut s, Seat(0), &Action::Discard { tile: t("5p") }).unwrap();
    for seed in 0..10 {
        assert_eq!(
            Riichi::rollout_action(&s, Seat(1), &mut SmallRng::seed_from_u64(seed)),
            Action::Ron
        );
    }
}

#[test]
fn the_policy_throws_the_useless_honor() {
    // The dealer's isolated N is worth nothing (E is both round and seat wind).
    let s = deal();
    for seed in 0..10 {
        assert_eq!(
            Riichi::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(seed)),
            Action::Tsumogiri { tile: t("N") }
        );
    }
}

#[test]
fn easy_is_the_policy_alone_and_the_others_search() {
    assert_eq!(Riichi::budget(Level::Easy, true), None);
    assert_eq!(
        Riichi::budget(Level::Medium, true),
        Some(Budget::Millis(300))
    );
    assert_eq!(
        Riichi::budget(Level::Hard, false),
        Some(Budget::Iterations(300))
    );
}

#[test]
fn the_search_does_not_read_hidden_hands() {
    let swap = |a: &str, b: &str| {
        let mut s = State::start(
            1,
            Some(wall(
                [
                    "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                    "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                    a,
                    b,
                ],
                &["N"],
            )),
        );
        Riichi::apply(&mut s, Seat(0), &Action::Discard { tile: t("5p") }).unwrap();
        s
    };
    let x = swap(
        "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
        "5p 5pr S S S W W N N P P F F",
    );
    let y = swap(
        "5p 5pr S S S W W N N P P F F",
        "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
    );
    for seed in 0..5 {
        let search = Search {
            budget: Some(Budget::Iterations(20)),
            blunder: 0.0,
            seed,
        };
        assert_eq!(
            choose::<Riichi>(&x, Seat(1), &search, &|| 0.0).unwrap(),
            choose::<Riichi>(&y, Seat(1), &search, &|| 0.0).unwrap()
        );
    }
}

/// Our points must equal what the engine pays (single winner, no yakuman).
#[test]
fn reported_points_match_the_engine_payments() {
    let mut rng = SmallRng::seed_from_u64(9);
    let mut checked = 0;
    for game in 0..3u64 {
        let mut s = State::start(100 + game, None);
        let mut seen = 0;
        while Riichi::outcome(&s).is_none() {
            let sticks = Riichi::view(&s, Viewer::Spectator).round.riichi_sticks;
            let seat = Riichi::pending(&s).0[0];
            let action = Riichi::rollout_action(&s, seat, &mut rng);
            Riichi::apply(&mut s, seat, &action).unwrap();
            if s.hands_played() != seen {
                seen = s.hands_played();
                let result = Riichi::view(&s, Viewer::Spectator).last_hand.unwrap();
                // The settlement moves points between players; the riichi sticks
                // on the table go to the winner, or to the first place when the
                // game ends without one (Mahjong Soul).
                let over = Riichi::outcome(&s).is_some();
                let collected = if result.wins.is_empty() && !over {
                    0
                } else {
                    sticks as i32 * 1000
                };
                assert_eq!(result.deltas.iter().sum::<i32>(), collected, "{result:?}");
                if let [win] = result.wins.as_slice() {
                    if !win.yakuman {
                        assert_eq!(
                            result.deltas[win.seat.index()],
                            win.points as i32 + sticks as i32 * 1000,
                            "{result:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked >= 10, "only {checked} wins checked");
}

/// A simulated hand scores its points, not only the ranking they give.
#[test]
fn a_hand_is_valued_by_its_points() {
    let root = deal();
    let mut sim = root.clone();
    Riichi::apply(&mut sim, Seat(0), &Action::Discard { tile: t("5p") }).unwrap();
    assert_eq!(Riichi::evaluate(&root, &sim), None, "the hand is not over");
    Riichi::apply(&mut sim, Seat(3), &Action::Pass).unwrap();
    Riichi::apply(&mut sim, Seat(1), &Action::Ron).unwrap();
    let rewards = Riichi::evaluate(&root, &sim).expect("the hand is over");
    let gain = f64::from(sim.scores()[1] - root.scores()[1]);
    // Ranking reward 1 (first), plus the points: more than 0.75, below 1.
    assert!(
        (rewards[1] - (0.5 + 0.5 * (0.5 + gain / 24_000.0))).abs() < 1e-9,
        "{rewards:?}"
    );
    assert!(rewards[1] > 0.75 && rewards[1] < 1.0);
    assert!(
        rewards[0] < rewards[2],
        "the dealer who dealt in did worse than a bystander"
    );
}

/// The search weighs sensible moves only: wins, riichi, yaku-giving calls,
/// and the discards that keep the hand as close to complete as possible.
#[test]
fn the_search_considers_sensible_moves_only() {
    let mut s = deal();
    let start = Riichi::search_actions(&s, Seat(0));
    assert!(start.contains(&Action::Tsumogiri { tile: t("N") }));
    assert!(
        !start.contains(&Action::Discard { tile: t("1m") }),
        "breaking a triplet raises shanten"
    );
    assert!(start
        .iter()
        .all(|a| Riichi::legal_actions(&s, Seat(0)).contains(a)));

    Riichi::apply(&mut s, Seat(0), &Action::Discard { tile: t("5p") }).unwrap();
    assert_eq!(Riichi::search_actions(&s, Seat(1)), vec![Action::Ron]);
    assert_eq!(
        Riichi::search_actions(&s, Seat(3)),
        vec![Action::Pass],
        "a pon of 5p gives no yaku"
    );

    let mut e = State::start(
        1,
        Some(wall(
            [
                "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
                "E E S S S W W N N P P F F",
            ],
            &["N"],
        )),
    );
    Riichi::apply(&mut e, Seat(0), &Action::Discard { tile: t("E") }).unwrap();
    let calls = Riichi::search_actions(&e, Seat(3));
    assert!(
        calls.contains(&Action::Pass) && calls.iter().any(|a| matches!(a, Action::Pon { .. })),
        "E is the round wind: {calls:?}"
    );
}
