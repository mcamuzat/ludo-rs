use corporation::{
    room_color, Action, Color, Config, Corporation, End, Phase, State, Step, TableSetup, BOARDROOM,
    QUIET_ROUNDS,
};
use ludo_core::{Game, Outcome, Seat};

use Color::{Blue as B, Green as G, Red as R, Yellow as Y};

/// [C]'s example table, promotion 1 (bottom) to 6 (top), each pair as
/// (employee, beneath).
const EXAMPLE_TABLE: [Color; 12] = [G, R, R, Y, B, G, G, B, R, B, Y, Y];
/// [C]'s positions A to E: the 3rd to 7th floors.
const A: u8 = 6;
const BB: u8 = 7;
const C: u8 = 8;
const D: u8 = 9;
const E: u8 = 10;

fn play(s: &mut State, seat: u8, action: Action) {
    Corporation::apply(s, Seat(seat), &action).unwrap();
}

/// A game past the setting up, with `table` laid.
fn with_table(players: u8, table: [Color; 12]) -> State {
    let mut s = Corporation::new(
        &Config {
            table: TableSetup::Random,
        },
        players,
        0,
    );
    s.table = table.into_iter().map(Some).collect();
    s
}

/// [C]'s example position: red on blue at A, yellow at B, yellow at D.
fn example() -> State {
    let mut s = with_table(2, EXAMPLE_TABLE);
    s.stacks[usize::from(A)] = vec![B, R];
    s.stacks[usize::from(BB)] = vec![Y];
    s.stacks[usize::from(D)] = vec![Y];
    s
}

#[test]
fn the_offices_read_off_the_rulebook() {
    let desks: Vec<_> = (0..4).map(room_color).collect();
    assert_eq!(desks, [Some(R), Some(B), Some(G), Some(Y)]);
    let floors: Vec<_> = (4..12).map(room_color).collect();
    assert_eq!(
        floors,
        [R, B, G, Y, R, B, G, Y].map(Some).to_vec(),
        "red, blue, green, yellow twice from the ground floor"
    );
    assert_eq!(room_color(BOARDROOM), None);
    // [C]'s A to E: green, yellow, red, blue, green.
    let ae: Vec<_> = [A, BB, C, D, E].into_iter().map(room_color).collect();
    assert_eq!(ae, [G, Y, R, B, G].map(Some).to_vec());
}

/// [C]'s alternative strategy: yellow on yellow first, then red climbs to
/// E, and blue on green ends the turn.
#[test]
fn the_rulebook_example_alternative() {
    let mut s = example();
    play(&mut s, 0, Action::Skip);
    play(&mut s, 0, Action::Skip);
    assert_eq!(s.phase, Phase::Promote);
    assert_eq!(s.eligible(), vec![A, BB]);
    play(&mut s, 0, Action::Promote { from: BB });
    // Red on blue is then the only promotion: red A→B. From there red and
    // the blue uncovered at A may both move; red goes on to E.
    assert_eq!(s.stacks[usize::from(BB)], vec![R]);
    for from in [BB, C, D] {
        assert_eq!(s.eligible(), vec![A, from]);
        play(&mut s, 0, Action::Promote { from });
    }
    // No red on green: blue on green is forced, A→B.
    assert_eq!(s.stacks[usize::from(A)], vec![]);
    assert_eq!(s.stacks[usize::from(BB)], vec![B]);
    assert_eq!(s.stacks[usize::from(C)], vec![Y]);
    assert_eq!(s.stacks[usize::from(D)], vec![Y]);
    assert_eq!(s.stacks[usize::from(E)], vec![R]);
    assert_eq!(s.turn, Seat(1));
    assert_eq!(s.phase, Phase::Swap);
    let steps = &s.last_turn.as_ref().unwrap().steps;
    let red: Vec<u8> = steps
        .iter()
        .filter_map(|st| match *st {
            Step::Promote { color: R, to, .. } => Some(to),
            _ => None,
        })
        .collect();
    assert_eq!(red, vec![BB, C, D, E]);
}

/// [C]'s first strategy: red on blue first lands red on the yellow at B,
/// from where it may go on, and the blue left at A must move too.
#[test]
fn the_rulebook_example_first_strategy() {
    let mut s = example();
    play(&mut s, 0, Action::Skip);
    play(&mut s, 0, Action::Skip);
    play(&mut s, 0, Action::Promote { from: A });
    assert_eq!(s.stacks[usize::from(BB)], vec![Y, R]);
    assert_eq!(s.eligible(), vec![A, BB]);
    play(&mut s, 0, Action::Promote { from: BB });
    // No red on red: red stops at C, uncovering the yellow at B.
    assert_eq!(s.stacks[usize::from(C)], vec![R]);
    assert_eq!(s.eligible(), vec![A, BB]);
    assert!(Corporation::apply(&mut s, Seat(0), &Action::Promote { from: C }).is_err());
}

