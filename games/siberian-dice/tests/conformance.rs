use ludo_core::conformance::{check, check_options};
use ludo_core::{Game, Seat, Viewer};
use siberian_dice::{Config, SiberianDice};

#[test]
fn two_players_conform() {
    check_options::<SiberianDice>(2);
    check::<SiberianDice>(&Config::default(), 2, 20, 52);
}

#[test]
fn the_view_shows_the_game_but_not_the_seed() {
    let s = SiberianDice::new(&Config::default(), 2, 9);
    let view = serde_json::to_value(SiberianDice::view(&s, Viewer::Seat(Seat(0)))).unwrap();
    let text = view.to_string();
    for hidden in ["seed", "rolls", "records"] {
        assert!(!text.contains(hidden), "{hidden} in {text}");
    }
    assert_eq!(view["cells"].as_array().unwrap().len(), 37);
    assert_eq!(view["reserve"], serde_json::json!([12, 12]));
    assert_eq!(view["perfect"], serde_json::json!([0, 0]));
    assert_eq!(view["step"], "Choose");
    assert_eq!(view["quietLeft"], 30);
    assert_eq!(view["winners"], serde_json::json!([]));
}
