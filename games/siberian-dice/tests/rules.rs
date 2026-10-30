use ludo_core::{Game, Outcome, Seat, Viewer};
use siberian_dice::board::{at, cells};
use siberian_dice::testing::{position, rolled};
use siberian_dice::{
    is_perfect, perfect_count, Action, Config, End, SiberianDice, Source, State, Step, QUIET_ROUNDS,
};

/// [R] The example figure: row 33-44-55-66 (66 is « X »), 43 and 54 above,
/// 45 below. (ab, owner, value), blue = 0, red = 1.
const FIGURE: [(u8, u8, u8); 6] = [
    (43, 1, 2),
    (54, 0, 3),
    (33, 1, 1),
    (44, 0, 5),
    (55, 1, 4),
    (45, 0, 6),
];

/// Blue has 5 perfect dice and an isolated 1 in 47; red has 5 perfect
/// dice; 36, the only empty cell beside 47, touches no other die.
const FIVE_EACH: [(u8, u8, u8); 12] = [
    (11, 0, 1),
    (12, 0, 1),
    (41, 0, 1),
    (24, 0, 1),
    (77, 0, 1),
    (76, 0, 2),
    (47, 0, 1),
    (52, 1, 1),
    (14, 1, 1),
    (33, 1, 1),
    (44, 1, 2),
    (55, 1, 1),
];

fn perfect(s: &State, ab: u8) -> bool {
    is_perfect(&s.board, at(ab))
}

fn apply(s: &mut State, seat: u8, action: Action) {
    SiberianDice::apply(s, Seat(seat), &action).unwrap();
}

#[test]
fn the_example_figure_of_the_rules() {
    let mut s = position(&FIGURE, 1);
    assert!(perfect(&s, 44) && perfect(&s, 54));
    for ab in [33, 43, 55, 45] {
        assert!(!perfect(&s, ab), "{ab}");
    }
    rolled(&mut s, Source::Cell(at(33)), 1);
    apply(&mut s, 1, Action::Place(at(66)));
    assert!(!perfect(&s, 44));
    for ab in [43, 55, 66, 54] {
        assert!(perfect(&s, ab), "{ab}");
    }
}

