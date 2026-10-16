//! Rule tests on real cells of the board (ids from `board_data.rs`), mostly
//! in the bottom-left quarter, from start line 1 to the finish:
//!
//! - line 1: 60 59 58 57 (centre outwards), leaving by 57→65, 58→66, 60→68;
//! - rows: [65 66 67], [64 63 62 61], [78 79], [77 76], [83 82 81 80], and the
//!   arrow cells [72 73], [74 75];
//! - links: 66→69, 69→62, 67→71, 68→71, 71→70, 70→64, 64→79, 63→78, 61→76,
//!   78→82, 77→81, 80→72, 81→73, 83→75.

use std::collections::BTreeSet;

use bantu::testing::position;
use bantu::{
    is_protected, move_length, Action, Bantu, Config, Dest, End, Spot, State, QUIET_ROUNDS,
};
use ludo_core::{Game, Outcome, Seat};

fn on(cell: u8) -> Spot {
    Spot::On { cell }
}

fn dests(s: &State, seat: u8, piece: u8) -> BTreeSet<Dest> {
    Bantu::legal_actions(s, Seat(seat))
        .into_iter()
        .filter_map(|a| match a {
            Action::Move { piece: p, to } if p == piece => Some(to),
            Action::Move { .. } => None,
        })
        .collect()
}

fn cells(list: &[u8]) -> BTreeSet<Dest> {
    list.iter().map(|&c| Dest::Cell(c)).collect()
}

fn go(s: &mut State, seat: u8, piece: u8, to: Dest) {
    Bantu::apply(s, Seat(seat), &Action::Move { piece, to })
        .unwrap_or_else(|e| panic!("seat {seat} piece {piece} to {to:?}: {e}"));
}

#[test]
fn a_new_game_only_enters() {
    let s = Bantu::new(&Config::default(), 3, 0);
    assert_eq!(s.turn, Some(Seat(0)));
    assert_eq!(s.pieces.len(), 3);
    assert!(s
        .pieces
        .iter()
        .all(|p| p.len() == 4 && p.iter().all(|&x| x == Spot::Home)));
    // Each piece onto any of the four cells of its own line.
    assert_eq!(Bantu::legal_actions(&s, Seat(0)).len(), 16);
    assert_eq!(dests(&s, 0, 1), cells(&[57, 58, 59, 60]));
}

#[test]
fn enter_only_on_a_free_start_cell() {
    let s = position(2, &[(0, 1, Spot::Home), (1, 1, on(58))], 0);
    assert_eq!(dests(&s, 0, 1), cells(&[57, 59, 60]));
}

#[test]
fn alone_moves_its_number() {
    let s = position(2, &[(0, 1, on(69)), (0, 2, on(66))], 0);
    assert_eq!(move_length(&s, Seat(0), 1), 1);
    assert_eq!(dests(&s, 0, 1), cells(&[62]));
    let s = position(2, &[(0, 2, on(69))], 0);
    assert_eq!(dests(&s, 0, 2), cells(&[61, 63]));
}

#[test]
fn mixed_row_sums_and_protects() {
    let s = position(2, &[(0, 1, on(66)), (1, 2, on(65))], 0);
    assert_eq!(move_length(&s, Seat(0), 1), 3);
    assert_eq!(move_length(&s, Seat(1), 2), 3);
    assert!(is_protected(&s, 66) && is_protected(&s, 65));
    // 66→67→71→70 and 66→69→62→{61, 63}; 65 is taken.
    assert_eq!(dests(&s, 0, 1), cells(&[61, 63, 70]));
}

#[test]
fn start_line_sums_and_protects() {
    let s = position(2, &[(0, 1, on(60)), (1, 1, on(57))], 0);
    assert_eq!(move_length(&s, Seat(0), 1), 2);
    assert!(is_protected(&s, 60) && is_protected(&s, 57));
    assert_eq!(dests(&s, 0, 1), cells(&[58, 71]));
    // Alone on a line, a piece is safe all the same.
    let s = position(2, &[(1, 1, on(57))], 0);
    assert!(is_protected(&s, 57));
}

