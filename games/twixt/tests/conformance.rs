use ludo_core::conformance::{check, check_options};
use twixt::{Config, Twixt};

#[test]
fn twixt_conforms() {
    check_options::<Twixt>(2);
    check::<Twixt>(&Config::default(), 2, 20, 7);
}
