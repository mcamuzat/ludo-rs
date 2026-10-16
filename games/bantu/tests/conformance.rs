use bantu::{Bantu, Config};
use ludo_core::conformance::check;

#[test]
fn every_table_size_conforms() {
    for players in 2..=4u8 {
        check::<Bantu>(&Config::default(), players, 20, 30 + u64::from(players));
    }
}

#[test]
fn the_short_game_conforms() {
    for players in 2..=4u8 {
        check::<Bantu>(
            &Config { short: true },
            players,
            20,
            40 + u64::from(players),
        );
    }
}
