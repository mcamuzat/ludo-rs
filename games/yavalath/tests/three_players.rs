use ludo_core::{Game, Outcome, Seat, Viewer};
use yavalath::{Action, Cell, Color, Config, State, Yavalath};

fn three() -> State {
    Yavalath::new(&Config::default(), 3, 0)
}

fn put(s: &mut State, color: Color, cells: &[(i8, i8)]) {
    for &(q, r) in cells {
        s.board.set(Cell::new(q, r), Some(color));
        s.stones += 1;
    }
}

fn places(s: &State, seat: Seat) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Yavalath::legal_actions(s, seat)
        .into_iter()
        .filter_map(|a| match a {
            Action::Place(c) => Some(c),
            Action::Swap => None,
        })
        .collect();
    cells.sort();
    cells
}

fn ranks(s: &State) -> Option<Vec<u8>> {
    Yavalath::outcome(s).map(|Outcome::Ranked(r)| r)
}

#[test]
fn the_next_players_win_must_be_blocked() {
    let mut s = three();
    put(&mut s, Color::Black, &[(0, 0), (1, 0), (3, 0)]);
    assert_eq!(places(&s, Seat(0)), vec![Cell::new(2, 0)]);
    assert_eq!(
        Yavalath::view(&s, Viewer::Spectator).forced,
        vec![Cell::new(2, 0)]
    );
    assert!(Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(-2, 2))).is_err());
}

#[test]
fn a_forced_block_can_eliminate_the_blocker() {
    let mut s = three();
    put(&mut s, Color::Black, &[(0, 0), (1, 0), (3, 0)]);
    put(&mut s, Color::White, &[(2, -1), (2, 1)]);
    Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(2, 0))).unwrap();
    assert_eq!(s.eliminated, vec![Seat(0)]);
    assert_eq!(
        s.board.get(Cell::new(2, 0)),
        Some(Color::White),
        "stones stay"
    );
    assert_eq!(Yavalath::pending(&s).0, vec![Seat(1)]);
    assert_eq!(ranks(&s), None, "two players remain");
}

#[test]
fn an_immediate_win_beats_the_forced_block() {
    let mut s = three();
    put(&mut s, Color::Black, &[(0, 0), (1, 0), (3, 0)]);
    put(&mut s, Color::White, &[(-3, 1), (-2, 1), (0, 1)]);
    assert_eq!(places(&s, Seat(0)), vec![Cell::new(-1, 1), Cell::new(2, 0)]);
}

#[test]
fn the_next_player_skips_the_eliminated() {
    let mut s = three();
    s.eliminated.push(Seat(1));
    assert_eq!(s.next_color(Color::White), Color::Red);
    put(&mut s, Color::Red, &[(0, 0), (1, 0), (3, 0)]);
    assert_eq!(places(&s, Seat(0)), vec![Cell::new(2, 0)]);
}

#[test]
fn the_last_survivor_wins_and_the_eliminated_rank_in_reverse_order() {
    let mut s = three();
    s.eliminated.push(Seat(2));
    put(&mut s, Color::White, &[(0, 0), (1, 0)]);
    Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(2, 0))).unwrap();
    assert_eq!(s.winner, Some(Seat(1)));
    assert_eq!(ranks(&s), Some(vec![1, 0, 2]));
}

#[test]
fn a_four_ranks_the_other_survivor_second() {
    let mut s = three();
    s.eliminated.push(Seat(2));
    put(&mut s, Color::White, &[(0, 0), (1, 0), (3, 0)]);
    Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(2, 0))).unwrap();
    assert_eq!(ranks(&s), Some(vec![0, 1, 2]));
}
