use ludo_core::conformance::check;
use pairs::continuous::{Config, ContinuousPairs};

#[test]
fn continuous_pairs_conforms_at_every_table_size() {
    for players in 2..=8u8 {
        check::<ContinuousPairs>(&Config::default(), players, 60, u64::from(players));
    }
}

#[test]
fn continuous_pairs_conforms_with_a_short_target() {
    check::<ContinuousPairs>(&Config { target: Some(6) }, 4, 100, 99);
}

#[test]
fn continuous_pairs_conforms_with_an_absurd_target() {
    check::<ContinuousPairs>(&Config { target: Some(255) }, 2, 30, 5);
    check::<ContinuousPairs>(&Config { target: Some(255) }, 8, 30, 6);
}
#[test]
fn auberge_conforms_at_every_table_size() {
    use pairs::auberge::{Auberge, Config};
    for players in 2..=8u8 {
        check::<Auberge>(&Config::default(), players, 40, 100 + u64::from(players));
    }
    check::<Auberge>(&Config { rounds: Some(1) }, 3, 100, 7);
    check::<Auberge>(&Config { rounds: Some(255) }, 2, 10, 8);
}

#[test]
fn ghost_town_conforms_at_every_table_size() {
    use pairs::ghost_town::{Config, GhostTown};
    for players in 2..=6u8 {
        check::<GhostTown>(&Config::default(), players, 40, 200 + u64::from(players));
    }
    check::<GhostTown>(&Config { hands: Some(1) }, 3, 100, 17);
    check::<GhostTown>(&Config { hands: Some(255) }, 2, 5, 18);
}

#[test]
fn zombie_life_conforms_at_every_table_size() {
    use pairs::zombie_life::{Config, ZombieLife};
    for players in 3..=6u8 {
        check::<ZombieLife>(&Config::default(), players, 40, 300 + u64::from(players));
    }
    check::<ZombieLife>(&Config { target: Some(1) }, 3, 100, 27);
    check::<ZombieLife>(&Config { target: Some(255) }, 6, 3, 28);
}