#[test]
fn a_new_game() {
    let s = SiberianDice::new(&Config::default(), 2, 7);
    assert!(s.board.iter().all(Option::is_none));
    assert_eq!(s.reserve, [12, 12]);
    assert_eq!(s.turn, Seat(0));
    assert_eq!(
        SiberianDice::legal_actions(&s, Seat(0)),
        vec![Action::Pick(Source::Reserve)]
    );
    assert!(SiberianDice::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn picking_from_the_reserve_rolls_a_die() {
    let mut s = SiberianDice::new(&Config::default(), 2, 7);
    apply(&mut s, 0, Action::Pick(Source::Reserve));
    assert_eq!(s.reserve, [11, 12]);
    let Step::Place { from, value } = s.step else {
        panic!("{:?}", s.step)
    };
    assert_eq!(from, Source::Reserve);
    assert!((1..=6).contains(&value));
    assert_eq!(SiberianDice::legal_actions(&s, Seat(0)).len(), 37);
    apply(&mut s, 0, Action::Place(at(44)));
    assert_eq!(s.turn, Seat(1));
    assert_eq!(s.step, Step::Choose);
    assert_eq!(s.board[usize::from(at(44))].unwrap().value, value);
}

#[test]
fn a_die_can_go_back_to_the_cell_it_left() {
    // Blue 1 in 44 beside red 1 in 55: both perfect.
    let mut s = position(&[(44, 0, 1), (55, 1, 1)], 0);
    assert!(perfect(&s, 55));
    apply(&mut s, 0, Action::Pick(Source::Cell(at(44))));
    assert!(s.board[usize::from(at(44))].is_none());
    assert!(!perfect(&s, 55), "the lifted die no longer counts");
    assert!(SiberianDice::legal_actions(&s, Seat(0)).contains(&Action::Place(at(44))));
    apply(&mut s, 0, Action::Place(at(44)));
    assert!(perfect(&s, 55));
    assert_eq!(s.board[usize::from(at(44))].unwrap().owner, 0);
}

#[test]
fn an_empty_reserve_cannot_be_picked_from() {
    let mut s = position(&[(11, 0, 1)], 0);
    s.reserve[0] = 0;
    assert_eq!(
        SiberianDice::legal_actions(&s, Seat(0)),
        vec![Action::Pick(Source::Cell(at(11)))]
    );
    let before = s.clone();
    assert!(SiberianDice::apply(&mut s, Seat(0), &Action::Pick(Source::Reserve)).is_err());
    assert_eq!(s, before);
}

#[test]
fn timeout_picks_a_board_die_when_the_reserve_is_empty() {
    let mut s = position(&[(11, 0, 1), (77, 1, 1)], 0);
    assert_eq!(
        SiberianDice::timeout_action(&s, Seat(0)),
        Some(Action::Pick(Source::Reserve))
    );
    s.reserve[0] = 0;
    assert_eq!(
        SiberianDice::timeout_action(&s, Seat(0)),
        Some(Action::Pick(Source::Cell(at(11))))
    );
    rolled(&mut s, Source::Cell(at(11)), 4);
    assert!(matches!(
        SiberianDice::timeout_action(&s, Seat(0)),
        Some(Action::Place(_))
    ));
}

#[test]
fn an_opponents_move_can_win_the_game() {
    let mut s = position(&FIVE_EACH, 1);
    assert_eq!(perfect_count(&s.board, 0), 5);
    assert_eq!(perfect_count(&s.board, 1), 5);
    rolled(&mut s, Source::Reserve, 3);
    apply(&mut s, 1, Action::Place(at(36)));
    assert_eq!(perfect_count(&s.board, 0), 6);
    assert_eq!(s.end, Some(End::Won { seat: 0 }));
    assert_eq!(SiberianDice::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
}

#[test]
fn six_each_at_once_is_a_draw() {
    let mut s = position(&FIVE_EACH, 1);
    rolled(&mut s, Source::Reserve, 1);
    apply(&mut s, 1, Action::Place(at(36)));
    assert_eq!(s.end, Some(End::BothSix));
    assert_eq!(SiberianDice::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn the_quiet_limit_draws() {
    let start = {
        let mut s = position(&FIGURE, 0);
        s.quiet = 2 * QUIET_ROUNDS - 1;
        s
    };
    // A 6 in the corner 14 touches nothing: no record.
    let mut s = start.clone();
    rolled(&mut s, Source::Reserve, 6);
    apply(&mut s, 0, Action::Place(at(14)));
    assert_eq!(s.end, Some(End::QuietLimit));
    assert_eq!(SiberianDice::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
    // A 1 in 66 beside 55 alone: a third perfect blue die, a record.
    let mut s = start;
    rolled(&mut s, Source::Reserve, 1);
    apply(&mut s, 0, Action::Place(at(66)));
    assert_eq!(perfect_count(&s.board, 0), 3);
    assert_eq!(s.end, None);
    assert_eq!(s.quiet, 0);
}

#[test]
fn illegal_moves_change_nothing() {
    let s = position(&FIGURE, 0);
    let refused = |s: &State, seat: u8, action: Action| {
        let mut t = s.clone();
        assert!(
            SiberianDice::apply(&mut t, Seat(seat), &action).is_err(),
            "{action:?}"
        );
        assert_eq!(&t, s);
    };
    refused(&s, 0, Action::Place(at(11)));
    refused(&s, 0, Action::Pick(Source::Cell(at(33))));
    refused(&s, 0, Action::Pick(Source::Cell(at(11))));
    refused(&s, 1, Action::Pick(Source::Reserve));
    let mut placing = s.clone();
    rolled(&mut placing, Source::Reserve, 2);
    refused(&placing, 0, Action::Place(at(44)));
    refused(&placing, 0, Action::Pick(Source::Reserve));
    let mut over = position(&FIVE_EACH, 1);
    rolled(&mut over, Source::Reserve, 3);
    apply(&mut over, 1, Action::Place(at(36)));
    refused(&over, 0, Action::Pick(Source::Reserve));
    refused(&over, 1, Action::Pick(Source::Reserve));
}

#[test]
fn after_the_end_nothing_is_pending() {
    let mut s = position(&FIVE_EACH, 1);
    rolled(&mut s, Source::Reserve, 3);
    apply(&mut s, 1, Action::Place(at(36)));
    assert!(SiberianDice::pending(&s).is_empty());
    let view = SiberianDice::view(&s, Viewer::Spectator);
    assert_eq!(view.winners, vec![0]);
    let mut s = position(&FIVE_EACH, 1);
    rolled(&mut s, Source::Reserve, 1);
    apply(&mut s, 1, Action::Place(at(36)));
    assert_eq!(
        SiberianDice::view(&s, Viewer::Spectator).winners,
        vec![0, 1]
    );
}

#[test]
fn the_rolls_of_a_seed_are_pinned() {
    let mut s = SiberianDice::new(&Config::default(), 2, 1);
    let mut rolls = Vec::new();
    for k in 0..8u8 {
        let seat = k % 2;
        apply(&mut s, seat, Action::Pick(Source::Reserve));
        let Step::Place { value, .. } = s.step else {
            panic!()
        };
        rolls.push(value);
        // Rolls do not depend on where the dice go.
        let free = (0..cells().len() as u8)
            .rev()
            .find(|&c| s.board[usize::from(c)].is_none())
            .unwrap();
        apply(&mut s, seat, Action::Place(free));
    }
    assert_eq!(rolls, PINNED);
}

const PINNED: [u8; 8] = [6, 3, 1, 2, 1, 6, 2, 1];
