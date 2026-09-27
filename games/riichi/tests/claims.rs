use ludo_core::{Game, Seat, Viewer};
use riichi::testing::wall;
use riichi::{Action, Riichi, State, Tile};

fn t(text: &str) -> Tile {
    Tile::parse(text).unwrap()
}

/// Dealer (seat 0) draws N and can discard 5p. Seat 1 waits on 5p (tanyao)
/// and could also chi it; seat 3 holds two 5p and could pon it.
fn ron_or_pon() -> State {
    State::start(
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
    )
}

/// Same, but seat 3 holds two E and the dealer discards E.
fn pon_only() -> State {
    State::start(
        1,
        Some(wall(
            [
                "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
                "E E S S S W W N N P P F F",
            ],
            &["N"],
        )),
    )
}

fn play(s: &mut State, seat: u8, action: Action) {
    Riichi::apply(s, Seat(seat), &action)
        .unwrap_or_else(|e| panic!("siège {seat} {action:?} : {e}"));
}

fn json<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap()
}

#[test]
fn the_dealer_acts_first() {
    let s = ron_or_pon();
    assert_eq!(Riichi::pending(&s).0, vec![Seat(0)]);
    assert!(Riichi::legal_actions(&s, Seat(0)).contains(&Action::Discard { tile: t("5p") }));
    assert!(Riichi::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn a_discard_asks_only_the_players_who_can_call() {
    let mut s = ron_or_pon();
    play(&mut s, 0, Action::Discard { tile: t("5p") });
    assert_eq!(Riichi::pending(&s).0, vec![Seat(1), Seat(3)]);
    let seat1 = Riichi::legal_actions(&s, Seat(1));
    assert!(seat1.contains(&Action::Ron) && seat1.contains(&Action::Pass));
    assert!(Riichi::legal_actions(&s, Seat(3))
        .iter()
        .any(|a| matches!(a, Action::Pon { .. })));
    assert!(Riichi::legal_actions(&s, Seat(2)).is_empty());
}

#[test]
fn nobody_learns_who_else_may_call() {
    let mut s = ron_or_pon();
    play(&mut s, 0, Action::Discard { tile: t("5p") });
    assert_eq!(
        Riichi::visible_pending(&s, Viewer::Seat(Seat(1))).0,
        vec![Seat(1)]
    );
    assert_eq!(
        Riichi::visible_pending(&s, Viewer::Seat(Seat(3))).0,
        vec![Seat(3)]
    );
    assert!(Riichi::visible_pending(&s, Viewer::Seat(Seat(2)))
        .0
        .is_empty());
    assert!(Riichi::visible_pending(&s, Viewer::Spectator).0.is_empty());
}

#[test]
fn answers_wait_until_everyone_has_answered() {
    let mut s = ron_or_pon();
    play(&mut s, 0, Action::Discard { tile: t("5p") });
    let pon = Riichi::legal_actions(&s, Seat(3))
        .into_iter()
        .find(|a| matches!(a, Action::Pon { .. }))
        .unwrap();
    let before = json(&Riichi::view(&s, Viewer::Seat(Seat(0))));
    play(&mut s, 3, pon.clone());
    assert_eq!(Riichi::pending(&s).0, vec![Seat(1)]);
    assert_eq!(
        json(&Riichi::view(&s, Viewer::Seat(Seat(0)))),
        before,
        "nothing happens yet"
    );
    assert!(Riichi::view(&s, Viewer::Seat(Seat(3))).answered);
    assert!(
        Riichi::apply(&mut s, Seat(3), &pon).is_err(),
        "a second answer is refused"
    );
}

#[test]
fn ron_beats_pon_whatever_the_order() {
    let finish = |first_pon: bool| {
        let mut s = ron_or_pon();
        play(&mut s, 0, Action::Discard { tile: t("5p") });
        let pon = Riichi::legal_actions(&s, Seat(3))
            .into_iter()
            .find(|a| matches!(a, Action::Pon { .. }))
            .unwrap();
        if first_pon {
            play(&mut s, 3, pon);
            play(&mut s, 1, Action::Ron);
        } else {
            play(&mut s, 1, Action::Ron);
            play(&mut s, 3, pon);
        }
        s
    };
    let a = finish(true);
    let b = finish(false);
    for seat in 0..4 {
        assert_eq!(
            json(&Riichi::view(&a, Viewer::Seat(Seat(seat)))),
            json(&Riichi::view(&b, Viewer::Seat(Seat(seat))))
        );
    }
    for s in [&a, &b] {
        let scores = s.scores();
        assert!(scores[1] > 25_000 && scores[0] < 25_000, "{scores:?}");
        assert_eq!(
            Riichi::view(s, Viewer::Spectator).round.hand,
            2,
            "the dealer lost: next hand"
        );
    }
    assert_eq!(a.scores(), b.scores());
}

#[test]
fn a_pon_gives_the_turn_to_the_caller() {
    let mut s = pon_only();
    play(&mut s, 0, Action::Discard { tile: t("E") });
    assert_eq!(Riichi::pending(&s).0, vec![Seat(3)]);
    let pon = Riichi::legal_actions(&s, Seat(3))
        .into_iter()
        .find(|a| matches!(a, Action::Pon { .. }))
        .unwrap();
    play(&mut s, 3, pon);
    assert_eq!(Riichi::pending(&s).0, vec![Seat(3)]);
    let v = Riichi::view(&s, Viewer::Seat(Seat(3)));
    let meld = &v.players[3].melds[0];
    assert_eq!(meld.from, Some(Seat(0)));
    assert_eq!(meld.tiles.iter().filter(|x| x.notation() == "E").count(), 3);
    assert!(Riichi::legal_actions(&s, Seat(3))
        .iter()
        .all(|a| matches!(a, Action::Discard { .. })));
}

#[test]
fn waiting_answers_survive_a_serde_round_trip() {
    let mut s = ron_or_pon();
    play(&mut s, 0, Action::Discard { tile: t("5p") });
    play(&mut s, 3, Action::Pass);
    let back: State = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(Riichi::pending(&back).0, vec![Seat(1)]);
    assert!(Riichi::view(&back, Viewer::Seat(Seat(3))).answered);
    for seat in 0..4 {
        assert_eq!(
            json(&Riichi::view(&back, Viewer::Seat(Seat(seat)))),
            json(&Riichi::view(&s, Viewer::Seat(Seat(seat))))
        );
    }
}

#[test]
fn the_view_hides_other_hands_and_splits_the_drawn_tile() {
    let s = ron_or_pon();
    let v = Riichi::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(v.hand.len(), 13);
    assert_eq!(v.drawn.map(Tile::notation).as_deref(), Some("N"));
    assert_eq!(v.players[1].hand_size, 13);
    let other = Riichi::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(other.drawn, None);
    assert_eq!(other.hand.len(), 13);
    let json = json(&v);
    assert!(
        json.get("players").unwrap()[1].get("hand").is_none(),
        "no other hand in the view"
    );
    assert_eq!(
        v.round.wall_left, 69,
        "136 - 52 dealt - 1 drawn - 14 dead wall"
    );
    assert_eq!(v.players[0].seat_wind, riichi::Wind::E);
    assert_eq!(v.players[1].seat_wind, riichi::Wind::S);
}

#[test]
fn timeout_actions() {
    let mut s = ron_or_pon();
    assert_eq!(
        Riichi::timeout_action(&s, Seat(0)),
        Some(Action::Tsumogiri { tile: t("N") }),
        "tsumogiri"
    );
    assert_eq!(Riichi::timeout_action(&s, Seat(1)), None, "not pending");
    play(&mut s, 0, Action::Discard { tile: t("5p") });
    assert_eq!(Riichi::timeout_action(&s, Seat(1)), Some(Action::Pass));
}

#[test]
fn seat_one_sees_its_tanyao_wait() {
    let s = ron_or_pon();
    let help = Riichi::view(&s, Viewer::Seat(Seat(1))).help.unwrap();
    assert_eq!(help.shanten, 0);
    assert_eq!(help.waits, vec![t("2p"), t("5p")], "234p + 5p: 2p-5p wait");
}

#[test]
fn illegal_answers_change_nothing() {
    let mut s = ron_or_pon();
    let before = json(&s);
    assert!(Riichi::apply(&mut s, Seat(0), &Action::Discard { tile: t("C") }).is_err());
    assert!(Riichi::apply(&mut s, Seat(2), &Action::Pass).is_err());
    assert_eq!(json(&s), before);
}

/// Seat 1 holds a 2m and draws another: which copy leaves is public. (Not the
/// dealer: Mahjong Soul counts the dealer's first discard as from the hand.)
fn drawn_copy() -> State {
    let mut s = State::start(
        1,
        Some(wall(
            [
                "1m 1m 1m 9m 9m 9m 1s 1s 1s 9s 9s E 5p",
                "2m 3m 4m 5m 6m 7m 2p 3p 4p 6s 7s 8s 5p",
                "1p 1p 1p 9p 9p 9p 2s 2s 2s 3s 3s 3s W",
                "5p 5pr S S S W W N N P P F F",
            ],
            &["N", "2m"],
        )),
    );
    play(&mut s, 0, Action::Tsumogiri { tile: t("N") });
    for seat in Riichi::pending(&s).0 {
        if seat != Seat(1) {
            play(&mut s, seat.0, Action::Pass);
        }
    }
    s
}

#[test]
fn discarding_the_drawn_tile_is_its_own_move() {
    let s = drawn_copy();
    assert_eq!(Riichi::pending(&s).0, vec![Seat(1)]);
    let legal = Riichi::legal_actions(&s, Seat(1));
    assert!(
        legal.contains(&Action::Tsumogiri { tile: t("2m") }),
        "{legal:?}"
    );
    assert!(
        legal.contains(&Action::Discard { tile: t("2m") }),
        "a copy from the hand"
    );
    assert_eq!(
        Riichi::timeout_action(&s, Seat(1)),
        Some(Action::Tsumogiri { tile: t("2m") })
    );

    let mut drawn = s.clone();
    play(&mut drawn, 1, Action::Tsumogiri { tile: t("2m") });
    let pond = &Riichi::view(&drawn, Viewer::Seat(Seat(2))).players[1].discards;
    assert!(pond[0].tsumogiri, "{pond:?}");

    let mut kept = s.clone();
    play(&mut kept, 1, Action::Discard { tile: t("2m") });
    let pond = &Riichi::view(&kept, Viewer::Seat(Seat(2))).players[1].discards;
    assert!(!pond[0].tsumogiri, "{pond:?}");
}
