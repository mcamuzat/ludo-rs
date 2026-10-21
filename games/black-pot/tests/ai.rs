use black_pot::board::CellId;
use black_pot::testing::position;
use black_pot::{Action, BlackPot, Config};
use ludo_core::{Game, Heuristics, Seat};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn the_search_never_considers_not_found() {
    for g in 0..10u64 {
        let s = BlackPot::new(&Config::default(), 2, g);
        let seat = BlackPot::pending(&s).0[0];
        let search = BlackPot::search_actions(&s, seat);
        assert!(!search.is_empty());
        assert!(!search.contains(&Action::NotFound));
        assert!(!BlackPot::random_actions(&s, seat).contains(&Action::NotFound));
        let legal = BlackPot::legal_actions(&s, seat);
        assert!(search.iter().all(|a| legal.contains(a)));
    }
}

#[test]
fn the_search_keeps_the_move_completing_a_400_line() {
    // A window of four Black-Pots, team 0 holding all of it but one of them.
    let window = black_pot::board::windows()
        .iter()
        .find(|w| {
            w.cells
                .iter()
                .filter(|&&c| {
                    black_pot::board::cells()[usize::from(c)].kind
                        == black_pot::board::Kind::BlackPot
                })
                .count()
                == 4
        })
        .unwrap();
    let missing = *window
        .cells
        .iter()
        .find(|&&c| {
            black_pot::board::cells()[usize::from(c)].kind == black_pot::board::Kind::BlackPot
        })
        .unwrap();
    let owners: Vec<(CellId, u8)> = window
        .cells
        .iter()
        .filter(|&&c| c != missing)
        .map(|&c| (c, 0))
        .collect();
    let value = black_pot::board::cells()[usize::from(missing)].value;
    // Some roll that reaches `value` with the three dice.
    let dice = (0..216u32)
        .map(|k| [(k % 6 + 1) as u8, (k / 6 % 6 + 1) as u8, (k / 36 + 1) as u8])
        .find(|d| black_pot::calc::values(d, &[0, 1, 2]).contains_key(&u32::from(value)))
        .unwrap();
    let s = position(2, &owners, dice, &[0]);
    let search = BlackPot::search_actions(&s, Seat(0));
    assert!(
        search.iter().any(|a| matches!(a, Action::Play { groups } if groups.iter().any(|g| g.cell == Some(missing)))),
        "{search:?}"
    );
}

#[test]
fn rollouts_play_legal_moves_and_finish() {
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    for g in 0..30u64 {
        let players = if g % 2 == 0 { 2 } else { 4 };
        let mut s = BlackPot::new(&Config::default(), players, g);
        let mut steps = 0;
        while let Some(&seat) = BlackPot::pending(&s).0.first() {
            let action = BlackPot::rollout_action(&s, seat, &mut rng);
            assert_ne!(action, Action::NotFound);
            BlackPot::apply(&mut s, seat, &action).unwrap();
            steps += 1;
            assert!(steps < 10_000);
        }
    }
}
