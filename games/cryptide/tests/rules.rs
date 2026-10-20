use cryptide::testing::{fixed_puzzle, table};
use cryptide::{Action, Answer, Cell, Config, Cryptide, Event, Mask, Phase, State, Tokens};
use ludo_core::{Game, Outcome, Pending, Seat};

fn mask(s: &State, seat: Seat) -> Mask {
    s.hint_mask(seat)
}

/// The first space matching `pred`.
fn find(pred: impl Fn(Cell) -> bool) -> Cell {
    (0..108).map(Cell).find(|&c| pred(c)).expect("a space")
}

fn play(s: &mut State, seat: Seat, a: Action) {
    Cryptide::apply(s, seat, &a).unwrap_or_else(|e| panic!("{a:?} by {seat:?}: {e}"));
}

fn refused(s: &mut State, seat: Seat, a: Action) {
    let before = serde_json::to_string(s).expect("state");
    assert!(
        Cryptide::apply(s, seat, &a).is_err(),
        "{a:?} should be refused"
    );
    assert_eq!(serde_json::to_string(s).expect("state"), before);
}

fn tokens(s: &State, c: Cell) -> &Tokens {
    &s.tokens[c.index()]
}

#[test]
fn setup_places_two_cubes_each_in_turn_order() {
    let mut s = Cryptide::new(&Config::default(), 3, 11);
    assert_eq!(s.phase, Phase::Setup);
    let first = s.turn;
    let mut order = Vec::new();
    for _ in 0..6 {
        let seat = s.turn;
        order.push(seat.0);
        let inside = find(|c| mask(&s, seat).contains(c));
        refused(&mut s, seat, Action::Cube(inside));
        let c = find(|c| !mask(&s, seat).contains(c) && tokens(&s, c).cube.is_none());
        play(&mut s, seat, Action::Cube(c));
        if s.phase == Phase::Setup {
            let next = s.turn;
            refused(&mut s, next, Action::Cube(c));
        }
    }
    let f = first.0;
    assert_eq!(
        order,
        vec![f, (f + 1) % 3, (f + 2) % 3, f, (f + 1) % 3, (f + 2) % 3]
    );
    assert_eq!(s.phase, Phase::Turn);
    assert_eq!(s.turn, first);
}

#[test]
fn setup_cube_may_go_on_a_structure() {
    let mut s = Cryptide::new(&Config::default(), 3, 11);
    let seat = s.turn;
    let board = s.puzzle.board();
    let on_structure = (0..108)
        .map(Cell)
        .find(|&c| board.square(c).structure.is_some() && !mask(&s, seat).contains(c))
        .expect("a structure outside the clue");
    play(&mut s, seat, Action::Cube(on_structure));
}

#[test]
fn asking_places_the_target_answer() {
    let mut s = table(3, fixed_puzzle(3));
    let yes = find(|c| mask(&s, Seat(1)).contains(c));
    play(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: yes,
            target: Seat(1),
        },
    );
    assert_eq!(tokens(&s, yes).discs, vec![Seat(1)]);
    assert_eq!((s.phase.clone(), s.turn), (Phase::Turn, Seat(1)));

    let no = find(|c| !mask(&s, Seat(2)).contains(c));
    play(
        &mut s,
        Seat(1),
        Action::Ask {
            cell: no,
            target: Seat(2),
        },
    );
    assert_eq!(tokens(&s, no).cube, Some(Seat(2)));
    assert_eq!((s.phase.clone(), s.turn), (Phase::Penalty, Seat(1)));
}

#[test]
fn penalty_cube_must_be_outside_ones_clue() {
    let mut s = table(3, fixed_puzzle(3));
    let no = find(|c| !mask(&s, Seat(1)).contains(c));
    play(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: no,
            target: Seat(1),
        },
    );
    let inside = find(|c| mask(&s, Seat(0)).contains(c));
    refused(&mut s, Seat(0), Action::Cube(inside));
    refused(&mut s, Seat(0), Action::Cube(no));
    let c = find(|c| !mask(&s, Seat(0)).contains(c) && tokens(&s, c).cube.is_none());
    play(&mut s, Seat(0), Action::Cube(c));
    assert_eq!(tokens(&s, c).cube, Some(Seat(0)));
    assert_eq!((s.phase.clone(), s.turn), (Phase::Turn, Seat(1)));
}

#[test]
fn penalty_is_skipped_without_a_legal_cell() {
    let mut s = table(3, fixed_puzzle(3));
    // Seat 2 has a cube on every space outside seat 0's clue but one.
    let outside: Vec<Cell> = (0..108)
        .map(Cell)
        .filter(|&c| !mask(&s, Seat(0)).contains(c))
        .collect();
    let asked = *outside
        .iter()
        .find(|&&c| !mask(&s, Seat(1)).contains(c))
        .expect("a space both exclude");
    for &c in &outside {
        if c != asked {
            s.tokens[c.index()].cube = Some(Seat(2));
        }
    }
    play(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: asked,
            target: Seat(1),
        },
    );
    assert_eq!((s.phase.clone(), s.turn), (Phase::Turn, Seat(1)));
}

#[test]
fn cannot_act_on_a_cube() {
    let mut s = table(3, fixed_puzzle(3));
    let c = find(|c| mask(&s, Seat(0)).contains(c) && mask(&s, Seat(1)).contains(c));
    s.tokens[c.index()].cube = Some(Seat(2));
    refused(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: c,
            target: Seat(1),
        },
    );
    refused(&mut s, Seat(0), Action::Search(c));
}

