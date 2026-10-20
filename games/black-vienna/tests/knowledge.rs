use black_vienna::knowledge::{direct_facts, sample_world, Column, Mark, Note};
use black_vienna::testing::{card, letters, table, trio};
use black_vienna::{suspects, Answer, BlackVienna, Config, Suspect};
use ludo_core::{Determinize, Game, Seat, Viewer};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

fn answer(target: u8, name: &str, count: u8) -> Answer {
    Answer {
        by: Seat(0),
        target: Seat(target),
        card: card(name),
        count,
        placed: count,
        special: false,
    }
}

fn note(letter: char, column: Column, mark: Mark) -> Note {
    Note {
        suspect: Suspect::from_letter(letter).unwrap(),
        column,
        mark,
    }
}

#[test]
fn direct_facts_are_only_direct() {
    let mut s = table(&["ABC", "DEFGHIJK", "LMNOPQRS"], "XYZ", ["", "", ""]);
    s.answers = vec![
        answer(1, "BLM", 0),
        answer(2, "DHR", 3),
        answer(1, "ACL", 1),
    ];
    let notes = direct_facts(&s, Seat(0));
    let mut expected = Vec::new();
    for l in ['A', 'B', 'C'] {
        expected.push(note(l, Column::Seat(Seat(0)), Mark::Plus));
        expected.push(note(l, Column::Seat(Seat(1)), Mark::Minus));
        expected.push(note(l, Column::Seat(Seat(2)), Mark::Minus));
        expected.push(note(l, Column::Trio, Mark::Minus));
    }
    for l in ['B', 'L', 'M'] {
        expected.push(note(l, Column::Seat(Seat(1)), Mark::Minus));
    }
    for l in ['D', 'H', 'R'] {
        expected.push(note(l, Column::Seat(Seat(2)), Mark::Plus));
    }
    let mut got = notes.clone();
    got.sort();
    got.dedup();
    expected.sort();
    expected.dedup();
    assert_eq!(got, expected, "nothing from the ACL answered 1");
}

#[test]
fn the_wrong_accuser_notes_the_trio() {
    let mut s = table(&["ABC", "DEFGHIJK", "LMNOPQRS"], "XYZ", ["", "", ""]);
    s.wrong[0] = true;
    let notes = direct_facts(&s, Seat(0));
    for l in ['X', 'Y', 'Z'] {
        assert!(notes.contains(&note(l, Column::Trio, Mark::Plus)));
    }
    assert!(!direct_facts(&s, Seat(1)).contains(&note('X', Column::Trio, Mark::Plus)));
}

#[test]
fn a_sampled_world_fits_every_answer() {
    let mut rng = SmallRng::seed_from_u64(5);
    for _ in 0..20 {
        let mut s = BlackVienna::new(&Config::default(), 4, rng.gen());
        // Ask around a bit with random legal questions.
        for _ in 0..10 {
            let seat = s.turn;
            let asks: Vec<_> = BlackVienna::legal_actions(&s, seat)
                .into_iter()
                .filter(|a| matches!(a, black_vienna::Action::Ask { .. }))
                .collect();
            let pick = asks[rng.gen_range(0..asks.len() as u32) as usize].clone();
            BlackVienna::apply(&mut s, seat, &pick).unwrap();
        }
        let observer = Seat(1);
        let mut trios = Vec::new();
        for _ in 0..20 {
            let (hands, trio) = sample_world(&s, observer, &mut rng);
            assert_eq!(hands[1], s.hands[1]);
            for (h, real) in hands.iter().zip(&s.hands) {
                assert_eq!(h.len(), real.len());
            }
            for a in &s.answers {
                let held = suspects(a.card)
                    .iter()
                    .filter(|x| hands[a.target.index()].contains(x))
                    .count() as u8;
                assert_eq!(held, a.count, "{a:?}");
            }
            let mut all: Vec<Suspect> = hands.concat();
            all.extend(trio);
            all.sort();
            all.dedup();
            assert_eq!(all.len(), 27);
            trios.push(trio);
        }
        trios.sort();
        trios.dedup();
        assert!(!trios.is_empty());
    }
}

