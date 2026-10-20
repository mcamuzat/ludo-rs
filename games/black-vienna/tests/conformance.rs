use black_vienna::{BlackVienna, Config};
use ludo_core::conformance::check;

#[test]
fn black_vienna_conforms_at_every_table_size() {
    for players in 3..=6u8 {
        check::<BlackVienna>(&Config::default(), players, 30, 100 + u64::from(players));
    }
}
