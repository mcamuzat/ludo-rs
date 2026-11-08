use enemy_chocolatier::{Config, EnemyChocolatier};
use ludo_core::conformance::check;

#[test]
fn every_player_count_conforms() {
    for players in 2..=6 {
        check::<EnemyChocolatier>(&Config::default(), players, 6, 90 + u64::from(players));
    }
}
