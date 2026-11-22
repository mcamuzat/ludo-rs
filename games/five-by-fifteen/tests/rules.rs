use std::collections::BTreeSet;

use five_by_fifteen::testing::state;
use five_by_fifteen::{
    Action, Color, Config, FiveByFifteen, Locomotive, Pos, Route, State, Status, Tile, COLORS,
};
use ludo_core::{Game, Outcome, Seat, Viewer};

type Cells = [[Option<Tile>; 15]; 5];

fn t(color: Color, number: u8) -> Option<Tile> {
    Some(Tile { color, number })
}

fn pos(row: u8, col: u8) -> Pos {
    Pos { row, col }
}

fn empty() -> Cells {
    [[None; 15]; 5]
}

/// Every row sorted (anchors in `COLORS` order), the last column empty.
fn solved() -> Cells {
    let mut cells = empty();
    for (r, color) in COLORS.into_iter().enumerate() {
        for (c, cell) in cells[r].iter_mut().take(14).enumerate() {
            *cell = t(color, c as u8 + 2);
        }
    }
    cells
}

fn places_to(s: &State, to: Pos) -> BTreeSet<Pos> {
    FiveByFifteen::legal_actions(s, Seat(0))
        .into_iter()
        .filter_map(|a| match a {
            Action::Place { from, to: t } if t == to => Some(from),
            _ => None,
        })
        .collect()
}

fn places_from(s: &State, from: Pos) -> BTreeSet<Pos> {
    FiveByFifteen::legal_actions(s, Seat(0))
        .into_iter()
        .filter_map(|a| match a {
            Action::Place { from: f, to } if f == from => Some(to),
            _ => None,
        })
        .collect()
}

fn shifts(s: &State) -> Vec<Action> {
    FiveByFifteen::legal_actions(s, Seat(0))
        .into_iter()
        .filter(|a| matches!(a, Action::Shift { .. }))
        .collect()
}

fn play(s: &mut State, a: Action) {
    FiveByFifteen::apply(s, Seat(0), &a).expect("a legal move");
}

/// Amber 13-14-15 in row 3, columns 8 to 10, the gap after it; a Violet 9
/// right of the gap and the Violet 8 elsewhere.
fn engine(route: Route) -> State {
    let mut cells = empty();
    cells[3][8] = t(Color::Amber, 13);
    cells[3][9] = t(Color::Amber, 14);
    cells[3][10] = t(Color::Amber, 15);
    cells[3][12] = t(Color::Violet, 9);
    cells[0][5] = t(Color::Violet, 8);
    state(COLORS, cells, route)
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let s = FiveByFifteen::new(&Config::default(), 1, 7);
    let first: Vec<String> = s.cells[0]
        .iter()
        .map(|c| c.map_or("-".into(), |t| format!("{:?}{}", t.color, t.number)))
        .collect();
    use Color::*;
    assert_eq!(s.anchors, [Crimson, Azure, Amber, Emerald, Violet]);
    assert_eq!(
        first.join(" "),
        "Crimson14 Amber2 Violet4 Violet5 - Azure3 Violet7 Amber8 Emerald7 Azure15 \
         Emerald2 Amber7 Azure11 Azure6 Azure10"
    );
}

#[test]
fn a_deal_has_every_tile_once_and_five_gaps() {
    for seed in 0..20 {
        let s = FiveByFifteen::new(&Config::default(), 1, seed);
        let tiles: BTreeSet<(Color, u8)> = s
            .cells
            .iter()
            .flatten()
            .flatten()
            .map(|t| (t.color, t.number))
            .collect();
        assert_eq!(tiles.len(), 70);
        assert!(tiles.iter().all(|&(_, n)| (2..=15).contains(&n)));
        assert_eq!(s.cells.iter().flatten().filter(|c| c.is_none()).count(), 5);
        let anchors: BTreeSet<Color> = s.anchors.into_iter().collect();
        assert_eq!(anchors.len(), 5);
        assert_eq!(s.status, Status::Playing);
    }
}

