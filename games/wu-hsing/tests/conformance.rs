use ludo_core::conformance::check;
use wu_hsing::{Config, WuHsing};

#[test]
fn every_table_size_conforms() {
    for players in 2..=4u8 {
        check::<WuHsing>(&Config::default(), players, 30, 40 + u64::from(players));
    }
}