#[test]
fn the_wrong_accuser_keeps_the_trio() {
    let mut s = BlackVienna::new(&Config::default(), 3, 9);
    let observer = s.turn;
    s.wrong[observer.index()] = true;
    let mut rng = SmallRng::seed_from_u64(1);
    for _ in 0..10 {
        let d = BlackVienna::determinize(&s, observer, &mut rng);
        assert_eq!(d.trio, s.trio);
        assert_eq!(
            BlackVienna::view(&d, Viewer::Seat(observer)),
            BlackVienna::view(&s, Viewer::Seat(observer))
        );
    }
    let other = Seat((observer.0 + 1) % 3);
    let trios: Vec<_> = (0..20)
        .map(|_| BlackVienna::determinize(&s, other, &mut rng).trio)
        .collect();
    assert!(
        trios.iter().any(|&t| t != s.trio),
        "the others' trio varies"
    );
    let _ = (letters("A"), trio("ABC"));
}

// ---- Deductions by level (task 6) ----

use black_vienna::knowledge::{knowledge, possible_trios};
use black_vienna::{Action, Phase, State};
use ludo_core::ai::Level;
use std::time::Instant;

fn s_(letter: char) -> Suspect {
    Suspect::from_letter(letter).unwrap()
}

/// Seat 0 holds A…H; seat 1 eight cards, seat 2 eight cards; the trio XYZ.
fn three() -> State {
    table(&["ABCDEFGH", "IJKLMNOP", "QRSTUVWÖ"], "XYZ", ["", "", ""])
}

#[test]
fn medium_finds_the_last_suspect_of_an_answer() {
    let mut s = three();
    // Seat 1 has neither K, M nor U; on BLM it has one, and B is seat 0's:
    // so it has L.
    s.answers = vec![answer(1, "KMU", 0), answer(1, "BLM", 1)];
    let easy = knowledge(&s, Seat(0), Level::Easy);
    let medium = knowledge(&s, Seat(0), Level::Medium);
    assert_eq!(easy.get(Column::Seat(Seat(1)), s_('L')), None);
    assert_eq!(medium.get(Column::Seat(Seat(1)), s_('L')), Some(true));
    // And so L is in no other column.
    assert_eq!(medium.get(Column::Seat(Seat(2)), s_('L')), Some(false));
    assert_eq!(medium.get(Column::Trio, s_('L')), Some(false));
}

#[test]
fn medium_closes_a_full_hand() {
    let mut s = table(
        &["ABCD", "EFGH", "IJKL", "MNOP", "QRST", "UVWÖ"],
        "XYZ",
        ["", "", ""],
    );
    // Zeros rule out W, X, R, U and Z at seat 1; then E and G (EGW = 2),
    // F (FRX = 1) and H (HUZ = 1) fill its four cards.
    s.answers = ["IÖW", "JQX", "IPR", "KMU", "DJZ"]
        .iter()
        .map(|name| answer(1, name, 0))
        .chain([
            answer(1, "EGW", 2),
            answer(1, "FRX", 1),
            answer(1, "HUZ", 1),
        ])
        .collect();
    let easy = knowledge(&s, Seat(0), Level::Easy);
    let medium = knowledge(&s, Seat(0), Level::Medium);
    for l in ['E', 'F', 'G', 'H'] {
        assert_eq!(medium.get(Column::Seat(Seat(1)), s_(l)), Some(true), "{l}");
    }
    assert_eq!(easy.get(Column::Seat(Seat(1)), s_('S')), None);
    assert_eq!(medium.get(Column::Seat(Seat(1)), s_('S')), Some(false));
}

