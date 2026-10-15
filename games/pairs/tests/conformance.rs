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
    check::<GhostTown>(&Config { hands: 1 }, 3, 100, 17);
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

#[test]
fn goblin_poker_conforms_at_every_table_size() {
    use pairs::goblin_poker::{Config, GoblinPoker};
    for players in 3..=8u8 {
        check::<GoblinPoker>(&Config::default(), players, 40, 400 + u64::from(players));
    }
    check::<GoblinPoker>(&Config { hands: Some(1) }, 3, 100, 37);
    check::<GoblinPoker>(&Config { hands: Some(255) }, 8, 5, 38);
}

#[test]
fn to_the_moon_conforms_at_every_table_size() {
    use pairs::to_the_moon::{Config, ToTheMoon};
    for players in 2..=6u8 {
        check::<ToTheMoon>(&Config::default(), players, 40, 500 + u64::from(players));
    }
    check::<ToTheMoon>(&Config { rounds: Some(1) }, 3, 100, 47);
    check::<ToTheMoon>(&Config { rounds: Some(255) }, 6, 3, 48);
}

#[test]
fn deadfall_conforms_at_every_table_size() {
    use pairs::deadfall::{Config, Deadfall};
    for players in 2..=7u8 {
        check::<Deadfall>(&Config::default(), players, 40, 600 + u64::from(players));
    }
    check::<Deadfall>(&Config { hands: Some(1) }, 3, 100, 57);
    check::<Deadfall>(&Config { hands: Some(255) }, 7, 5, 58);
}

#[test]
fn captains_orders_conforms_at_every_table_size() {
    use pairs::captains_orders::{CaptainsOrders, Config};
    for players in 4..=8u8 {
        check::<CaptainsOrders>(&Config::default(), players, 40, 700 + u64::from(players));
    }
    check::<CaptainsOrders>(&Config { wins: Some(1) }, 4, 100, 67);
    check::<CaptainsOrders>(&Config { wins: Some(255) }, 8, 5, 68);
}

#[test]
fn pendleton_conforms() {
    use pairs::pendleton::{Config, Pendleton};
    check::<Pendleton>(&Config::default(), 2, 40, 800);
    check::<Pendleton>(&Config { target: Some(1) }, 2, 100, 77);
    check::<Pendleton>(
        &Config {
            target: Some(65535),
        },
        2,
        3,
        78,
    );
}

#[test]
fn kitty_conforms_at_every_table_size() {
    use pairs::kitty::{Config, Kitty};
    for players in 3..=6u8 {
        check::<Kitty>(&Config::default(), players, 40, 900 + u64::from(players));
    }
}

#[test]
fn octopus_conforms_at_every_table_size() {
    use pairs::octopus::{Config, Octopus};
    for players in 4..=6u8 {
        check::<Octopus>(&Config::default(), players, 40, 1000 + u64::from(players));
    }
    check::<Octopus>(&Config { hands: Some(1) }, 4, 100, 87);
    check::<Octopus>(&Config { hands: Some(255) }, 6, 3, 88);
}

#[test]
fn port_conforms_at_every_table_size() {
    use pairs::port::{Config, Port};
    for players in 2..=7u8 {
        check::<Port>(&Config::default(), players, 30, 1100 + u64::from(players));
    }
    check::<Port>(&Config { target: Some(1) }, 3, 100, 97);
    check::<Port>(&Config { target: Some(255) }, 2, 3, 98);
}

#[test]
fn starboard_conforms_at_every_table_size() {
    use pairs::starboard::{Config, Starboard};
    for players in 2..=7u8 {
        check::<Starboard>(&Config::default(), players, 30, 1200 + u64::from(players));
    }
    check::<Starboard>(&Config { rounds: Some(1) }, 3, 100, 107);
    check::<Starboard>(&Config { rounds: Some(255) }, 2, 3, 108);
}

#[test]
fn sweep_conforms_at_every_table_size() {
    use pairs::sweep::{Config, Sweep};
    for players in 3..=6u8 {
        check::<Sweep>(&Config::default(), players, 40, 1300 + u64::from(players));
    }
}

#[test]
fn rocket_conforms_at_every_table_size() {
    use pairs::rocket::{Config, Rocket};
    for players in 1..=6u8 {
        check::<Rocket>(&Config::default(), players, 30, 1400 + u64::from(players));
    }
    check::<Rocket>(&Config { rounds: Some(1) }, 3, 100, 117);
    check::<Rocket>(&Config { rounds: Some(255) }, 6, 3, 118);
}

#[test]
fn monster_conforms_at_every_table_size() {
    use pairs::monster::{Config, Monster};
    for players in 4..=8u8 {
        check::<Monster>(&Config::default(), players, 30, 1500 + u64::from(players));
    }
    check::<Monster>(&Config { games: Some(1) }, 4, 100, 127);
    check::<Monster>(&Config { games: Some(255) }, 8, 3, 128);
}

#[test]
fn troll_guts_conforms_at_every_table_size() {
    use pairs::troll_guts::{Config, TrollGuts};
    for players in 3..=6u8 {
        check::<TrollGuts>(&Config::default(), players, 20, 1600 + u64::from(players));
    }
    check::<TrollGuts>(&Config { hands: Some(1) }, 3, 60, 137);
    check::<TrollGuts>(&Config { hands: Some(255) }, 6, 2, 138);
}

#[test]
fn blackstone_conforms_at_every_table_size() {
    use pairs::blackstone::{Blackstone, Config};
    for players in 2..=8u8 {
        check::<Blackstone>(&Config::default(), players, 30, 1700 + u64::from(players));
    }
    check::<Blackstone>(&Config { hands: Some(1) }, 3, 100, 147);
    check::<Blackstone>(&Config { hands: Some(255) }, 8, 3, 148);
}
