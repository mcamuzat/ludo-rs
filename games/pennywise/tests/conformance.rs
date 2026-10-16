use ludo_core::conformance::check;
use pennywise::{Character, Config, Pennywise};

#[test]
fn pennywise_conforms_at_every_table_size() {
    for players in 2..=6u8 {
        check::<Pennywise>(&Config::default(), players, 40, 300 + u64::from(players));
    }
}

#[test]
fn pennywise_conforms_with_every_character() {
    for character in [Character::Darlene, Character::Sugar, Character::Taylor] {
        for players in [2, 4] {
            check::<Pennywise>(&Config { character }, players, 30, 400 + u64::from(players));
        }
    }
}
