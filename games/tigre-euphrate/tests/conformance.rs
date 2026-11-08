use ludo_core::conformance::check;
use tigre_euphrate::{Config, Reinforcements, TigreEuphrate};

#[test]
fn every_variant_and_player_count_conforms() {
    for (k, reinforcements) in [Reinforcements::English, Reinforcements::Free]
        .into_iter()
        .enumerate()
    {
        for players in 2..=4 {
            check::<TigreEuphrate>(&Config { reinforcements }, players, 6, 70 + k as u64);
        }
    }
}
