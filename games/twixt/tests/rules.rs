use ludo_core::{Game, Outcome, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use twixt::{replay, Color, Config, Point, Twixt, TwixtAction, TwixtState};

fn p(x: u8, y: u8) -> Point {
    Point { x, y }
}

fn place(x: u8, y: u8) -> TwixtAction {
    TwixtAction::Place(p(x, y))
}

fn seat_to_move(s: &TwixtState) -> Seat {
    Twixt::pending(s).0[0]
}

/// Plays the holes in order, Red first.
fn play(holes: &[(u8, u8)]) -> TwixtState {
    let mut s = Twixt::new(&Config::default(), 2, 0);
    for &(x, y) in holes {
        let seat = seat_to_move(&s);
        Twixt::apply(&mut s, seat, &place(x, y)).unwrap_or_else(|e| panic!("({x},{y}): {e}"));
    }
    s
}

fn has_link(s: &TwixtState, c: Color, a: Point, b: Point) -> bool {
    s.board
        .link_list()
        .iter()
        .any(|&(lc, la, lb)| lc == c && ((la, lb) == (a, b) || (la, lb) == (b, a)))
}

#[test]
fn red_cannot_play_in_blacks_border() {
    let s = Twixt::new(&Config::default(), 2, 0);
    let legal = Twixt::legal_actions(&s, Seat(0));
    assert!(!legal.contains(&place(0, 5)));
    assert!(!legal.contains(&place(23, 5)));
    assert!(legal.contains(&place(5, 0)));
    assert!(!legal.contains(&place(0, 0)));
    let mut s = s;
    assert!(Twixt::apply(&mut s, Seat(0), &place(0, 5)).is_err());
    // Black cannot play in Red's border nor in a corner.
    let s = play(&[(5, 5)]);
    let legal = Twixt::legal_actions(&s, Seat(1));
    assert!(!legal.contains(&place(5, 0)));
    assert!(legal.contains(&place(0, 5)));
    assert!(!legal.contains(&place(23, 23)));
}

#[test]
fn knight_pegs_link_automatically() {
    let s = play(&[(5, 5), (15, 15), (6, 7)]);
    assert!(has_link(&s, Color::Red, p(5, 5), p(6, 7)));
}

#[test]
fn opponent_link_blocks() {
    let s = play(&[
        (2, 5),
        (10, 10),
        (20, 3),
        (11, 12),
        (11, 10),
        (20, 20),
        (10, 12),
    ]);
    assert!(has_link(&s, Color::Black, p(10, 10), p(11, 12)));
    assert!(!has_link(&s, Color::Red, p(11, 10), p(10, 12)));
}

#[test]
fn own_links_may_cross() {
    let s = play(&[(5, 5), (15, 15), (6, 7), (15, 19), (6, 5), (19, 15), (5, 7)]);
    assert!(has_link(&s, Color::Red, p(5, 5), p(6, 7)));
    assert!(has_link(&s, Color::Red, p(6, 5), p(5, 7)));
}

#[test]
fn swap_reflects_the_first_peg() {
    let mut s = play(&[(3, 7)]);
    assert!(Twixt::legal_actions(&s, Seat(1)).contains(&TwixtAction::Swap));
    Twixt::apply(&mut s, Seat(1), &TwixtAction::Swap).unwrap();
    let v = Twixt::view(&s, Viewer::Spectator);
    assert_eq!(v.pegs, vec![(p(7, 3), Color::Black)]);
    assert_eq!(v.turn, Color::Red);
    assert_eq!(v.last, Some(p(7, 3)));
    assert_eq!(seat_to_move(&s), Seat(0));
    // Not for Red, nor later.
    assert!(!Twixt::legal_actions(&s, Seat(0)).contains(&TwixtAction::Swap));
    let mut s = play(&[(3, 7), (10, 10), (12, 12)]);
    assert!(Twixt::apply(&mut s, Seat(1), &TwixtAction::Swap).is_err());
}

#[test]
fn a_chain_between_borders_wins() {
    // A Red chain (x, y) = (5 + i%2, 2i), y = 0 .. 22, then (4, 23) closes it.
    let mut holes = Vec::new();
    for i in 0..12u8 {
        holes.push((5 + i % 2, 2 * i));
        holes.push((20, 1 + i)); // Black, out of the way.
    }
    holes.push((4, 23));
    let s = play(&holes);
    assert_eq!(Twixt::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
    assert!(Twixt::pending(&s).is_empty());
    let v = Twixt::view(&s, Viewer::Spectator);
    assert_eq!(v.result, Some(twixt::Finish::Won(Color::Red)));
    let chain = v.chain.expect("winning chain");
    assert!(chain.contains(&p(5, 0)) && chain.contains(&p(4, 23)));
}

/// Random games: once a colour can no longer connect, it never wins; a game
/// that runs out of holes is a draw; some draws are found with holes left.
#[test]
fn blocked_position_is_a_draw() {
    let mut early_draws = 0;
    for seed in 0..40u64 {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut s = Twixt::new(&Config::default(), 2, 0);
        let mut cut = [false; 2];
        while let Some(&seat) = Twixt::pending(&s).0.first() {
            let legal = Twixt::legal_actions(&s, seat);
            let a = legal.choose(&mut rng).unwrap().clone();
            Twixt::apply(&mut s, seat, &a).unwrap();
            for (i, c) in [Color::Black, Color::Red].into_iter().enumerate() {
                if !s.board.can_still_connect(c) {
                    cut[i] = true;
                }
                if cut[i] {
                    assert_ne!(s.result, Some(Some(c)), "seed {seed}: a cut colour won");
                }
            }
        }
        if s.result == Some(None) {
            assert_eq!(Twixt::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
            let free = (0..24u8)
                .flat_map(|x| (0..24u8).map(move |y| p(x, y)))
                .filter(|&q| s.board.legal(q))
                .count();
            if free > 0 {
                early_draws += 1;
            }
        }
    }
    assert!(
        early_draws > 0,
        "no draw detected before the board filled up"
    );
}

#[test]
fn replay_matches_apply() {
    let s = play(&[(5, 5), (15, 15), (6, 7), (15, 19), (6, 5), (19, 15), (5, 7)]);
    let b = replay(&s.moves).unwrap();
    assert_eq!(b.link_list(), s.board.link_list());
    assert_eq!(b.turn, s.board.turn);
    let mut swapped = play(&[(3, 7)]);
    Twixt::apply(&mut swapped, Seat(1), &TwixtAction::Swap).unwrap();
    assert_eq!(replay(&swapped.moves).unwrap().turn, Color::Red);
    assert!(replay(&[place(0, 0)]).is_err());
}

#[test]
fn illegal_moves_leave_the_state_unchanged() {
    let s = play(&[(5, 5), (15, 15), (6, 7)]);
    let before = serde_json::to_string(&s).unwrap();
    for (seat, a) in [
        (Seat(1), place(5, 5)),
        (Seat(0), place(10, 10)),
        (Seat(1), TwixtAction::Swap),
        (Seat(1), place(30, 2)),
    ] {
        let mut t = s.clone();
        assert!(Twixt::apply(&mut t, seat, &a).is_err(), "{a:?}");
        assert_eq!(serde_json::to_string(&t).unwrap(), before);
    }
}

#[test]
fn state_round_trips_through_json() {
    let s = play(&[(5, 5), (15, 15), (6, 7)]);
    let back: TwixtState = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(back.board.link_list(), s.board.link_list());
    assert_eq!(back.moves, s.moves);
}

#[test]
fn the_view_tells_a_draw_from_a_game_in_progress() {
    let s = play(&[(5, 5)]);
    let v = serde_json::to_value(Twixt::view(&s, Viewer::Spectator)).unwrap();
    assert_eq!(v["result"], serde_json::Value::Null);
    let mut drawn = s.clone();
    drawn.result = Some(None);
    let v = serde_json::to_value(Twixt::view(&drawn, Viewer::Spectator)).unwrap();
    assert_eq!(v["result"], serde_json::json!("Draw"));
}
