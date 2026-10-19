use ludo_core::conformance::check;
use monad::{Config, Monad};

#[test]
fn monad_conforms_at_every_table_size() {
    for players in 2..=4u8 {
        check::<Monad>(&Config::default(), players, 40, 600 + u64::from(players));
    }
}
