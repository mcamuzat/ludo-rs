use ludo_core::conformance::check;
use riichi::{Config, Riichi};

/// Random legal play through whole hanchans. Run in release (`cargo test --release`).
#[test]
fn four_players_conform() {
    check::<Riichi>(&Config::default(), 4, 10, 21);
}