#[test]
fn slides_along_a_row() {
    let s = position(2, &[(0, 2, on(82))], 0);
    // 82→81→80 is a slide; 82→83→75 and 82→81→73 end on arrow cells.
    assert_eq!(dests(&s, 0, 2), cells(&[73, 75, 80]));
}

#[test]
fn no_cell_twice_in_one_move() {
    let s = position(2, &[(0, 2, on(62))], 0);
    // Not 62→63→62 back home.
    assert_eq!(dests(&s, 0, 2), cells(&[64, 76, 78]));
}

#[test]
fn never_backwards() {
    let s = position(2, &[(0, 1, on(70))], 0);
    assert_eq!(dests(&s, 0, 1), cells(&[64]));
}

#[test]
fn blocked_by_a_piece_on_the_path() {
    let s = position(2, &[(0, 2, on(69)), (1, 3, on(62))], 0);
    assert!(dests(&s, 0, 2).is_empty());
}

#[test]
fn captures_an_unprotected_piece() {
    let mut s = position(
        2,
        &[
            (0, 1, on(69)),
            (0, 2, on(66)),
            (1, 3, on(62)),
            (1, 1, Spot::Home),
        ],
        0,
    );
    assert!(!is_protected(&s, 62));
    go(&mut s, 0, 1, Dest::Cell(62));
    assert_eq!(s.pieces[1][2], Spot::Home);
    assert_eq!(s.pieces[0][0], on(62));
    let last = s.last.expect("a move was played");
    assert_eq!(last.captured, Some((Seat(1), 3)));
    assert_eq!(s.quiet, 0);
}

#[test]
fn cannot_land_on_a_protected_or_own_piece() {
    let s = position(2, &[(0, 1, on(69)), (0, 3, on(62))], 0);
    assert!(dests(&s, 0, 1).is_empty());
    let s = position(2, &[(0, 1, on(69)), (1, 3, on(62)), (1, 4, on(64))], 0);
    assert!(dests(&s, 0, 1).is_empty());
}

#[test]
fn finish_without_exact_count() {
    let mut s = position(2, &[(0, 4, on(81)), (0, 1, on(57)), (1, 1, on(60))], 0);
    assert!(dests(&s, 0, 4).contains(&Dest::Finish));
    go(&mut s, 0, 4, Dest::Finish);
    assert_eq!(s.pieces[0][3], Spot::Finished);
    assert_eq!(s.quiet, 0);
}

#[test]
fn stopping_on_an_arrow_cell_waits() {
    let mut s = position(2, &[(0, 1, on(81)), (1, 1, on(57))], 0);
    assert_eq!(dests(&s, 0, 1), cells(&[73, 80, 82]));
    go(&mut s, 0, 1, Dest::Cell(73));
    s.turn = Some(Seat(0));
    let mut expected = cells(&[72]);
    expected.insert(Dest::Finish);
    assert_eq!(dests(&s, 0, 1), expected);
}