#[test]
fn cannot_ask_where_the_target_has_a_token() {
    let mut s = table(3, fixed_puzzle(3));
    let c = find(|c| mask(&s, Seat(1)).contains(c));
    s.tokens[c.index()].discs.push(Seat(1));
    refused(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: c,
            target: Seat(1),
        },
    );
    play(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: c,
            target: Seat(2),
        },
    );
}

#[test]
fn cannot_ask_oneself() {
    let mut s = table(3, fixed_puzzle(3));
    refused(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: Cell(0),
            target: Seat(0),
        },
    );
}

#[test]
fn search_requires_a_cell_of_ones_clue() {
    let mut s = table(3, fixed_puzzle(3));
    let c = find(|c| !mask(&s, Seat(0)).contains(c));
    refused(&mut s, Seat(0), Action::Search(c));
}

#[test]
fn search_stops_at_the_first_cube() {
    let mut s = table(4, fixed_puzzle(4));
    // Seat 1 says yes, seat 2 no: seat 3 is not asked.
    let c = find(|c| {
        mask(&s, Seat(0)).contains(c)
            && mask(&s, Seat(1)).contains(c)
            && !mask(&s, Seat(2)).contains(c)
    });
    play(&mut s, Seat(0), Action::Search(c));
    let t = tokens(&s, c);
    assert_eq!(t.discs, vec![Seat(0), Seat(1)]);
    assert_eq!(t.cube, Some(Seat(2)));
    assert_eq!((s.phase.clone(), s.turn), (Phase::Penalty, Seat(0)));
    assert!(s
        .log
        .iter()
        .any(|e| matches!(e, Event::Searched { by: Seat(0), .. })));
}

#[test]
fn search_skips_and_counts_existing_discs() {
    let mut s = table(3, fixed_puzzle(3));
    let home = s.puzzle.habitat;
    s.tokens[home.index()].discs = vec![Seat(1), Seat(2)];
    play(&mut s, Seat(0), Action::Search(home));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(tokens(&s, home).discs, vec![Seat(1), Seat(2), Seat(0)]);
}

#[test]
fn search_on_own_disc_moves_the_disc() {
    let mut s = table(3, fixed_puzzle(3));
    let home = s.puzzle.habitat;
    s.tokens[home.index()].discs = vec![Seat(0)];
    play(&mut s, Seat(0), Action::Search(home));
    assert_eq!(s.phase, Phase::Relocate { search: home });
    let outside = find(|c| !mask(&s, Seat(0)).contains(c));
    refused(&mut s, Seat(0), Action::Disc(outside));
    refused(&mut s, Seat(0), Action::Disc(home));
    let other = find(|c| c != home && mask(&s, Seat(0)).contains(c));
    play(&mut s, Seat(0), Action::Disc(other));
    assert_eq!(tokens(&s, other).discs, vec![Seat(0)]);
    assert_eq!(s.phase, Phase::Over);
}

#[test]
fn searching_the_habitat_wins() {
    let mut s = table(5, fixed_puzzle(5));
    let home = s.puzzle.habitat;
    play(&mut s, Seat(0), Action::Search(home));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Cryptide::pending(&s), Pending::none());
    assert_eq!(
        Cryptide::outcome(&s),
        Some(Outcome::Ranked(vec![0, 1, 1, 1, 1]))
    );
    assert!(matches!(s.log.last(), Some(Event::Won { seat: Seat(0) })));
    assert_eq!(tokens(&s, home).discs.len(), 5);
}

#[test]
fn answers_come_in_turn_order_from_the_left() {
    let mut s = table(4, fixed_puzzle(4));
    s.turn = Seat(2);
    let home = s.puzzle.habitat;
    play(&mut s, Seat(2), Action::Search(home));
    assert_eq!(
        tokens(&s, home).discs,
        vec![Seat(2), Seat(3), Seat(0), Seat(1)]
    );
}

#[test]
fn only_the_seat_to_move_acts() {
    let mut s = table(3, fixed_puzzle(3));
    let home = s.puzzle.habitat;
    refused(&mut s, Seat(1), Action::Search(home));
    assert!(Cryptide::legal_actions(&s, Seat(1)).is_empty());
    let legal = Cryptide::legal_actions(&s, Seat(0));
    assert!(legal.contains(&Action::Search(home)));
    assert!(!legal
        .iter()
        .any(|a| matches!(a, Action::Cube(_) | Action::Disc(_))));
}

#[test]
fn wrong_phase_actions_are_refused() {
    let mut s = table(3, fixed_puzzle(3));
    let c = find(|c| !mask(&s, Seat(0)).contains(c));
    refused(&mut s, Seat(0), Action::Cube(c));
    refused(&mut s, Seat(0), Action::Disc(c));
    refused(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: Cell(200),
            target: Seat(1),
        },
    );
    refused(
        &mut s,
        Seat(0),
        Action::Ask {
            cell: c,
            target: Seat(9),
        },
    );
}

#[test]
fn answer_follows_the_hint() {
    let s = table(3, fixed_puzzle(3));
    let c = find(|c| mask(&s, Seat(1)).contains(c));
    assert_eq!(s.answer(Seat(1), c), Answer::Disc);
    let d = find(|c| !mask(&s, Seat(1)).contains(c));
    assert_eq!(s.answer(Seat(1), d), Answer::Cube);
}
