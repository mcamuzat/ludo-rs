use kurote::{cell, Action, End, Kurote, State, Wizard, QUIET_LIMIT};
use ludo_core::{Game, Outcome, Seat, Viewer};

/// A position from 5 rows: `g`/`y` a fresh green/yellow wizard, `G`/`Y` an
/// exhausted one, `.` an empty cell. Seat 0 is green.
fn board(rows: [&str; 5], turn: u8) -> State {
    let mut s = State::start();
    for (r, line) in rows.iter().enumerate() {
        for (c, ch) in line.chars().enumerate() {
            s.cells[r * 5 + c] = match ch {
                '.' => None,
                _ => Some(Wizard {
                    owner: u8::from(ch.eq_ignore_ascii_case(&'y')),
                    exhausted: ch.is_ascii_uppercase(),
                }),
            };
        }
    }
    s.turn = Some(Seat(turn));
    s
}

fn play(s: &mut State, seat: u8, action: Action) {
    Kurote::apply(s, Seat(seat), &action).unwrap();
}

#[test]
fn the_starting_position() {
    let s = Kurote::new(&Default::default(), 2, 0);
    let expected = board([".yyy.", "y...y", ".....", "g...g", ".ggg."], 0);
    assert_eq!(s.cells, expected.cells);
    assert_eq!(s.reserve, [2, 2]);
    assert_eq!(s.turn, Some(Seat(0)));
    // Five fresh wizards may not place from the side.
    assert!(!Kurote::legal_actions(&s, Seat(0))
        .iter()
        .any(|a| matches!(a, Action::Place { .. })));
}

/// [K] page 1: the leftmost green wizard's destinations from the start.
#[test]
fn the_rulebook_teleport_example() {
    let s = State::start();
    let mut got = s.destinations(cell(3, 0));
    got.sort_unstable();
    let mut want = vec![
        cell(2, 3),
        cell(2, 4),
        cell(3, 1),
        cell(3, 2),
        cell(3, 3),
        cell(4, 0),
        cell(4, 4),
    ];
    want.sort_unstable();
    assert_eq!(got, want);
}

/// [K] page 2: no green wizard may go where 3 yellows surround.
#[test]
fn the_rulebook_forbidden_cells() {
    let s = board(["y.yy.", "g.y..", ".ggg.", ".....", "....."], 0);
    for from in [cell(1, 0), cell(2, 1), cell(2, 2), cell(2, 3)] {
        let d = s.destinations(from);
        for x in [cell(0, 1), cell(1, 1), cell(1, 3)] {
            assert!(!d.contains(&x), "{from} may not reach {x}");
        }
    }
    assert!(s.destinations(cell(2, 2)).contains(&cell(2, 0)));
}

#[test]
fn a_wizard_needs_another_friend_next_to_its_destination() {
    let s = board(["....y", ".....", ".....", ".....", "g...g"], 0);
    // The two greens are far apart: each may only go next to the other.
    let mut d = s.destinations(cell(4, 0));
    d.sort_unstable();
    assert_eq!(d, vec![cell(3, 3), cell(3, 4), cell(4, 3)]);
}

