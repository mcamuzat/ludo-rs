use five_by_fifteen::{Config, FiveByFifteen, Route};
use ludo_core::conformance::check;

#[test]
fn every_route_conforms() {
    for (n, route) in [Route::Short, Route::Medium, Route::Long]
        .into_iter()
        .enumerate()
    {
        check::<FiveByFifteen>(&Config { route }, 1, 50, n as u64 + 3);
    }
}
