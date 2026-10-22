use ludo_core::conformance::check;
use manoever::{Config, Manoever};

#[test]
fn every_table_size_conforms() {
    for players in 2..=4u8 {
        check::<Manoever>(&Config::default(), players, 20, 50 + u64::from(players));
    }
}
