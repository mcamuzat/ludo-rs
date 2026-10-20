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
