use ludo_core::{Game, Outcome, Seat};
use manoever::testing::position;
use manoever::{
    bonus, destinations, height, max_move, scores, to_move, Action, Config, Dest, Manoever,
    Soldier, State,
};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn soldier(seat: u8, number: u8) -> Soldier {
    Soldier {
        seat: Seat(seat),
        number,
    }
}

fn mv(s: &mut State, seat: u8, number: u8, to: Dest) {
    Manoever::apply(
        s,
        Seat(seat),
        &Action::Move {
            soldier: number,
            to,
        },
    )
    .unwrap();
}

#[test]
fn position_counts_occupied_squares() {
    let s = position(2, &[(0, 1, 10), (0, 2, 10), (1, 1, 15), (1, 2, 30)], 0);
    assert_eq!(manoever::position(&s, 10), 3);
    assert_eq!(manoever::position(&s, 15), 2);
    assert_eq!(manoever::position(&s, 30), 1);
    // Black disc on 12 and 20 with nobody on them, soldiers in the barracks:
    // none of them counts.
    assert_eq!(max_move(&s, soldier(1, 1)), 2);
}

#[test]
fn rule_example_is_24() {
    let mut s = position(2, &[(0, 1, 47), (1, 1, 50), (1, 2, 60), (1, 3, 70)], 0);
    assert_eq!(height(&s, 47, 0), 2);
    assert_eq!(bonus(47, 0), 3);
    assert_eq!(max_move(&s, soldier(0, 1)), 24);
    s.stacks[47].push(soldier(0, 2));
    assert_eq!(height(&s, 47, 1), 3);
    assert_eq!(bonus(47, 1), 1);
    assert_eq!(max_move(&s, soldier(0, 2)), 12);
    assert!(destinations(&s, soldier(0, 1)).is_empty(), "covered");
}

#[test]
fn white_doubles_only_on_the_disc() {
    let s = position(2, &[(0, 1, 24), (1, 1, 10)], 0);
    assert_eq!(max_move(&s, soldier(0, 1)), 4);
}

#[test]
fn no_fourth_level() {
    let s = position(
        2,
        &[
            (1, 1, 11),
            (1, 2, 11),
            (1, 3, 11),
            (1, 4, 12),
            (1, 5, 12),
            (0, 1, 10),
        ],
        0,
    );
    assert_eq!(max_move(&s, soldier(0, 1)), 3);
    assert_eq!(destinations(&s, soldier(0, 1)), vec![Dest::Square(13)]);
}

#[test]
fn march_actions_are_moves() {
    let s = position(2, &[(0, 1, 30), (1, 1, 50)], 0);
    let legal = Manoever::legal_actions(&s, Seat(0));
    assert_eq!(legal.len(), 2);
    assert!(legal
        .iter()
        .all(|a| matches!(a, Action::Move { soldier: 1, .. })));
}

#[test]
fn minimum_one_maximum_live() {
    let mut s = position(2, &[(0, 1, 50), (0, 2, 36), (1, 1, 60), (1, 2, 70)], 0);
    assert_eq!(max_move(&s, soldier(0, 1)), 3);
    assert_eq!(max_move(&s, soldier(0, 2)), 16);
    mv(&mut s, 0, 2, Dest::Square(52));
    assert_eq!(max_move(&s, soldier(0, 1)), 4);
    assert_eq!(s.turn, Some(Seat(0)));
    assert_eq!(destinations(&s, soldier(0, 1))[0], Dest::Square(51));
}

#[test]
fn freed_soldier_moves_this_turn() {
    let mut s = position(2, &[(0, 2, 30), (0, 1, 30), (1, 1, 60)], 0);
    assert_eq!(to_move(&s, Seat(0)), vec![1]);
    mv(&mut s, 0, 1, Dest::Square(34));
    assert_eq!(to_move(&s, Seat(0)), vec![2]);
    assert_eq!(s.turn, Some(Seat(0)));
}

#[test]
fn covered_by_a_comrade_waits() {
    let mut s = position(2, &[(0, 1, 30), (0, 2, 32), (1, 1, 60)], 0);
    assert_eq!(to_move(&s, Seat(0)), vec![1, 2]);
    mv(&mut s, 0, 1, Dest::Square(32));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.last_turns[0].len(), 1);
}

#[test]
fn moved_soldier_does_not_move_again() {
    let mut s = position(2, &[(0, 1, 30), (0, 2, 50), (1, 1, 60)], 0);
    mv(&mut s, 0, 1, Dest::Square(31));
    assert_eq!(to_move(&s, Seat(0)), vec![2]);
    let legal = Manoever::legal_actions(&s, Seat(0));
    assert!(legal
        .iter()
        .all(|a| matches!(a, Action::Move { soldier: 2, .. })));
}