#[test]
fn a_tile_goes_after_its_predecessor_or_before_its_successor() {
    let mut cells = empty();
    cells[1][3] = t(Color::Crimson, 6);
    cells[4][10] = t(Color::Crimson, 7);
    cells[2][5] = t(Color::Azure, 6);
    cells[3][0] = t(Color::Crimson, 5);
    cells[0][7] = t(Color::Crimson, 8);
    let s = state(COLORS, cells, Route::Short);
    assert_eq!(
        places_from(&s, pos(4, 10)),
        BTreeSet::from([pos(1, 4), pos(0, 6)])
    );
}

#[test]
fn the_first_column_takes_the_anchor_s_two() {
    let mut cells = empty();
    cells[0][5] = t(Color::Azure, 2);
    cells[3][3] = t(Color::Emerald, 2);
    cells[4][8] = t(Color::Azure, 3);
    let s = state(COLORS, cells, Route::Short);
    assert_eq!(places_to(&s, pos(2, 0)), BTreeSet::from([pos(0, 5)]));
}

#[test]
fn nothing_goes_after_a_fifteen_but_a_shift() {
    let mut s = engine(Route::Short);
    assert_eq!(places_to(&s, pos(3, 11)), BTreeSet::from([pos(0, 5)]));
    assert_eq!(shifts(&s), vec![Action::Shift { row: 3, to: 11 }]);
    play(&mut s, Action::Shift { row: 3, to: 11 });
    assert_eq!(s.cells[3][8], None);
    assert_eq!(s.cells[3][9], t(Color::Amber, 13));
    assert_eq!(s.cells[3][10], t(Color::Amber, 14));
    assert_eq!(s.cells[3][11], t(Color::Amber, 15));
}

#[test]
fn a_shift_moves_the_whole_run() {
    let mut cells = empty();
    for (i, n) in (11..=15).enumerate() {
        cells[0][2 + i] = t(Color::Emerald, n);
    }
    let mut s = state(COLORS, cells, Route::Long);
    play(&mut s, Action::Shift { row: 0, to: 7 });
    assert_eq!(s.cells[0][2], None);
    for (i, n) in (11..=15).enumerate() {
        assert_eq!(s.cells[0][3 + i], t(Color::Emerald, n));
    }
}

#[test]
fn a_short_run_cannot_shift_on_a_longer_route() {
    assert!(shifts(&engine(Route::Medium)).is_empty());
    assert!(shifts(&engine(Route::Long)).is_empty());
}

#[test]
fn nothing_leaves_the_board() {
    let mut cells = empty();
    cells[3][12] = t(Color::Amber, 13);
    cells[3][13] = t(Color::Amber, 14);
    cells[3][14] = t(Color::Amber, 15);
    cells[0][3] = t(Color::Emerald, 2);
    let s = state(COLORS, cells, Route::Short);
    assert!(shifts(&s).is_empty());
}

#[test]
fn a_sorted_row_is_done() {
    let mut cells = solved();
    cells[4][0] = None;
    cells[4][14] = t(Color::Violet, 2);
    let s = state(COLORS, cells, Route::Short);
    assert_eq!(s.status, Status::Playing);
    assert!(shifts(&s).is_empty(), "{:?}", shifts(&s));
}

#[test]
fn illegal_moves_leave_the_state_unchanged() {
    let s = engine(Route::Short);
    let illegal = [
        Action::Place {
            from: pos(3, 11),
            to: pos(2, 0),
        },
        Action::Place {
            from: pos(0, 5),
            to: pos(3, 10),
        },
        Action::Place {
            from: pos(3, 12),
            to: pos(2, 0),
        },
        Action::Place {
            from: pos(0, 200),
            to: pos(9, 0),
        },
        Action::Shift { row: 0, to: 3 },
        Action::Shift { row: 9, to: 3 },
        Action::Shift { row: 3, to: 0 },
    ];
    for a in illegal {
        let mut after = s.clone();
        assert!(
            FiveByFifteen::apply(&mut after, Seat(0), &a).is_err(),
            "{a:?}"
        );
        assert_eq!(format!("{after:?}"), format!("{s:?}"));
    }
    let mut after = s.clone();
    let shift = Action::Shift { row: 3, to: 11 };
    assert!(FiveByFifteen::apply(&mut after, Seat(1), &shift).is_err());
}

