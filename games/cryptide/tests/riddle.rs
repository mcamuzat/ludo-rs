use std::time::Instant;

use cryptide::knowledge::possible;
use cryptide::{habitat, riddle, Event, HintTable, Mark, Mode, RiddleLevel, Tokens};

#[test]
fn a_riddle_gives_the_solver_s_hint_and_what_it_knows() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            for seed in 0..3u64 {
                let r = riddle(RiddleLevel::Full, mode, players, seed).expect("a riddle");
                assert_eq!(r.level, RiddleLevel::Full);
                assert_eq!(r.players, players);
                assert_eq!(r.hints.len(), usize::from(players));
                assert_eq!(r.hint, r.hints[r.solver.index()]);
                assert!(r.hint.cells(&board_of(&r)).contains(r.habitat));
                assert!(!r.log.iter().any(|e| matches!(e, Event::Won { .. })));
                for (p, given) in r.given.iter().enumerate() {
                    let expected = (p == r.solver.index()).then_some(r.hint);
                    assert_eq!(*given, expected);
                }
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

#[test]
fn a_warmup_shows_every_hint_and_no_token() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            let r = riddle(RiddleLevel::Warmup, mode, players, 1).expect("a warmup");
            assert_eq!(r.level, RiddleLevel::Warmup);
            assert!(r.tokens.iter().all(|t| *t == Tokens::default()));
            assert!(r.log.is_empty());
            assert_eq!(
                r.given,
                r.hints.iter().copied().map(Some).collect::<Vec<_>>()
            );
            let board = board_of(&r);
            let table = HintTable::new(&board, mode);
            let all: Vec<usize> = r.hints.iter().map(|h| table.index_of(h).unwrap()).collect();
            assert_eq!(habitat(&table, &all), Some(r.habitat));
            assert_eq!(r.hint_cells.len(), r.hints.len());
            for (p, cells) in r.hint_cells.iter().enumerate() {
                assert_eq!(*cells, r.hints[p].cells(&board).iter().collect::<Vec<_>>());
            }
        }
    }
}

#[test]
fn a_hint_riddle_leaves_one_hint_to_the_target() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            for seed in 0..3u64 {
                let r = riddle(RiddleLevel::Hint, mode, players, seed).expect("a hint riddle");
                assert_eq!(r.level, RiddleLevel::Hint);
                let board = board_of(&r);
                let table = HintTable::new(&board, mode);
                let target = r.solver;
                assert_eq!(r.hint, r.hints[target.index()]);
                assert_eq!(
                    possible(&table, &r.tokens, target),
                    [table.index_of(&r.hint).unwrap()]
                );
                assert!(r.given.iter().all(Option::is_none));
                assert_eq!(r.eliminations.len(), r.mode_hints.len() - 1);
                for e in &r.eliminations {
                    assert_ne!(e.hint, r.hint);
                    let inside = e.hint.cells(&board).contains(e.cell);
                    let t = &r.tokens[e.cell.index()];
                    match e.mark {
                        Mark::Disc => {
                            assert!(t.discs.contains(&target));
                            assert!(!inside);
                        }
                        Mark::Cube => {
                            assert_eq!(t.cube, Some(target));
                            assert!(inside);
                        }
                    }
                }
                assert!(!r.log.iter().any(|e| matches!(e, Event::Won { .. })));
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
    for level in [RiddleLevel::Warmup, RiddleLevel::Full] {
        assert_eq!(
            riddle(level, Mode::Normal, 4, 7),
            riddle(level, Mode::Normal, 4, 7)
        );
    }
}

#[test]
fn a_riddle_comes_quickly() {
    let t = Instant::now();
    riddle(RiddleLevel::Full, Mode::Advanced, 5, 3).expect("a riddle");
    assert!(t.elapsed().as_secs() < 5, "{:?}", t.elapsed());
}
