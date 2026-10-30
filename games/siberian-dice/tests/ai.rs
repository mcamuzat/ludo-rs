use ludo_core::{Game, Heuristics, Seat};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use siberian_dice::board::at;
use siberian_dice::testing::{position, rolled};
use siberian_dice::{gain, Action, End, SiberianDice, Source};

/// Blue has 5 perfect dice and an isolated 1 in 47; it is blue's turn.
const FIVE_BLUE: [(u8, u8, u8); 8] = [
    (11, 0, 1),
    (12, 0, 1),
    (41, 0, 1),
    (52, 1, 1),
    (77, 0, 1),
    (76, 0, 1),
    (24, 0, 2),
    (47, 0, 1),
];

#[test]
fn gain_counts_the_perfect_dice_made_and_lost() {
    let mut s = position(&FIVE_BLUE, 0);
    rolled(&mut s, Source::Reserve, 1);
    // 36 touches 47 alone: the new die and 47 become perfect.
    assert_eq!(gain(&s, Seat(0), at(36)), 2);
    // 53 touches red 52 alone: the new 1 is perfect, red 52 no longer is.
    assert_eq!(gain(&s, Seat(0), at(53)), 2);
    // 42 touches 41 and 52: both lose it, and the new 1 is not perfect.
    assert_eq!(gain(&s, Seat(0), at(42)), 0);
}

#[test]
fn the_rollout_takes_a_winning_cell() {
    let mut s = position(&FIVE_BLUE, 0);
    rolled(&mut s, Source::Reserve, 1);
    let wins = (0..50u64)
        .filter(|&g| {
            let mut rng = ChaCha8Rng::seed_from_u64(g);
            let action = SiberianDice::rollout_action(&s, Seat(0), &mut rng);
            let mut t = s.clone();
            SiberianDice::apply(&mut t, Seat(0), &action).unwrap();
            t.end == Some(End::Won { seat: 0 })
        })
        .count();
    assert!(wins >= 40, "{wins}");
}

#[test]
fn evaluate_favours_the_side_with_more_perfect_dice() {
    let root = position(&FIVE_BLUE, 0);
    let mut sim = root.clone();
    sim.rolls += 100;
    let v = SiberianDice::evaluate(&root, &sim).expect("a long simulation is cut");
    assert!(v[0] > 0.5 && v[0] < 1.0, "{v:?}");
    assert!((v[0] + v[1] - 1.0).abs() < 1e-9);
}

#[test]
fn every_rollout_action_is_legal() {
    let mut s = SiberianDice::new(&Default::default(), 2, 3);
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    for _ in 0..400 {
        if SiberianDice::outcome(&s).is_some() {
            break;
        }
        let seat = s.turn;
        let a: Action = SiberianDice::rollout_action(&s, seat, &mut rng);
        assert!(SiberianDice::legal_actions(&s, seat).contains(&a));
        SiberianDice::apply(&mut s, seat, &a).unwrap();
    }
}
