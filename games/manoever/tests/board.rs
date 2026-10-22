use manoever::board::{board_json, disc, is_end_square, Disc, Point, BARRACKS, GRAYS, SQUARES};

fn dist(a: Point, b: Point) -> f64 {
    let (dx, dy) = (f64::from(a.x - b.x), f64::from(a.y - b.y));
    (dx * dx + dy * dy).sqrt()
}

#[test]
fn consecutive_squares_are_neighbours() {
    assert_eq!(SQUARES.len(), 84);
    for (n, pair) in SQUARES.windows(2).enumerate() {
        let d = dist(pair[0], pair[1]);
        assert!(d < 120.0, "squares {} and {} are {d} apart", n + 1, n + 2);
    }
}

#[test]
fn nothing_overlaps() {
    let all: Vec<Point> = SQUARES.iter().chain(BARRACKS.iter()).copied().collect();
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert!(dist(*a, *b) >= 40.0, "{a:?} and {b:?} overlap");
        }
    }
}

#[test]
fn barracks_is_a_column_with_place_one_on_top() {
    assert_eq!(BARRACKS.len(), 12);
    for p in BARRACKS.iter() {
        assert!((p.x - BARRACKS[0].x).abs() <= 5, "{p:?} off the column");
    }
    for pair in BARRACKS.windows(2) {
        assert!(pair[0].y < pair[1].y, "places go down from I to XII");
    }
}

#[test]
fn discs_are_where_the_rules_put_them() {
    for sq in 1..=84u8 {
        let expected = match sq {
            12 | 20 | 47 | 65 => Some(Disc::Black),
            8 | 24 | 31 | 36 | 60 | 74 => Some(Disc::White),
            _ => None,
        };
        assert_eq!(disc(sq), expected, "square {sq}");
    }
    assert_eq!(GRAYS, [(25, 1), (40, 2), (55, 3)]);
}

#[test]
fn no_bonus_on_the_end_squares() {
    let ends: Vec<u8> = (1..=84).filter(|&s| is_end_square(s)).collect();
    assert_eq!(ends, [1, 2, 3, 4, 5, 6, 79, 80, 81, 82, 83, 84]);
    for sq in ends {
        assert_eq!(disc(sq), None);
        assert!(GRAYS.iter().all(|&(g, _)| g != sq));
    }
}

#[test]
fn board_json_is_up_to_date() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../web/src/games/manoever/board.json"
    );
    let fresh = serde_json::to_string_pretty(&board_json()).unwrap() + "\n";
    if std::env::var_os("UPDATE_BOARD_JSON").is_some() {
        std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        std::fs::write(path, &fresh).unwrap();
    }
    let on_disk = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        on_disk == fresh,
        "web/src/games/manoever/board.json is stale: rerun with UPDATE_BOARD_JSON=1"
    );
}
