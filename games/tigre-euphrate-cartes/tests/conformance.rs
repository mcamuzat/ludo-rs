use ludo_core::conformance::check;
use tigre_euphrate_cartes::{Config, TigreEuphrateCartes};

#[test]
fn every_table_size_conforms() {
    for players in 2..=4u8 {
        check::<TigreEuphrateCartes>(&Config::default(), players, 12, 70 + u64::from(players));
    }
}