#[test]
fn levels_never_exclude_the_true_trio_and_nest() {
    let mut rng = SmallRng::seed_from_u64(17);
    for players in 3..=6u8 {
        for _ in 0..5 {
            let mut s = BlackVienna::new(&Config::default(), players, rng.gen());
            for step in 0..14 {
                if s.phase != Phase::Turn {
                    break;
                }
                let seat = s.turn;
                let asks: Vec<Action> = BlackVienna::legal_actions(&s, seat)
                    .into_iter()
                    .filter(|a| matches!(a, Action::Ask { .. }))
                    .collect();
                if asks.is_empty() {
                    break;
                }
                let pick = asks[rng.gen_range(0..asks.len() as u32) as usize].clone();
                BlackVienna::apply(&mut s, seat, &pick).unwrap();
                if step % 4 != 3 {
                    continue;
                }
                let observer = Seat(rng.gen_range(0..u32::from(players)) as u8);
                let easy = possible_trios(&s, observer, Level::Easy);
                let medium = possible_trios(&s, observer, Level::Medium);
                let hard = possible_trios(&s, observer, Level::Hard);
                for list in [&easy, &medium, &hard] {
                    assert!(list.contains(&s.trio), "the true trio is always possible");
                }
                assert!(medium.iter().all(|t| easy.contains(t)));
                assert!(hard.iter().all(|t| medium.contains(t)));
            }
        }
    }
}

#[test]
fn hard_crosses_answers_that_medium_misses() {
    let mut rng = SmallRng::seed_from_u64(3);
    for _ in 0..200 {
        let mut s = BlackVienna::new(&Config::default(), 3, rng.gen());
        for _ in 0..30 {
            if s.phase != Phase::Turn {
                break;
            }
            let seat = s.turn;
            let asks: Vec<Action> = BlackVienna::legal_actions(&s, seat)
                .into_iter()
                .filter(|a| matches!(a, Action::Ask { .. }))
                .collect();
            if asks.is_empty() {
                break;
            }
            let pick = asks[rng.gen_range(0..asks.len() as u32) as usize].clone();
            BlackVienna::apply(&mut s, seat, &pick).unwrap();
            let hard = possible_trios(&s, Seat(0), Level::Hard);
            if hard.len() == 1 && possible_trios(&s, Seat(0), Level::Medium).len() > 1 {
                assert_eq!(hard[0], s.trio);
                return;
            }
        }
    }
    panic!("Hard never got ahead of Medium");
}

#[test]
fn hard_is_fast_at_the_start() {
    let mut s = BlackVienna::new(&Config::default(), 6, 4);
    for _ in 0..2 {
        let seat = s.turn;
        let ask = BlackVienna::legal_actions(&s, seat)
            .into_iter()
            .find(|a| matches!(a, Action::Ask { .. }))
            .unwrap();
        BlackVienna::apply(&mut s, seat, &ask).unwrap();
    }
    let started = Instant::now();
    let trios = possible_trios(&s, s.turn, Level::Hard);
    assert!(trios.len() > 100);
    assert!(
        started.elapsed().as_millis() < 500,
        "{:?}",
        started.elapsed()
    );
}

/// Review I-1: on long games at 5 and 6 players, Hard never loses the true
/// trio, and decides in well under a second.
#[test]
fn hard_keeps_the_true_trio_in_long_games() {
    use black_vienna::accuse_threshold;
    use ludo_core::Heuristics;
    let mut slowest = std::time::Duration::ZERO;
    for players in [5u8, 6] {
        for seed in 0..40u64 {
            let mut s =
                BlackVienna::new(&Config::default(), players, seed * 7 + u64::from(players));
            let mut rng = SmallRng::seed_from_u64(seed);
            for _ in 0..40 {
                if s.phase != Phase::Turn {
                    break;
                }
                let seat = s.turn;
                let started = Instant::now();
                let hard = possible_trios(&s, seat, Level::Hard);
                slowest = slowest.max(started.elapsed());
                assert!(hard.contains(&s.trio), "{players} joueurs, graine {seed}");
                let _ = accuse_threshold(Level::Hard);
                let action = BlackVienna::policy_action(&s, seat, Level::Medium, &mut rng).unwrap();
                if !matches!(action, Action::Ask { .. }) {
                    break;
                }
                BlackVienna::apply(&mut s, seat, &action).unwrap();
            }
        }
    }
    assert!(slowest.as_millis() < 1000, "{slowest:?}");
}
