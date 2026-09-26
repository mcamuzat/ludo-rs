use ludo_core::{Game, Seat, Viewer};
use riichi::testing::wall;
use riichi::{Action, DrawKind, Riichi, State, Tile};

fn t(text: &str) -> Tile {
    Tile::parse(text).unwrap()
}

fn ron_on_the_first_discard() -> State {
    let mut s = State::start(
        1,
        Some(wall(
            [
                "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
                "5p 5pr S S S W W N N P P F F",
            ],
            &["N"],
        )),
    );
    Riichi::apply(&mut s, Seat(0), &Action::Discard { tile: t("5p") }).unwrap();
    Riichi::apply(&mut s, Seat(3), &Action::Pass).unwrap();
    Riichi::apply(&mut s, Seat(1), &Action::Ron).unwrap();
    s
}

#[test]
fn a_ron_is_reported_with_its_yaku() {
    let s = ron_on_the_first_discard();
    assert_eq!(s.hands_played(), 1);
    let result = Riichi::view(&s, Viewer::Seat(Seat(2)))
        .last_hand
        .expect("a hand ended");
    assert_eq!(result.number, 1);
    assert_eq!(result.draw, None);
    assert_eq!(result.wins.len(), 1);
    let win = &result.wins[0];
    assert_eq!(win.seat, Seat(1));
    assert_eq!(win.from, Some(Seat(0)));
    assert_eq!(win.win_tile, t("5p"));
    assert_eq!(win.hand.len(), 14);
    assert!(
        win.yaku.iter().any(|y| y.name == "Tanyao"),
        "{:?}",
        win.yaku
    );
    assert!(
        win.han >= 1 && win.fu >= 30,
        "{} han {} fu",
        win.han,
        win.fu
    );
    assert_eq!(result.deltas[1], win.points as i32, "no sticks, no honba");
    assert_eq!(result.deltas.iter().sum::<i32>(), 0);
    assert!(result.deltas[0] < 0 && result.deltas[2] == 0 && result.deltas[3] == 0);
}

#[test]
fn every_seat_sees_the_same_result() {
    let s = ron_on_the_first_discard();
    let first = Riichi::view(&s, Viewer::Seat(Seat(0))).last_hand;
    for seat in 1..4 {
        assert_eq!(Riichi::view(&s, Viewer::Seat(Seat(seat))).last_hand, first);
    }
    assert_eq!(Riichi::view(&s, Viewer::Spectator).last_hand, first);
}

#[test]
fn a_hand_nobody_wins_ends_in_a_draw() {
    let mut s = State::start(11, None);
    for _ in 0..1000 {
        if s.hands_played() == 1 {
            break;
        }
        let seat = Riichi::pending(&s).0[0];
        let action = Riichi::timeout_action(&s, seat).expect("a pending seat has a default");
        Riichi::apply(&mut s, seat, &action).unwrap();
    }
    let result = Riichi::view(&s, Viewer::Spectator)
        .last_hand
        .expect("the hand ended");
    assert!(result.wins.is_empty());
    assert!(
        matches!(result.draw, Some(DrawKind::Exhaustive | DrawKind::Abortive)),
        "{:?}",
        result.draw
    );
    assert_eq!(result.deltas.iter().sum::<i32>(), 0);
}

#[test]
fn nothing_is_reported_before_the_first_hand_ends() {
    let s = State::start(3, None);
    assert_eq!(s.hands_played(), 0);
    assert_eq!(Riichi::view(&s, Viewer::Spectator).last_hand, None);
}