#[test]
fn sorting_every_row_wins() {
    let mut cells = solved();
    cells[4][13] = None;
    cells[0][14] = t(Color::Violet, 15);
    let mut s = state(COLORS, cells, Route::Short);
    assert_eq!(s.status, Status::Playing);
    play(
        &mut s,
        Action::Place {
            from: pos(0, 14),
            to: pos(4, 13),
        },
    );
    assert_eq!(s.status, Status::Won);
    assert!(FiveByFifteen::pending(&s).is_empty());
    assert_eq!(FiveByFifteen::outcome(&s), Some(Outcome::Ranked(vec![0])));
}

#[test]
fn no_move_left_is_stuck() {
    let mut cells = empty();
    cells[1][5] = t(Color::Emerald, 2);
    let mut s = state(COLORS, cells, Route::Short);
    assert_eq!(s.status, Status::Playing);
    play(
        &mut s,
        Action::Place {
            from: pos(1, 5),
            to: pos(0, 0),
        },
    );
    assert_eq!(s.status, Status::Stuck);
    assert!(FiveByFifteen::pending(&s).is_empty());
    assert!(FiveByFifteen::outcome(&s).is_some());
}

#[test]
fn giving_up_ends_the_game() {
    let mut s = engine(Route::Short);
    assert!(FiveByFifteen::legal_actions(&s, Seat(0)).contains(&Action::GiveUp));
    play(&mut s, Action::GiveUp);
    assert_eq!(s.status, Status::GaveUp);
    assert!(FiveByFifteen::legal_actions(&s, Seat(0)).is_empty());
    assert!(FiveByFifteen::outcome(&s).is_some());
}

#[test]
fn a_position_seen_before_is_flagged() {
    let mut cells = empty();
    cells[1][1] = t(Color::Emerald, 2);
    cells[3][5] = t(Color::Emerald, 4);
    cells[4][9] = t(Color::Emerald, 3);
    let mut s = state(COLORS, cells, Route::Short);
    let there = Action::Place {
        from: pos(1, 2),
        to: pos(3, 4),
    };
    let back = Action::Place {
        from: pos(3, 4),
        to: pos(1, 2),
    };
    play(
        &mut s,
        Action::Place {
            from: pos(4, 9),
            to: pos(1, 2),
        },
    );
    assert!(!s.repeated);
    play(&mut s, there);
    assert!(!s.repeated);
    play(&mut s, back);
    assert!(s.repeated);
    assert_eq!(s.status, Status::Playing);
    assert_eq!(s.moves, 3);
}

#[test]
fn the_view_shows_sorted_prefix_and_locomotives() {
    let mut s = engine(Route::Short);
    s.cells[0][0] = t(Color::Emerald, 2);
    s.cells[0][1] = t(Color::Emerald, 3);
    s.cells[0][2] = t(Color::Emerald, 4);
    let v = FiveByFifteen::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(v.sorted, [3, 0, 0, 0, 0]);
    assert_eq!(
        v.locomotives,
        vec![Locomotive {
            row: 3,
            col: 10,
            ready: true
        }]
    );
    let medium = FiveByFifteen::view(&engine(Route::Medium), Viewer::Spectator);
    assert_eq!(medium.min_run, 4);
    assert!(!medium.locomotives[0].ready);
}

#[test]
fn actions_serialize_as_the_web_expects() {
    let place = Action::Place {
        from: pos(1, 2),
        to: pos(3, 4),
    };
    assert_eq!(
        serde_json::to_string(&place).unwrap(),
        r#"{"Place":{"from":{"row":1,"col":2},"to":{"row":3,"col":4}}}"#
    );
    assert_eq!(
        serde_json::to_string(&Action::Shift { row: 1, to: 2 }).unwrap(),
        r#"{"Shift":{"row":1,"to":2}}"#
    );
    assert_eq!(
        serde_json::to_string(&Action::GiveUp).unwrap(),
        r#""GiveUp""#
    );
    let v = serde_json::to_value(FiveByFifteen::view(
        &engine(Route::Short),
        Viewer::Spectator,
    ))
    .unwrap();
    for key in [
        "route",
        "minRun",
        "anchors",
        "cells",
        "status",
        "moves",
        "repeated",
        "sorted",
        "locomotives",
    ] {
        assert!(v.get(key).is_some(), "{key}");
    }
}
