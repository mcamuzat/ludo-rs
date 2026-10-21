use black_pot::board::{board_json, cell_at, cells, kind_of, windows, Kind};

/// The values of page 4 of the rules, top to bottom, left to right (spec §4).
const GRID: [&[u8]; 11] = [
    &[19, 8, 3, 7, 5, 7, 17],
    &[1, 12, 17, 9, 2, 8, 13, 11, 1],
    &[19, 11, 13, 10, 3, 20, 4, 9, 14, 12, 17],
    &[2, 14, 16, 2, 4, 23, 3, 1, 15, 21, 2],
    &[5, 15, 4, 6, 21, 18, 21, 5, 3, 16, 6],
    &[1, 2, 10, 22, 24, 23, 24, 22, 10, 2, 1],
    &[6, 16, 3, 5, 21, 18, 21, 6, 4, 15, 5],
    &[2, 21, 15, 1, 3, 23, 4, 2, 16, 14, 2],
    &[17, 12, 14, 9, 4, 20, 3, 10, 13, 11, 19],
    &[1, 11, 13, 8, 2, 9, 17, 12, 1],
    &[17, 7, 5, 7, 3, 8, 19],
];

#[test]
fn the_board_has_109_cells_in_centred_rows() {
    assert_eq!(cells().len(), 109);
    let lengths: Vec<usize> = (0..11u8)
        .map(|r| cells().iter().filter(|c| c.row == r).count())
        .collect();
    assert_eq!(lengths, [7, 9, 11, 11, 11, 11, 11, 11, 11, 9, 7]);
    for (r, row) in GRID.iter().enumerate() {
        let off = (11 - row.len()) / 2;
        for (i, &value) in row.iter().enumerate() {
            let id = cell_at(r as i8, (off + i) as i8).expect("cell exists");
            assert_eq!(cells()[usize::from(id)].value, value, "row {r}, cell {i}");
        }
    }
    assert_eq!(cell_at(0, 0), None);
    assert_eq!(cell_at(0, 1), None);
    assert_eq!(cell_at(1, 0), None);
    assert_eq!(cell_at(11, 5), None);
    assert_eq!(cell_at(-1, 5), None);
}

#[test]
fn ids_follow_reading_order() {
    for (id, cell) in cells().iter().enumerate() {
        assert_eq!(
            cell_at(cell.row as i8, cell.col as i8),
            Some(id as u8),
            "cell {id}"
        );
    }
}

#[test]
fn kinds_follow_values() {
    for cell in cells() {
        assert_eq!(Some(cell.kind), kind_of(cell.value));
    }
    let count = |k: Kind| cells().iter().filter(|c| c.kind == k).count();
    assert_eq!(count(Kind::Lemon), 42);
    assert_eq!(count(Kind::Cherry), 38);
    assert_eq!(count(Kind::BlackPot), 29);
    assert_eq!(kind_of(0), None);
    assert_eq!(kind_of(25), None);
    assert_eq!(kind_of(6), Some(Kind::Lemon));
    assert_eq!(kind_of(9), Some(Kind::Cherry));
    assert_eq!(kind_of(13), Some(Kind::BlackPot));
}

#[test]
fn the_board_is_symmetric_by_a_half_turn() {
    for cell in cells() {
        let other = cell_at(10 - cell.row as i8, 10 - cell.col as i8).expect("mirror cell");
        assert_eq!(cells()[usize::from(other)].value, cell.value);
    }
}

#[test]
fn there_are_216_windows_none_with_five_black_pots() {
    assert_eq!(windows().len(), 216);
    let black_pots = |w: &black_pot::board::Window| {
        w.cells
            .iter()
            .filter(|&&c| cells()[usize::from(c)].kind == Kind::BlackPot)
            .count()
    };
    assert!(windows().iter().all(|w| black_pots(w) < 5));
    assert_eq!(windows().iter().filter(|w| black_pots(w) == 4).count(), 20);
    for w in windows() {
        let (dr, dc) = w.dir.step();
        let first = &cells()[usize::from(w.cells[0])];
        for (k, &c) in w.cells.iter().enumerate() {
            let k = k as i8;
            assert_eq!(
                cell_at(first.row as i8 + dr * k, first.col as i8 + dc * k),
                Some(c)
            );
        }
    }
}

#[test]
fn board_json_is_up_to_date() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../web/src/games/black-pot/board.json"
    );
    let fresh = serde_json::to_string_pretty(&board_json()).unwrap() + "\n";
    if std::env::var_os("UPDATE_BOARD_JSON").is_some() {
        std::fs::write(path, &fresh).unwrap();
    }
    let stored = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        stored == fresh,
        "web/src/games/black-pot/board.json is stale: rerun with UPDATE_BOARD_JSON=1"
    );
}