#[test]
fn moving_exhausts_and_three_around_captures() {
    // Yellow at (1,2) has greens at (2,1) and (2,3); a third arrives.
    let mut s = board([".....", "..y..", ".g.g.", ".g...", "....."], 0);
    play(
        &mut s,
        0,
        Action::Move {
            from: cell(3, 1),
            to: cell(1, 1),
        },
    );
    assert_eq!(s.cells[usize::from(cell(1, 2))], None);
    assert_eq!(s.captured, [1, 0]);
    assert_eq!(s.last.as_ref().unwrap().captured, vec![cell(1, 2)]);
    assert!(s.cells[usize::from(cell(1, 1))].unwrap().exhausted);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn an_exhausted_wizard_cannot_move_until_all_are() {
    let mut s = board(["y.y..", ".....", ".....", ".....", "Gg..."], 0);
    s.reserve = [0, 0];
    assert!(Kurote::apply(
        &mut s,
        Seat(0),
        &Action::Move {
            from: cell(4, 0),
            to: cell(3, 1)
        }
    )
    .is_err());
    play(
        &mut s,
        0,
        Action::Move {
            from: cell(4, 1),
            to: cell(3, 1),
        },
    );
    // Both greens were exhausted: they are fresh again at once.
    assert!(s.cells.iter().flatten().all(|w| !w.exhausted));
}

#[test]
fn placing_from_the_side() {
    let mut s = board(["yyy..", ".....", ".....", ".....", "g...g"], 0);
    let places: Vec<_> = Kurote::legal_actions(&s, Seat(0))
        .into_iter()
        .filter(|a| matches!(a, Action::Place { .. }))
        .collect();
    assert_eq!(places.len(), 3);
    play(&mut s, 0, Action::Place { to: cell(4, 2) });
    assert_eq!(s.reserve, [1, 2]);
    assert!(s.cells[usize::from(cell(4, 2))].unwrap().exhausted);
    // Yellow places on its own end only, and not next to 3 greens.
    let mut s = board(["..y..", "ggg..", ".....", ".....", "....."], 1);
    let yellow = Kurote::legal_actions(&s, Seat(1));
    assert!(yellow.contains(&Action::Place { to: cell(0, 4) }));
    assert!(!yellow.contains(&Action::Place { to: cell(0, 1) }));
    assert!(Kurote::apply(&mut s, Seat(1), &Action::Place { to: cell(4, 0) }).is_err());
}

#[test]
fn three_captures_win() {
    let mut s = board([".....", "..y..", ".g.g.", ".g...", "....."], 0);
    s.captured = [2, 0];
    play(
        &mut s,
        0,
        Action::Move {
            from: cell(3, 1),
            to: cell(1, 1),
        },
    );
    assert_eq!(s.end, Some(End::Captures { winner: Seat(0) }));
    assert_eq!(Kurote::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
    assert!(Kurote::pending(&s).is_empty());
}

#[test]
fn a_blocked_player_passes_and_two_passes_draw() {
    // A lone green with no reserve cannot move; a lone yellow neither.
    let mut s = board(["y....", ".....", ".....", ".....", "....g"], 0);
    s.reserve = [0, 0];
    assert_eq!(Kurote::legal_actions(&s, Seat(0)), vec![Action::Pass]);
    play(&mut s, 0, Action::Pass);
    play(&mut s, 1, Action::Pass);
    assert_eq!(s.end, Some(End::Blocked));
    assert_eq!(Kurote::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
    // Passing when something else is possible is refused.
    let mut s = State::start();
    assert!(Kurote::apply(&mut s, Seat(0), &Action::Pass).is_err());
}

#[test]
fn long_quiet_play_draws() {
    let mut s = State::start();
    s.quiet = QUIET_LIMIT - 1;
    let a = Kurote::legal_actions(&s, Seat(0))[0];
    play(&mut s, 0, a);
    assert_eq!(s.end, Some(End::Limit));
    assert_eq!(Kurote::view(&s, Viewer::Spectator).quiet_left, 0);
}

#[test]
fn refusals_leave_the_state_alone() {
    let mut s = State::start();
    let before = s.clone();
    for a in [
        Action::Move {
            from: cell(0, 1),
            to: cell(2, 2),
        },
        Action::Move { from: 99, to: 0 },
        Action::Place { to: 200 },
        Action::Move {
            from: cell(3, 0),
            to: cell(2, 0),
        },
    ] {
        assert!(Kurote::apply(&mut s, Seat(0), &a).is_err());
        assert_eq!(s, before);
    }
    assert!(Kurote::apply(&mut s, Seat(1), &Action::Pass).is_err());
}
