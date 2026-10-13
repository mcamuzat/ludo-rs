use ludo_core::conformance::check;
use pairs::continuous::{Config, ContinuousPairs};

#[test]
fn continuous_pairs_conforms_at_every_table_size() {
    for players in 2..=8u8 {
        check::<ContinuousPairs>(&Config::default(), players, 60, u64::from(players));
    }
}

#[test]
fn continuous_pairs_conforms_with_a_short_target() {
    check::<ContinuousPairs>(&Config { target: Some(6) }, 4, 100, 99);
}
