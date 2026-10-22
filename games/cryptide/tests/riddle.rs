use std::time::Instant;

use cryptide::{riddle, Event, Mode};

#[test]
fn a_riddle_gives_the_solver_s_hint_and_what_it_knows() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            for seed in 0..3u64 {
                let r = riddle(mode, players, seed).expect("a riddle");
                assert_eq!(r.players, players);
                assert_eq!(r.hints.len(), usize::from(players));
                assert_eq!(r.hint, r.hints[r.solver.index()]);
                assert!(r.hint.cells(&board_of(&r)).contains(r.habitat));
                assert!(!r.log.iter().any(|e| matches!(e, Event::Won { .. })));
                for (p, list) in r.deduction.iter().enumerate() {
                    if p == r.solver.index() {
                        assert!(list.is_empty());
                    } else {
                        assert!(list.contains(&r.hints[p]), "the true hint stays possible");
                    }
                }
            }
        }
    }
}

fn board_of(r: &cryptide::Riddle) -> cryptide::Board {
    cryptide::Board {
        squares: r.squares.clone(),
    }
}

#[test]
fn a_riddle_is_pinned_by_its_seed() {
    assert_eq!(riddle(Mode::Normal, 4, 7), riddle(Mode::Normal, 4, 7));
}

#[test]
fn a_riddle_comes_quickly() {
    let t = Instant::now();
    riddle(Mode::Advanced, 5, 3).expect("a riddle");
    assert!(t.elapsed().as_secs() < 5, "{:?}", t.elapsed());
}