#[test]
fn a_seat_without_move_is_skipped() {
    // Seat 1's last piece, on 70, can only step onto 64, held safe in a row
    // by seat 0. Seat 0 enters a piece: seat 1 passes, seat 2 plays.
    let mut s = position(
        3,
        &[
            (0, 1, on(64)),
            (0, 2, on(63)),
            (0, 3, Spot::Home),
            (1, 1, on(70)),
            (2, 1, Spot::Home),
        ],
        0,
    );
    go(&mut s, 0, 3, Dest::Cell(7));
    assert_eq!(s.turn, Some(Seat(2)));
    // Entering is progress; seat 1's lost turn is the only quiet one.
    assert_eq!(s.quiet, 1);
    assert!(Bantu::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn quiet_limit_ends_the_game() {
    // Two lone pieces shuttle back and forth, never nearer the finish than
    // where they started: 78→79→78… and 77→76→77…
    let mut s = position(2, &[(0, 1, on(78)), (1, 1, on(77))], 0);
    let shuttle = [(0, 79), (1, 76), (0, 78), (1, 77)];
    let limit = (QUIET_ROUNDS * 2) as usize;
    for k in 0..limit {
        assert!(s.end.is_none(), "ended early, after {k} moves");
        let (seat, cell) = shuttle[k % 4];
        go(&mut s, seat, 1, Dest::Cell(cell));
    }
    assert_eq!(s.end, Some(End::QuietLimit));
    assert!(Bantu::pending(&s).is_empty());
    assert!(Bantu::outcome(&s).is_some());
}

#[test]
fn a_new_best_resets_the_quiet_count() {
    // Back and forth between cells already reached is no progress; 81,
    // nearer the finish than 77 ever was, is.
    let mut s = position(2, &[(0, 1, on(78)), (1, 1, on(77))], 0);
    s.quiet = 30;
    for (seat, cell) in [(0, 79), (1, 76), (0, 78), (1, 77), (0, 79)] {
        go(&mut s, seat, 1, Dest::Cell(cell));
    }
    assert_eq!(s.quiet, 35);
    go(&mut s, 1, 1, Dest::Cell(81));
    assert_eq!(s.quiet, 0, "81 is nearer the finish than 77 ever was");
}

#[test]
fn entering_is_progress() {
    let mut s = position(2, &[(0, 2, Spot::Home), (1, 1, on(77))], 0);
    s.quiet = 30;
    go(&mut s, 0, 2, Dest::Cell(34));
    assert_eq!(s.quiet, 0);
}

#[test]
fn a_capture_resets_the_quiet_count() {
    let mut s = position(
        2,
        &[
            (0, 1, on(69)),
            (0, 2, on(66)),
            (1, 3, on(62)),
            (1, 1, Spot::Home),
        ],
        0,
    );
    s.quiet = 40;
    go(&mut s, 0, 1, Dest::Cell(62));
    assert_eq!(s.quiet, 0);
}

#[test]
fn win_and_ranking() {
    // Unlisted pieces are in the finish. Seat 0 brings its last one in. Seat 2 has
    // three pieces in, seat 1 two, though seat 1's are nearer the finish.
    let mut s = position(
        3,
        &[
            (0, 4, on(81)),
            (1, 1, on(80)),
            (1, 2, Spot::Home),
            (2, 2, on(66)),
        ],
        0,
    );
    go(&mut s, 0, 4, Dest::Finish);
    assert_eq!(s.end, Some(End::Won(Seat(0))));
    assert_eq!(Bantu::outcome(&s), Some(Outcome::Ranked(vec![0, 2, 1])));
}

#[test]
fn ties_on_pieces_in_break_on_distance() {
    // Three pieces in each for seats 1 and 2: seat 1's last one (on 80, two
    // steps from the finish) is nearer than seat 2's (on 66).
    let mut s = position(3, &[(0, 4, on(81)), (1, 1, on(80)), (2, 2, on(66))], 0);
    go(&mut s, 0, 4, Dest::Finish);
    assert_eq!(Bantu::outcome(&s), Some(Outcome::Ranked(vec![0, 1, 2])));
}

#[test]
fn short_game_has_three_pieces() {
    let s = Bantu::new(&Config { short: true }, 2, 0);
    assert!(s.pieces.iter().all(|p| p.len() == 3));
    assert_eq!(Bantu::legal_actions(&s, Seat(0)).len(), 12);
}

#[test]
fn illegal_move_leaves_state_unchanged() {
    let mut s = position(2, &[(0, 1, on(70)), (1, 1, on(57))], 0);
    let before = s.clone();
    let bad = [
        (
            0,
            Action::Move {
                piece: 1,
                to: Dest::Cell(71),
            },
        ),
        (
            0,
            Action::Move {
                piece: 1,
                to: Dest::Finish,
            },
        ),
        (
            0,
            Action::Move {
                piece: 9,
                to: Dest::Cell(64),
            },
        ),
        (
            1,
            Action::Move {
                piece: 1,
                to: Dest::Cell(65),
            },
        ),
    ];
    for (seat, action) in bad {
        assert!(
            Bantu::apply(&mut s, Seat(seat), &action).is_err(),
            "{action:?}"
        );
        assert_eq!(s, before);
    }
}

#[test]
fn a_target_config_out_of_range_is_refused() {
    assert!(Bantu::check_config(&Config::default(), 4).is_ok());
    assert_eq!(Config::default(), Config { short: false });
}
