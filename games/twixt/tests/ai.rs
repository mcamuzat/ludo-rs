use ludo_core::ai::{choose, Budget, Level, Search};
use ludo_core::{Game, Seat};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use twixt::swap::swap_wanted;
use twixt::{Color, Config, Point, Twixt, TwixtAction, TwixtState};

fn p(x: u8, y: u8) -> Point {
    Point { x, y }
}

/// Alternates the two lists, Red first.
fn position(red: &[(u8, u8)], black: &[(u8, u8)]) -> TwixtState {
    let mut s = Twixt::new(&Config::default(), 2, 0);
    for i in 0..red.len().max(black.len()) {
        for (list, seat) in [(red, Seat(0)), (black, Seat(1))] {
            if let Some(&(x, y)) = list.get(i) {
                Twixt::apply(&mut s, seat, &TwixtAction::Place(p(x, y))).unwrap();
            }
        }
    }
    s
}

fn search(level: Level) -> Search {
    Search {
        budget: <Twixt as ludo_core::Heuristics>::budget(level, false),
        blunder: 0.0,
        seed: 3,
        level,
    }
}

fn ai(s: &TwixtState, level: Level) -> TwixtAction {
    let seat = Twixt::pending(s).0[0];
    choose::<Twixt>(s, seat, &search(level), &|| 0.0).unwrap()
}

#[test]
fn medium_takes_the_immediate_win() {
    // Red's chain (5 + i%2, 2i) from y = 0 to y = 22 lacks its last peg.
    let red: Vec<_> = (0..12u8).map(|i| (5 + i % 2, 2 * i)).collect();
    let black: Vec<_> = (0..12u8).map(|i| (20, 1 + i)).collect();
    let mut s = position(&red, &black);
    let a = ai(&s, Level::Medium);
    Twixt::apply(&mut s, Seat(0), &a).unwrap();
    assert_eq!(s.result, Some(Some(Color::Red)), "{a:?}");
}

#[test]
fn hard_blocks_the_immediate_loss() {
    // Black's chain (0,13), (2i+1, 11-i) reaches (21,1): (23,2) would win.
    let mut black = vec![(0, 13)];
    black.extend((0..11u8).map(|i| (2 * i + 1, 11 - i)));
    let red = [
        (22, 0),
        (3, 20),
        (7, 20),
        (11, 20),
        (15, 20),
        (19, 20),
        (3, 16),
        (7, 16),
        (11, 16),
        (15, 16),
        (19, 16),
        (5, 22),
    ];
    let mut s = position(&red, &black);
    let mut threat = s.clone();
    Twixt::apply(&mut threat, Seat(0), &TwixtAction::Place(p(3, 18))).unwrap();
    Twixt::apply(&mut threat, Seat(1), &TwixtAction::Place(p(23, 2))).unwrap();
    assert_eq!(
        threat.result,
        Some(Some(Color::Black)),
        "the threat is real"
    );

    let a = ai(&s, Level::Hard);
    Twixt::apply(&mut s, Seat(0), &a).unwrap();
    for reply in Twixt::legal_actions(&s, Seat(1)) {
        let mut t = s.clone();
        Twixt::apply(&mut t, Seat(1), &reply).unwrap();
        assert_ne!(
            t.result,
            Some(Some(Color::Black)),
            "{a:?} lets {reply:?} win"
        );
    }
}

#[test]
fn swap_follows_the_swap_model() {
    assert!(swap_wanted(p(11, 11)));
    assert!(!swap_wanted(p(1, 0)));
    for (first, expected) in [(p(11, 11), true), (p(1, 0), false)] {
        let s = position(&[(first.x, first.y)], &[]);
        for level in [Level::Easy, Level::Medium, Level::Hard] {
            assert_eq!(
                ai(&s, level) == TwixtAction::Swap,
                expected,
                "{first:?} {level:?}"
            );
        }
    }
}

#[test]
fn every_level_plays_legal_moves() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(5);
    for _ in 0..4 {
        let mut s = Twixt::new(&Config::default(), 2, 0);
        for _ in 0..30 {
            let seat = Twixt::pending(&s).0[0];
            let a = Twixt::legal_actions(&s, seat)
                .choose(&mut rng)
                .unwrap()
                .clone();
            Twixt::apply(&mut s, seat, &a).unwrap();
            if s.result.is_some() {
                break;
            }
        }
        if s.result.is_some() {
            continue;
        }
        let seat = Twixt::pending(&s).0[0];
        for level in [Level::Easy, Level::Medium] {
            let a = ai(&s, level);
            assert!(
                Twixt::legal_actions(&s, seat).contains(&a),
                "{level:?} {a:?}"
            );
        }
    }
}

#[test]
fn hard_respects_an_iteration_budget() {
    let s = position(&[(10, 10)], &[(12, 12)]);
    let quick = Search {
        budget: Some(Budget::Iterations(4)),
        ..search(Level::Hard)
    };
    let a = choose::<Twixt>(&s, Seat(0), &quick, &|| 0.0).unwrap();
    assert!(Twixt::legal_actions(&s, Seat(0)).contains(&a));
}