#[test]
fn fully_covered_seat_is_skipped() {
    let mut s = position(
        3,
        &[(1, 1, 41), (2, 1, 41), (1, 2, 43), (2, 2, 43), (0, 1, 30)],
        0,
    );
    // Yellow's soldiers 3 and 4 are in the barracks, 1 and 2 under blue.
    mv(&mut s, 0, 1, Dest::Square(31));
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn march_opens_on_a_seat_that_can_move() {
    let mut s = Manoever::new(&Config::default(), 4, 11);
    // Red's three soldiers under yellow's; only green's last one waits.
    let out = [
        (1, 0, 1),
        (1, 1, 1),
        (2, 0, 2),
        (2, 1, 2),
        (3, 0, 3),
        (3, 1, 3),
        (4, 2, 1),
        (4, 3, 1),
        (5, 2, 2),
        (5, 3, 2),
        (6, 2, 3),
    ];
    for &(sq, seat, number) in &out {
        s.stacks[sq].push(soldier(seat, number));
    }
    s.waiting = vec![0, 0, 0, 1];
    s.turn = Some(Seat(3));
    Manoever::apply(&mut s, Seat(3), &Action::Roll).unwrap();
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn gray_is_won_on_landing() {
    let mut s = position(2, &[(0, 1, 23), (1, 1, 60), (1, 2, 70)], 0);
    mv(&mut s, 0, 1, Dest::Square(25));
    assert_eq!(s.grays[0], Some(Seat(0)));
    assert_eq!(s.last_turns[0][0].gray, Some(1));
    assert_eq!(height(&s, 25, 0), 1);

    let mut s = position(2, &[(0, 1, 23), (1, 1, 60), (1, 2, 70)], 0);
    mv(&mut s, 0, 1, Dest::Square(26));
    assert_eq!(s.grays[0], None);
}

#[test]
fn barracks_without_exact_count() {
    let s = position(2, &[(0, 1, 84), (1, 1, 50), (1, 2, 41)], 0);
    assert_eq!(destinations(&s, soldier(0, 1)), vec![Dest::Barracks]);

    let mut s = position(
        2,
        &[
            (1, 4, 80),
            (1, 5, 80),
            (0, 1, 80),
            (1, 1, 81),
            (1, 2, 82),
            (1, 3, 83),
        ],
        0,
    );
    assert_eq!(max_move(&s, soldier(0, 1)), 12);
    assert_eq!(
        destinations(&s, soldier(0, 1)),
        vec![
            Dest::Square(81),
            Dest::Square(82),
            Dest::Square(83),
            Dest::Square(84),
            Dest::Barracks
        ]
    );
    // Red 2-6 and yellow 6 hold places XII to VII.
    mv(&mut s, 0, 1, Dest::Barracks);
    assert_eq!(s.barracks[5], Some(soldier(0, 1)));
    assert_eq!(s.last_turns[0][0].place, Some(6));
}

#[test]
fn last_soldier_goes_home_automatically() {
    let mut s = position(2, &[(0, 1, 84), (1, 1, 24)], 0);
    mv(&mut s, 0, 1, Dest::Barracks);
    assert_eq!(s.barracks[1], Some(soldier(0, 1)));
    assert_eq!(s.barracks[0], Some(soldier(1, 1)));
    assert!(s.over);
    assert!(Manoever::pending(&s).is_empty());
    assert_eq!(s.grays, [None; 3]);
    // Red 2-6 on XII to VIII, yellow 2-6 on VII to III.
    assert_eq!(
        scores(&s),
        vec![12 + 11 + 10 + 9 + 8 + 2, 7 + 6 + 5 + 4 + 3 + 1]
    );
    assert_eq!(Manoever::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
}

#[test]
fn shared_win() {
    let mut s = position(2, &[], 0);
    let red = [12, 1, 10, 3, 8, 5];
    let yellow = [11, 2, 9, 4, 7, 6];
    for (i, (&r, &y)) in red.iter().zip(&yellow).enumerate() {
        s.barracks[r - 1] = Some(soldier(0, i as u8 + 1));
        s.barracks[y - 1] = Some(soldier(1, i as u8 + 1));
    }
    s.grays = [Some(Seat(0)), Some(Seat(1)), None];
    s.over = true;
    s.turn = None;
    assert_eq!(scores(&s), vec![40, 41]);
    s.grays = [None, None, None];
    assert_eq!(Manoever::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn illegal_input_changes_nothing() {
    let mut s = position(
        2,
        &[(0, 1, 30), (0, 2, 50), (1, 1, 50), (0, 3, 21), (1, 2, 60)],
        0,
    );
    let tries = [
        (
            0,
            Action::Move {
                soldier: 1,
                to: Dest::Square(0),
            },
        ),
        (
            0,
            Action::Move {
                soldier: 1,
                to: Dest::Square(200),
            },
        ),
        (
            0,
            Action::Move {
                soldier: 1,
                to: Dest::Square(30),
            },
        ),
        (
            0,
            Action::Move {
                soldier: 9,
                to: Dest::Square(31),
            },
        ),
        (
            0,
            Action::Move {
                soldier: 2,
                to: Dest::Square(51),
            },
        ),
        (0, Action::Roll),
        (
            1,
            Action::Move {
                soldier: 1,
                to: Dest::Square(51),
            },
        ),
    ];
    for (seat, action) in &tries {
        let before = s.clone();
        assert!(
            Manoever::apply(&mut s, Seat(*seat), action).is_err(),
            "{action:?}"
        );
        assert_eq!(s, before);
    }
    mv(&mut s, 0, 1, Dest::Square(31));
    let before = s.clone();
    let again = Action::Move {
        soldier: 1,
        to: Dest::Square(32),
    };
    assert!(Manoever::apply(&mut s, Seat(0), &again).is_err());
    assert_eq!(s, before);

    let mut done = position(2, &[(0, 1, 84), (1, 1, 24)], 0);
    mv(&mut done, 0, 1, Dest::Barracks);
    let before = done.clone();
    assert!(Manoever::apply(&mut done, Seat(1), &Action::Roll).is_err());
    assert_eq!(done, before);
}

#[test]
fn game_always_ends() {
    for players in 2..=4u8 {
        for seed in 0..200u64 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut s = Manoever::new(&Config::default(), players, seed);
            let mut actions = 0;
            while let Some(&seat) = Manoever::pending(&s).0.first() {
                let legal = Manoever::legal_actions(&s, seat);
                let action = *legal.choose(&mut rng).expect("a pending seat can act");
                Manoever::apply(&mut s, seat, &action).unwrap();
                actions += 1;
                assert!(actions < 3000, "{players} players, seed {seed} never ends");
            }
            assert!(s.over);
            assert!(s.barracks.iter().all(Option::is_some));
        }
    }
}
