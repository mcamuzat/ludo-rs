use kurote::{Config, Kurote};
use ludo_core::conformance::check;

#[test]
fn kurote_conforms() {
    check::<Kurote>(&Config::default(), 2, 60, 41);
}
