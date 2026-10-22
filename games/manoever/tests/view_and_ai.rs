use ludo_core::{Game, Heuristics, Seat, Viewer};
use manoever::testing::position;
use manoever::{Action, Config, Dest, Manoever};
use rand::rngs::SmallRng;
use rand::SeedableRng;

#[test]
fn view_hides_the_seed() {
    let mut s = Manoever::new(&Config::default(), 2, 0xDEAD_BEEF_1234_5678);
    Manoever::apply(&mut s, Seat(0), &Action::Roll).unwrap();
    let json = serde_json::to_string(&Manoever::view(&s, Viewer::Seat(Seat(1)))).unwrap();
    assert!(!json.contains("seed"), "{json}");
    assert!(!json.contains(&s.seed.to_string()));
    let v = Manoever::view(&s, Viewer::Spectator);
    assert!(v.setup);
    assert_eq!(v.waiting, vec![5, 6]);
    assert_eq!(v.last_roll.map(|(seat, _)| seat), Some(Seat(0)));
}

#[test]
fn view_explains_each_soldier_to_move() {
    let s = position(2, &[(0, 1, 47), (1, 1, 50), (1, 2, 60), (1, 3, 70)], 0);
    let v = Manoever::view(&s, Viewer::Seat(Seat(1)));
    assert!(!v.setup);
    assert_eq!(v.stacks.len(), 84);
    assert_eq!(v.stacks[46].len(), 1);
    assert_eq!(v.to_move.len(), 1);
    let m = &v.to_move[0];
    assert_eq!(
        (m.number, m.square, m.position, m.height, m.bonus, m.max),
        (1, 47, 4, 2, 3, 24)
    );
    // 48..=71, all with room (50, 60 and 70 hold one soldier each).
    assert_eq!(m.dests.len(), 24);
    assert_eq!(v.grays.len(), 3);
    assert_eq!(v.grays[2].square, 55);
    assert_eq!(v.grays[2].taken_by, None);
    assert_eq!(v.scores.len(), 2);
    assert!(!v.over);
}

#[test]
fn timeout_rolls_or_goes_farthest() {
    let s = Manoever::new(&Config::default(), 3, 1);
    assert_eq!(Manoever::timeout_action(&s, Seat(0)), Some(Action::Roll));
    let s = position(2, &[(0, 2, 30), (0, 1, 50), (1, 1, 60)], 0);
    assert_eq!(
        Manoever::timeout_action(&s, Seat(0)),
        Some(Action::Move {
            soldier: 1,
            to: Dest::Square(52)
        })
    );
}

#[test]
fn rollout_goes_home_when_it_can() {
    let s = position(2, &[(0, 1, 84), (0, 2, 10), (1, 1, 20)], 0);
    let mut rng = SmallRng::seed_from_u64(1);
    for _ in 0..20 {
        assert_eq!(
            Manoever::rollout_action(&s, Seat(0), &mut rng),
            Action::Move {
                soldier: 1,
                to: Dest::Barracks
            }
        );
    }
}

#[test]
fn rollout_terminates() {
    let mut rng = SmallRng::seed_from_u64(3);
    for players in 2..=4u8 {
        for game in 0..20u64 {
            let mut s = Manoever::new(&Config::default(), players, game);
            let mut plies = 0;
            while let Some(seat) = s.turn {
                let a = Manoever::rollout_action(&s, seat, &mut rng);
                Manoever::apply(&mut s, seat, &a).unwrap();
                plies += 1;
                assert!(plies < 3000, "endless game");
            }
            assert!(Manoever::outcome(&s).is_some());
        }
    }
}

#[test]
fn search_actions_are_legal_and_not_empty() {
    let s = position(2, &[(0, 1, 47), (1, 1, 50), (1, 2, 60), (1, 3, 70)], 0);
    let legal = Manoever::legal_actions(&s, Seat(0));
    let some = Manoever::search_actions(&s, Seat(0));
    assert!(!some.is_empty());
    assert!(some.iter().all(|a| legal.contains(a)));
}

#[test]
fn evaluate_cuts_long_simulations() {
    let root = Manoever::new(&Config::default(), 2, 0);
    assert!(Manoever::evaluate(&root, &root).is_none());
    let mut late = root.clone();
    late.moves = root.moves + 1000;
    let rewards = Manoever::evaluate(&root, &late).expect("cut");
    assert_eq!(rewards.len(), 2);
}
