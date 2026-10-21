use black_pot::{BlackPot, Config};
use ludo_core::conformance::{check, check_options};
use ludo_core::{Game, Seat, Viewer};

#[test]
fn two_and_four_players_conform() {
    for players in [2u8, 4] {
        check_options::<BlackPot>(players);
        check::<BlackPot>(&Config::default(), players, 20, 50 + u64::from(players));
    }
}

#[test]
fn the_view_shows_the_game_but_not_the_seed_nor_the_moves() {
    let s = BlackPot::new(&Config::default(), 4, 9);
    let view = serde_json::to_value(BlackPot::view(&s, Viewer::Seat(Seat(0)))).unwrap();
    let text = view.to_string();
    for hidden in ["seed", "rolls", "legal", "Play"] {
        assert!(!text.contains(hidden), "{hidden} in {text}");
    }
    assert_eq!(view["players"], 4);
    assert_eq!(view["pawns"].as_array().unwrap().len(), 109);
    assert_eq!(view["dice"], serde_json::json!(s.dice));
    assert_eq!(view["roller"], s.roller.0);
    assert_eq!(view["turn"], s.queue[0].0);
    assert_eq!(view["teams"][0]["seats"], serde_json::json!([0, 2]));
    assert_eq!(view["teams"][1]["pawnsLeft"], 40);
    assert_eq!(view["teams"][1]["score"], 400);
    assert_eq!(view["quietLeft"], 10);
    assert_eq!(view["turns"], s.turns);
    assert_eq!(view["end"], serde_json::Value::Null);
    assert_eq!(view["winners"], serde_json::json!([]));
}

#[test]
fn the_view_names_the_winners() {
    let mut s = black_pot::testing::position(2, &[], [4, 2, 6], &[0]);
    s.end = Some(black_pot::End::QuietLimit);
    let view = serde_json::to_value(BlackPot::view(&s, Viewer::Spectator)).unwrap();
    assert_eq!(view["winners"], serde_json::json!([0, 1]));
    s.pawns_left[1] = 30;
    let view = serde_json::to_value(BlackPot::view(&s, Viewer::Spectator)).unwrap();
    assert_eq!(view["winners"], serde_json::json!([0]));
}
