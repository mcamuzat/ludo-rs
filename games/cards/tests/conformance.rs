use cards::president::{Config, Jokers, President};
use ludo_core::conformance::check;

#[test]
fn president_conforms_at_every_table_size() {
    for players in 4..=7u8 {
        check::<President>(&Config::default(), players, 20, 100 + u64::from(players));
    }
}

#[test]
fn president_conforms_with_every_variant() {
    let all = Config {
        rounds: 3,
        revolution: true,
        no_finish_on_two: true,
        jokers: Jokers::Wild,
    };
    check::<President>(&all, 4, 20, 7);
    check::<President>(&all, 5, 15, 8);
    let strong = Config {
        jokers: Jokers::High,
        revolution: true,
        ..Config::default()
    };
    check::<President>(&strong, 6, 15, 9);
}
