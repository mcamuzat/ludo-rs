use ludo_core::{Determinize, Game, Seat};
use manoever::{Action, Config, Manoever, Soldier, State};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn new(players: u8, seed: u64) -> State {
    Manoever::new(&Config::default(), players, seed)
}

/// Squares holding a soldier of `seat` numbered `number`.
fn square_of(s: &State, seat: u8, number: u8) -> Option<u8> {
    (1..=84u8).find(|&sq| {
        s.stacks[usize::from(sq)].contains(&Soldier {
            seat: Seat(seat),
            number,
        })
    })
}

#[test]
fn soldiers_wait_in_the_barracks() {
    for (players, each) in [(2u8, 6u8), (3, 4), (4, 3)] {
        let s = new(players, 1);
        assert_eq!(s.waiting, vec![each; usize::from(players)]);
        assert_eq!(s.turn, Some(Seat(0)));
        assert_eq!(Manoever::legal_actions(&s, Seat(0)), vec![Action::Roll]);
        assert!(Manoever::legal_actions(&s, Seat(1)).is_empty());
    }
}

#[test]
fn roll_places_on_the_die() {
    let mut s = new(2, 7);
    Manoever::apply(&mut s, Seat(0), &Action::Roll).unwrap();
    let (seat, die) = s.last_roll.unwrap();
    assert_eq!(seat, Seat(0));
    assert!((1..=6).contains(&die));
    assert_eq!(square_of(&s, 0, 1), Some(die));
    assert_eq!(s.waiting, vec![5, 6]);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn full_squares_are_rerolled() {
    for seed in 0..50 {
        let mut s = new(2, seed);
        // Squares 1 to 3 full: 9 soldiers out, 3 left (seat 0 has 1, seat 1 two).
        let out = [
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 1),
            (1, 2),
            (1, 3),
            (1, 4),
        ];
        for (i, &(seat, number)) in out.iter().enumerate() {
            s.stacks[i / 3 + 1].push(Soldier {
                seat: Seat(seat),
                number,
            });
        }
        s.waiting = vec![1, 2];
        Manoever::apply(&mut s, Seat(0), &Action::Roll).unwrap();
        let sq = square_of(&s, 0, 6).expect("soldier 6 placed");
        assert!((4..=6).contains(&sq), "seed {seed} placed on {sq}");
        assert_eq!(s.last_roll, Some((Seat(0), sq)));
    }
}

#[test]
fn setup_ends_with_seat_zero() {
    let mut s = new(3, 3);
    for k in 0..12u8 {
        let seat = Seat(k % 3);
        assert_eq!(s.turn, Some(seat));
        Manoever::apply(&mut s, seat, &Action::Roll).unwrap();
    }
    assert_eq!(s.waiting, vec![0, 0, 0]);
    assert_eq!(s.turn, Some(Seat(0)));
    let placed: usize = (1..=6).map(|sq| s.stacks[sq].len()).sum();
    assert_eq!(placed, 12);
}

#[test]
fn same_seed_same_rolls() {
    let play = |seed| {
        let mut s = new(4, seed);
        for k in 0..12u8 {
            Manoever::apply(&mut s, Seat(k % 4), &Action::Roll).unwrap();
        }
        s.stacks
    };
    assert_eq!(play(42), play(42));
}

#[test]
fn determinize_redraws_only_the_seed() {
    let mut s = new(2, 5);
    Manoever::apply(&mut s, Seat(0), &Action::Roll).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(9);
    let d = Manoever::determinize(&s, Seat(1), &mut rng);
    assert_eq!(d.stacks, s.stacks);
    assert_eq!(d.waiting, s.waiting);
    assert_eq!(d.turn, s.turn);
    assert_ne!(d.seed, s.seed);
}
