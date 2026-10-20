use cryptide::{Config, Cryptide, Mode};
use ludo_core::conformance::check;

fn conforms(mode: Mode, players: u8) {
    check::<Cryptide>(&Config { mode }, players, 30, 100 + u64::from(players));
}

#[test]
fn normal_at_three() {
    conforms(Mode::Normal, 3);
}

#[test]
fn normal_at_four() {
    conforms(Mode::Normal, 4);
}

#[test]
fn normal_at_five() {
    conforms(Mode::Normal, 5);
}

#[test]
fn advanced_at_three() {
    conforms(Mode::Advanced, 3);
}

#[test]
fn advanced_at_four() {
    conforms(Mode::Advanced, 4);
}

#[test]
fn advanced_at_five() {
    conforms(Mode::Advanced, 5);
}
