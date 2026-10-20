use cryptide::knowledge::{possible, query, Inference, CAP};
use cryptide::testing::{fixed_puzzle, table};
use cryptide::{ideal_cube_share, Action, Cell, Config, Cryptide, Mode, Phase};
use ludo_core::ai::Level;
use ludo_core::{Game, Heuristics, Seat};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const LEVELS: [Level; 3] = [Level::Easy, Level::Medium, Level::Hard];

#[test]
fn cube_shares_follow_the_table_size() {
    assert_eq!(ideal_cube_share(3), 0.35);
    assert_eq!(ideal_cube_share(4), 0.30);
    assert_eq!(ideal_cube_share(5), 0.25);
}

#[test]
fn policy_is_legal_at_every_level() {
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in [3u8, 5] {
            for game in 0..5u64 {
                let mut s = Cryptide::new(&Config { mode }, players, game);
                let mut moves = 0;
                while s.phase != Phase::Over {
                    let seat = s.turn;
                    let level = LEVELS[(seat.index() + game as usize) % 3];
                    let a = Cryptide::policy_action(&s, seat, level, &mut rng).expect("an action");
                    assert!(Cryptide::legal_actions(&s, seat).contains(&a), "{a:?}");
                    Cryptide::apply(&mut s, seat, &a).expect("legal");
                    moves += 1;
                    assert!(moves < 2_000, "the game does not end");
                }
            }
        }
    }
}

#[test]
fn medium_searches_a_single_candidate() {
    // The other seats have shown their whole hint: a disc on every space
    // it allows, a cube on every other one (where no cube is yet).
    let mut s = table(3, fixed_puzzle(3));
    for p in 1..3u8 {
        for c in (0..108).map(Cell) {
            if s.hint_mask(Seat(p)).contains(c) {
                s.tokens[c.index()].discs.push(Seat(p));
            } else if s.tokens[c.index()].cube.is_none() {
                s.tokens[c.index()].cube = Some(Seat(p));
            }
        }
    }
    let k = query(&s, Seat(0), Inference::Bot, CAP)
        .expect("a hint")
        .combos();
    if k.cells.len() == 1 {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let a = Cryptide::policy_action(&s, Seat(0), Level::Medium, &mut rng);
        assert_eq!(a, Some(Action::Search(s.puzzle.habitat)));
    } else {
        panic!(
            "the test table should leave one candidate, got {}",
            k.cells.len()
        );
    }
}

#[test]
fn medium_asks_the_least_known_opponent() {
    // Seats 1, 2 and 3 have each shown two cubes, seat 4 nothing.
    let mut s = table(5, fixed_puzzle(5));
    for p in 1..4u8 {
        let shown: Vec<Cell> = (0..108)
            .map(Cell)
            .filter(|&c| !s.hint_mask(Seat(p)).contains(c) && s.tokens[c.index()].cube.is_none())
            .take(2)
            .collect();
        for c in shown {
            s.tokens[c.index()].cube = Some(Seat(p));
        }
    }
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    for _ in 0..10 {
        match Cryptide::policy_action(&s, Seat(0), Level::Medium, &mut rng) {
            Some(Action::Ask { target, .. }) => assert_eq!(target, Seat(4)),
            other => panic!("expected a question, got {other:?}"),
        }
    }
}

#[test]
fn penalty_reveals_the_least() {
    let mut s = table(3, fixed_puzzle(3));
    s.phase = Phase::Penalty;
    let mut rng = ChaCha8Rng::seed_from_u64(4);
    let Some(Action::Cube(c)) = Cryptide::policy_action(&s, Seat(0), Level::Medium, &mut rng)
    else {
        panic!("a cube");
    };
    let mine_known = possible(s.table(), &s.tokens, Seat(0));
    let cost = |c: Cell| {
        mine_known
            .iter()
            .filter(|&&i| s.table().masks[i].contains(c))
            .count()
    };
    let best = Cryptide::legal_actions(&s, Seat(0))
        .into_iter()
        .filter_map(|a| match a {
            Action::Cube(c) => Some(cost(c)),
            _ => None,
        })
        .min()
        .expect("legal cubes");
    assert_eq!(cost(c), best);
}
