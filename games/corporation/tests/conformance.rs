use corporation::{Config, Corporation, TableSetup};
use ludo_core::conformance::check;

#[test]
fn every_table_and_player_count_conforms() {
    for (k, table) in [TableSetup::Draft, TableSetup::Random]
        .into_iter()
        .enumerate()
    {
        for players in 2..=4 {
            check::<Corporation>(&Config { table }, players, 40, 50 + k as u64);
        }
    }
}
