use black_vienna::knowledge::possible_trios;
use black_vienna::{accuse_threshold, Action, BlackVienna, Config, Phase, State};
use ludo_core::ai::Level;
use ludo_core::{Game, Heuristics, Seat};
use rand::rngs::SmallRng;
use rand::SeedableRng;

const LEVELS: [Level; 3] = [Level::Easy, Level::Medium, Level::Hard];

fn policy(s: &State, seat: Seat, level: Level, seed: u64) -> Action {
    BlackVienna::policy_action(s, seat, level, &mut SmallRng::seed_from_u64(seed))
        .expect("the policy always decides")
}

/// Medium's questions, until the game has some answers or Medium would
/// accuse.
fn asked_around(seed: u64, players: u8, questions: usize) -> State {
    let mut s = BlackVienna::new(&Config::default(), players, seed);
    for k in 0..questions {
        if s.phase != Phase::Turn {
            break;
        }
        let seat = s.turn;
        let action = policy(&s, seat, Level::Medium, seed + k as u64);
        if !matches!(action, Action::Ask { .. }) {
            break;
        }
        BlackVienna::apply(&mut s, seat, &action).unwrap();
    }
    s
}

#[test]
fn thresholds_follow_the_level() {
    assert_eq!(LEVELS.map(accuse_threshold), [3, 2, 1]);
}

#[test]
fn accuses_exactly_at_its_threshold() {
    let mut seen = [false, false];
    for seed in 0..60 {
        let s = asked_around(seed, 3, (seed % 25) as usize);
        if s.phase != Phase::Turn {
            continue;
        }
        for level in LEVELS {
            let count = possible_trios(&s, s.turn, level).len();
            let accused = matches!(policy(&s, s.turn, level, seed), Action::Accuse { .. });
            assert_eq!(
                accused,
                count <= accuse_threshold(level),
                "{level:?} with {count} trios"
            );
            seen[usize::from(accused)] = true;
        }
    }
    assert_eq!(seen, [true, true], "both cases met");
}

#[test]
fn never_blunders_an_accusation() {
    let s = asked_around(4, 4, 3);
    let random = BlackVienna::random_actions(&s, s.turn);
    assert!(!random.is_empty());
    assert!(random.iter().all(|a| matches!(a, Action::Ask { .. })));
}

#[test]
fn policy_is_always_legal() {
    for players in 3..=6u8 {
        for level in LEVELS {
            for seed in 0..3 {
                let mut s = BlackVienna::new(&Config::default(), players, seed);
                let mut steps = 0;
                while let Some(&seat) = BlackVienna::pending(&s).0.first() {
                    let action = policy(&s, seat, level, steps);
                    assert!(
                        BlackVienna::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    BlackVienna::apply(&mut s, seat, &action).unwrap();
                    steps += 1;
                    assert!(steps < 500);
                }
            }
        }
    }
}
