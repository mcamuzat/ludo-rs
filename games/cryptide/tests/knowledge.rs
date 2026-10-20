use std::time::Instant;

use cryptide::knowledge::{possible, query, Inference, CAP};
use cryptide::testing::{fixed_puzzle, table};
use cryptide::{Cell, Config, Cryptide, Mode, Phase, State};
use ludo_core::ai::Level;
use ludo_core::{Game, Heuristics, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn true_hint(s: &State, seat: Seat) -> usize {
    s.table()
        .index_of(&s.puzzle.hints[seat.index()])
        .expect("a hint of the mode")
}

#[test]
fn possible_follows_tokens() {
    let mut s = table(3, fixed_puzzle(3));
    let all = possible(s.table(), &s.tokens, Seat(1));
    assert_eq!(all.len(), s.table().hints.len());
    let c = Cell(40);
    s.tokens[c.index()].discs.push(Seat(1));
    let with_disc = possible(s.table(), &s.tokens, Seat(1));
    assert!(with_disc.iter().all(|&i| s.table().masks[i].contains(c)));
    assert!(with_disc.len() < all.len());
    let d = Cell(41);
    s.tokens[d.index()].cube = Some(Seat(1));
    let with_cube = possible(s.table(), &s.tokens, Seat(1));
    assert!(with_cube.iter().all(|&i| !s.table().masks[i].contains(d)));
    // Another seat's tokens say nothing about seat 1.
    s.tokens[Cell(42).index()].cube = Some(Seat(2));
    assert_eq!(possible(s.table(), &s.tokens, Seat(1)), with_cube);
}

/// A game after the setup and `moves` random legal moves (no winning search).
fn played(mode: Mode, players: u8, seed: u64, moves: usize) -> State {
    let mut s = Cryptide::new(&Config { mode }, players, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut done = 0;
    while s.phase != Phase::Over && (s.phase == Phase::Setup || done < moves) {
        let seat = s.turn;
        let legal: Vec<_> = Cryptide::legal_actions(&s, seat)
            .into_iter()
            .filter(|a| *a != cryptide::Action::Search(s.puzzle.habitat))
            .collect();
        let Some(a) = legal.choose(&mut rng) else {
            break;
        };
        let mut next = s.clone();
        Cryptide::apply(&mut next, seat, a).expect("legal");
        if next.phase == Phase::Over {
            continue;
        }
        if s.phase != Phase::Setup {
            done += 1;
        }
        s = next;
    }
    s
}

#[test]
fn combos_find_the_true_habitat() {
    for players in [3u8, 4] {
        for seed in 0..30 {
            let s = played(Mode::Normal, players, seed, 6);
            let me = Seat(0);
            for inference in [Inference::Bot, Inference::Puzzle] {
                let k = query(&s, me, inference, u64::MAX).expect("a hint").combos();
                assert!(k.complete);
                assert!(
                    k.cells.iter().any(|&(c, _)| c == s.puzzle.habitat),
                    "{players} {seed}"
                );
                for p in 1..players {
                    assert!(k.per_seat[usize::from(p)].contains(&true_hint(&s, Seat(p))));
                }
                assert!(k.per_seat[0].is_empty());
            }
        }
    }
}

/// The setup played by the AI (its least revealing cubes: the worst case
/// for the enumeration), in advanced mode at five.
fn after_ai_setup(seed: u64) -> State {
    let mut s = Cryptide::new(
        &Config {
            mode: Mode::Advanced,
        },
        5,
        seed,
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    while s.phase == Phase::Setup {
        let seat = s.turn;
        let a = Cryptide::policy_action(&s, seat, Level::Medium, &mut rng).expect("a cube");
        Cryptide::apply(&mut s, seat, &a).expect("legal");
    }
    s
}

#[test]
fn enumeration_is_fast_at_five_advanced() {
    // Measured worst case: about 4 ms natively in release; much slower in
    // a debug build.
    let limit_ms = if cfg!(debug_assertions) { 400 } else { 30 };
    for seed in 0..10 {
        let s = after_ai_setup(seed);
        for inference in [Inference::Bot, Inference::Puzzle] {
            let start = Instant::now();
            let k = query(&s, Seat(0), inference, CAP).expect("a hint").combos();
            let spent = start.elapsed();
            assert!(k.complete, "seed {seed}: the cap stopped the walk");
            assert!(k.cells.iter().any(|&(c, _)| c == s.puzzle.habitat));
            assert!(spent.as_millis() < limit_ms, "seed {seed}: {spent:?}");
        }
    }
}

#[test]
fn view_reveals_only_at_the_end() {
    let mut s = table(3, fixed_puzzle(3));
    let v = Cryptide::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(v.my_hint, Some(s.puzzle.hints[1]));
    assert!(v.reveal.is_none());
    assert_eq!(v.hints.len(), 23);
    let spectator = Cryptide::view(&s, Viewer::Spectator);
    assert_eq!(spectator.my_hint, None);
    let json = serde_json::to_string(&spectator).expect("view");
    assert!(!json.contains("habitat"));
    let home = s.puzzle.habitat;
    Cryptide::apply(&mut s, Seat(0), &cryptide::Action::Search(home)).expect("legal");
    let v = Cryptide::view(&s, Viewer::Spectator);
    let reveal = v.reveal.expect("over");
    assert_eq!(reveal.habitat, home);
    assert_eq!(reveal.hints, s.puzzle.hints);
    assert_eq!(v.focus, Some(home));
}
