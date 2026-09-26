use ludo_core::conformance::check;
use yavalath::{Config, Yavalath};

#[test]
fn two_players_conform() {
    check::<Yavalath>(&Config::default(), 2, 300, 7);
}

#[test]
fn three_players_conform() {
    check::<Yavalath>(&Config::default(), 3, 200, 8);
}
