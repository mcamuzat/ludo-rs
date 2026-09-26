use ludo_core::{Game, Outcome, Seat, Viewer};
use yavalath::{all_cells, Action, Cell, Color, Config, State, Yavalath, CELL_COUNT};

fn new(players: u8) -> State {
    Yavalath::new(&Config::default(), players, 0)
}

fn place(s: &mut State, q: i8, r: i8) {
    let seat = Yavalath::pending(s).0[0];
    Yavalath::apply(s, seat, &Action::Place(Cell::new(q, r)))
        .unwrap_or_else(|e| panic!("({q},{r}) : {e}"));
}

fn play(s: &mut State, moves: &[(i8, i8)]) {
    for &(q, r) in moves {
        place(s, q, r);
    }
}

fn ranks(s: &State) -> Option<Vec<u8>> {
    Yavalath::outcome(s).map(|Outcome::Ranked(r)| r)
}

#[test]
fn the_board_has_61_cells_indexed_densely() {
    let indices: Vec<usize> = all_cells().map(|c| c.index().unwrap()).collect();
    assert_eq!(indices, (0..CELL_COUNT).collect::<Vec<_>>());
    assert!(Cell::new(5, 0).index().is_none());
    assert!(Cell::new(3, 2).index().is_none());
}

#[test]
fn four_in_a_row_wins() {
    let mut s = new(2);
    play(
        &mut s,
        &[(0, 0), (0, -4), (1, 0), (4, -4), (3, 0), (2, -4), (2, 0)],
    );
    assert_eq!(ranks(&s), Some(vec![0, 1]));
    assert_eq!(s.lines.len(), 1);
    assert_eq!(s.lines[0].len(), 4);
}

#[test]
fn three_in_a_row_loses() {
    let mut s = new(2);
    play(&mut s, &[(0, 0), (0, -4), (1, 0), (4, -4), (2, 0)]);
    assert_eq!(ranks(&s), Some(vec![1, 0]));
    assert_eq!(s.eliminated, vec![Seat(0)]);
    assert_eq!(s.winner, Some(Seat(1)));
}

#[test]
fn three_and_four_at_once_wins() {
    let mut s = new(2);
    play(
        &mut s,
        &[
            (0, 0),
            (-4, 4),
            (1, 0),
            (-2, 4),
            (3, 0),
            (0, 4),
            (2, -1),
            (-4, 2),
            (2, 1),
            (-2, 2),
            (2, 0),
        ],
    );
    assert_eq!(ranks(&s), Some(vec![0, 1]));
    assert_eq!(s.lines.len(), 1, "only the 4 is highlighted");
    assert_eq!(s.lines[0].len(), 4);
}

#[test]
fn an_occupied_cell_is_refused_and_nothing_changes() {
    let mut s = new(2);
    place(&mut s, 0, 0);
    let before = s.clone();
    assert!(Yavalath::apply(&mut s, Seat(1), &Action::Place(Cell::new(0, 0))).is_err());
    assert!(Yavalath::apply(&mut s, Seat(1), &Action::Place(Cell::new(9, 0))).is_err());
    assert_eq!(s, before);
}

#[test]
fn playing_out_of_turn_is_refused() {
    let mut s = new(2);
    assert!(Yavalath::apply(&mut s, Seat(1), &Action::Place(Cell::new(0, 0))).is_err());
}

#[test]
fn black_may_swap_on_its_first_move_only() {
    let mut s = new(2);
    place(&mut s, 0, 0);
    assert!(Yavalath::legal_actions(&s, Seat(1)).contains(&Action::Swap));
    Yavalath::apply(&mut s, Seat(1), &Action::Swap).unwrap();
    assert_eq!(s.seat_colors, vec![Color::Black, Color::White]);
    assert_eq!(Yavalath::pending(&s).0, vec![Seat(0)]);
    assert_eq!(s.board.get(Cell::new(0, 0)), Some(Color::White));
    place(&mut s, 1, 1);
    assert_eq!(s.board.get(Cell::new(1, 1)), Some(Color::Black));
    assert!(!Yavalath::legal_actions(&s, Seat(1)).contains(&Action::Swap));
}

#[test]
fn no_swap_with_three_players() {
    let mut s = new(3);
    place(&mut s, 0, 0);
    assert!(!Yavalath::legal_actions(&s, Seat(1)).contains(&Action::Swap));
}

/// Two colours with no three in a row in any direction.
fn pattern(c: Cell) -> Color {
    if (c.q + 2 * c.r).rem_euclid(4) < 2 {
        Color::White
    } else {
        Color::Black
    }
}

#[test]
fn a_full_board_is_a_draw() {
    let mut s = new(2);
    let last = Cell::new(0, 0);
    for c in all_cells().filter(|&c| c != last) {
        s.board.set(c, Some(pattern(c)));
    }
    s.stones = 60;
    s.current = Some(pattern(last));
    place(&mut s, 0, 0);
    assert_eq!(ranks(&s), Some(vec![0, 0]));
}

#[test]
fn the_view_shows_the_board_and_whose_turn_it_is() {
    let mut s = new(2);
    place(&mut s, 0, 0);
    let v = Yavalath::view(&s, Viewer::Spectator);
    assert_eq!(v.cells.len(), CELL_COUNT);
    assert_eq!(v.current, Some(Seat(1)));
    assert_eq!(v.last_move, Some(Cell::new(0, 0)));
    let json = serde_json::to_value(&v).unwrap();
    assert!(json.get("seatColors").is_some() && json.get("lastMove").is_some());
    assert_eq!(
        serde_json::to_value(Action::Place(Cell::new(1, -1))).unwrap(),
        serde_json::json!({ "Place": { "q": 1, "r": -1 } })
    );
    assert_eq!(
        serde_json::to_value(Action::Swap).unwrap(),
        serde_json::json!("Swap")
    );
}
