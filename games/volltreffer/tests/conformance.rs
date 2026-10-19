use ludo_core::conformance::check;
use volltreffer::{Config, Volltreffer};

#[test]
fn volltreffer_conforms_at_every_table_size() {
    for players in 3..=5u8 {
        check::<Volltreffer>(&Config::default(), players, 12, 700 + u64::from(players));
    }
}

#[test]
fn volltreffer_conforms_with_a_narrow_sanctuary() {
    let config = Config {
        narrow_sanctuary: true,
    };
    check::<Volltreffer>(&config, 4, 6, 777);
}
