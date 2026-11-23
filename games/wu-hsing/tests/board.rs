use wu_hsing::{valid_cycle, values, Board, Dir, Placement};

/// The first domino with these values (in either order), and whether it is
/// flipped to put `first` on the anchor.
fn domino(first: u8, second: u8) -> (u8, bool) {
    (0..30)
        .find_map(|id| match values(id) {
            (a, b) if (a, b) == (first, second) => Some((id, false)),
            (a, b) if (b, a) == (first, second) => Some((id, true)),
            _ => None,
        })
        .expect("values 1 to 5")
}

fn at(first: u8, second: u8, row: i8, col: i8, dir: Dir) -> Placement {
    let (domino, flipped) = domino(first, second);
    Placement {
        domino,
        row,
        col,
        dir,
        flipped,
    }
}

/// Starting square: 1-2 over 3-4, at (0,0).
fn start() -> Board {
    Board::new([domino(1, 2).0, domino(3, 4).0])
}

#[test]
fn cycles_accept_and_reject() {
    let valid: [&[u8]; 9] = [
        &[],
        &[1],
        &[1, 1, 1],
        &[1, 2, 1, 2],
        &[1, 2, 3, 1, 2, 3],
        &[1, 2, 3, 4, 1, 2, 3, 4],
        &[1, 2, 3, 4, 5, 1, 2, 3, 4, 5],
        &[1, 2, 3],
        &[1, 2],
    ];
    for v in valid {
        assert!(valid_cycle(v), "{v:?}");
    }
    let invalid: [&[u8]; 3] = [&[1, 1, 2], &[1, 2, 1, 3], &[1, 2, 3, 1, 2, 4]];
    for v in invalid {
        assert!(!valid_cycle(v), "{v:?}");
    }
}

#[test]
fn starting_square_is_laid() {
    let b = start();
    assert_eq!(b.value_at(0, 0), Some(1));
    assert_eq!(b.value_at(0, 1), Some(2));
    assert_eq!(b.value_at(1, 0), Some(3));
    assert_eq!(b.value_at(1, 1), Some(4));
    assert_eq!(b.value_at(2, 0), None);
    assert_eq!(b.placed().len(), 2);
    assert!(b.placed().iter().all(|p| p.by.is_none()));
}

#[test]
fn own_line_alone_does_not_count() {
    let err = start().evaluate(&at(1, 2, 0, 2, Dir::Right)).unwrap_err();
    assert!(err.0.contains("deux groupes"), "{}", err.0);
}

#[test]
fn bridging_two_columns_scores_doubled() {
    assert_eq!(start().evaluate(&at(5, 5, -1, 0, Dir::Right)), Ok(18));
}

#[test]
fn occupied_cells_are_refused() {
    let err = start().evaluate(&at(5, 5, 1, 1, Dir::Right)).unwrap_err();
    assert!(err.0.contains("pas libres"), "{}", err.0);
}

#[test]
fn breaking_a_cycle_is_refused() {
    // Column 0 would read 1, 3, 3, 5.
    let err = start().evaluate(&at(3, 5, 2, 0, Dir::Down)).unwrap_err();
    assert!(err.0.contains("cycle"), "{}", err.0);
}

#[test]
fn fourth_aligned_domino_is_refused() {
    let mut b = start();
    // A third horizontal domino under the square: columns read 1,3,1 and 2,4,2.
    let third = at(1, 2, 2, 0, Dir::Right);
    assert_eq!(b.evaluate(&third), Ok(9));
    b.place(third, 0);
    let err = b.evaluate(&at(3, 4, 3, 0, Dir::Right)).unwrap_err();
    assert!(err.0.contains("Quatre dominos"), "{}", err.0);
}

/// Rows 10 and 11 hold four `a` and three `b` left of column 10; column 10
/// holds `b` above and `a` below. A vertical a-b at (10,10) then extends a
/// row of 5, a row of 4 and its own column to 4.
fn example_b(a: u8, b: u8) -> Board {
    let mut board = start();
    for p in [
        at(a, a, 10, 6, Dir::Right),
        at(a, a, 10, 8, Dir::Right),
        at(b, b, 11, 7, Dir::Right),
        at(b, 3, 11, 9, Dir::Down),
        at(b, 3, 9, 10, Dir::Right),
        at(a, 3, 12, 10, Dir::Right),
    ] {
        board.place(p, 0);
    }
    board
}

#[test]
fn rulebook_example_b_scores_80() {
    assert_eq!(
        example_b(1, 2).evaluate(&at(1, 2, 10, 10, Dir::Down)),
        Ok(80)
    );
    assert_eq!(
        example_b(1, 1).evaluate(&at(1, 1, 10, 10, Dir::Down)),
        Ok(160)
    );
}

#[test]
fn negative_coordinates_work() {
    let b = start();
    let left = at(5, 5, 0, -1, Dir::Down);
    let above = at(5, 5, -1, 0, Dir::Right);
    assert_eq!(b.evaluate(&left), Ok(18));
    let moves: Vec<Placement> = b
        .moves(&[left.domino])
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    assert!(moves.contains(&left) && moves.contains(&above), "{moves:?}");
}

#[test]
fn moves_skip_flipped_doubles_and_are_ordered() {
    let b = start();
    let moves = b.moves(&[domino(1, 5).0, domino(5, 5).0]);
    assert!(!moves.is_empty());
    assert!(moves
        .iter()
        .all(|(p, _)| !(p.flipped && p.domino == domino(5, 5).0)));
    let keys: Vec<_> = moves
        .iter()
        .map(|(p, _)| (p.domino, p.row, p.col, p.dir, p.flipped))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
    for (p, points) in &moves {
        assert_eq!(b.evaluate(p), Ok(*points));
    }
}

#[test]
fn board_survives_a_json_round_trip() {
    let mut b = start();
    b.place(at(5, 5, -1, 0, Dir::Right), 0);
    b.place(at(1, 2, 2, 0, Dir::Right), 1);
    let hand: Vec<u8> = (0..15).collect();
    let json = serde_json::to_string(&b).unwrap();
    let back: Board = serde_json::from_str(&json).unwrap();
    assert_eq!(back.moves(&hand), b.moves(&hand));
    assert_eq!(back.placed(), b.placed());
}