#[test]
fn the_table_is_drafted_in_turn() {
    let mut s = Corporation::new(&Config::default(), 3, 0);
    assert_eq!(s.phase, Phase::Draft);
    assert_eq!(Corporation::legal_actions(&s, Seat(0)).len(), 12 * 4);
    for k in 0..12u8 {
        let seat = k % 3;
        assert_eq!(s.turn, Seat(seat));
        play(
            &mut s,
            seat,
            Action::Place {
                circle: k,
                color: EXAMPLE_TABLE[usize::from(k)],
            },
        );
    }
    assert_eq!(s.phase, Phase::Swap);
    assert_eq!(s.turn, Seat(0));
    assert!(s.table.iter().all(Option::is_some));
}

#[test]
fn a_fourth_counter_of_a_colour_is_refused() {
    let mut s = Corporation::new(&Config::default(), 2, 0);
    for circle in 0..3 {
        play(&mut s, circle % 2, Action::Place { circle, color: R });
    }
    let refused = Action::Place {
        circle: 3,
        color: R,
    };
    assert!(Corporation::apply(&mut s, Seat(1), &refused).is_err());
}

#[test]
fn a_swap_of_equal_colours_is_not_offered() {
    let s = with_table(2, EXAMPLE_TABLE);
    let legal = Corporation::legal_actions(&s, Seat(0));
    // 66 pairs of circles less 4 × 3 of the same colour, plus Skip.
    assert_eq!(legal.len(), 66 - 12 + 1);
    assert!(!legal.contains(&Action::Swap { a: 10, b: 11 }));
}

#[test]
fn an_entry_lands_on_top_and_may_be_promoted() {
    let mut s = with_table(2, EXAMPLE_TABLE);
    play(&mut s, 0, Action::Skip);
    play(&mut s, 0, Action::Enter { color: G, desk: 2 });
    // Green on the green desk: no promotion, the turn is over.
    assert_eq!(s.stacks[2], vec![G]);
    assert_eq!(s.turn, Seat(1));
    play(&mut s, 1, Action::Skip);
    play(&mut s, 1, Action::Enter { color: B, desk: 2 });
    // [C] Blue on the green employee (promotion 3) climbs to the red ground
    // floor; the green uncovered stays, alone on its green desk.
    assert_eq!(s.stacks[2], vec![G]);
    assert_eq!(s.stacks[4], vec![B]);
    assert_eq!(s.outside, [3, 2, 2, 3]);
}

#[test]
fn the_entry_is_skipped_when_nobody_is_outside() {
    let mut s = with_table(2, EXAMPLE_TABLE);
    s.outside = [0; 4];
    play(&mut s, 0, Action::Swap { a: 0, b: 1 });
    assert_eq!(s.turn, Seat(1));
    assert_eq!(s.phase, Phase::Swap);
}

/// [C] Your colour wins even on another's turn; ❓ the game stops at once.
#[test]
fn three_in_the_boardroom_win_at_once() {
    let mut s = with_table(2, EXAMPLE_TABLE);
    s.turn = Seat(1);
    s.stacks[usize::from(BOARDROOM)] = vec![R, R];
    // Red alone on the yellow top floor (promotion 2), and green alone on
    // the red ground floor (promotion 1).
    s.stacks[11] = vec![R];
    s.stacks[4] = vec![G];
    play(&mut s, 1, Action::Skip);
    play(&mut s, 1, Action::Skip);
    assert_eq!(s.eligible(), vec![4, 11]);
    play(&mut s, 1, Action::Promote { from: 11 });
    assert_eq!(s.end, Some(End::Won { seat: Seat(0) }));
    assert_eq!(s.stacks[4], vec![G]);
    assert!(Corporation::pending(&s).is_empty());
    assert_eq!(Corporation::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
}

#[test]
fn a_neutral_colour_in_the_boardroom_wins_nothing() {
    let mut s = with_table(2, EXAMPLE_TABLE);
    s.stacks[usize::from(BOARDROOM)] = vec![Y, Y];
    s.stacks[11] = vec![Y];
    play(&mut s, 0, Action::Skip);
    play(&mut s, 0, Action::Skip);
    assert_eq!(s.arrived(Y), 3);
    assert_eq!(s.end, None);
    assert_eq!(s.turn, Seat(1));
}

#[test]
fn the_losers_rank_by_employees_arrived() {
    let mut s = with_table(4, EXAMPLE_TABLE);
    s.stacks[usize::from(BOARDROOM)] = vec![G, G, G, B, Y, Y];
    s.end = Some(End::Won { seat: Seat(2) });
    s.phase = Phase::Over;
    assert_eq!(
        Corporation::outcome(&s),
        Some(Outcome::Ranked(vec![3, 2, 0, 1]))
    );
}

#[test]
fn quiet_rounds_draw_the_game() {
    let mut s = with_table(2, EXAMPLE_TABLE);
    s.outside = [0; 4];
    for k in 0..QUIET_ROUNDS * 2 {
        play(&mut s, (k % 2) as u8, Action::Skip);
    }
    assert_eq!(s.end, Some(End::Quiet));
    assert_eq!(Corporation::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn only_the_top_employee_is_promoted() {
    let mut s = with_table(2, EXAMPLE_TABLE);
    // Green under yellow on the red ground floor: green on red is on the
    // table, but the yellow covers it; yellow on green is not.
    s.stacks[4] = vec![G, Y];
    assert!(s.eligible().is_empty());
}
