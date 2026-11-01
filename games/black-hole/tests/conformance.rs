use black_hole::{BlackHole, Config, Order};
use ludo_core::conformance::check;

#[test]
fn every_order_conforms() {
    for (k, order) in [Order::Ascending, Order::Free, Order::Random]
        .into_iter()
        .enumerate()
    {
        check::<BlackHole>(&Config { order }, 2, 60, 30 + k as u64);
    }
}
