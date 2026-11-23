use ludo_core::{Determinize, Game, Outcome, Seat, Viewer};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use wu_hsing::{values, Action, Board, Config, State, WuHsing};

fn id(a: u8, b: u8) -> u8 {
    (0..30).find(|&i| values(i) == (a, b)).unwrap()
}

/// The same values in the other set.
fn twin(i: u8) -> u8 {
    if i < 15 {
        i + 15
    } else {
        i - 15
    }
}

fn fresh(players: u8, seed: u64) -> State {
    WuHsing::new(&Config::default(), players, seed)
}

/// A game on the known square 1-2 over 3-4, seat `turn` to play.
fn on_square(players: u8, hands: Vec<Vec<u8>>, first: u8, turn: u8) -> State {
    let mut s = fresh(players, 1);
    s.board = Board::new([id(1, 2), id(3, 4)]);
    s.hands = hands;
    s.discarded = None;
    s.first = Seat(first);
    s.turn = Some(Seat(turn));
    s
}

/// A position where the seat to play has no placement, found by playing
/// games out.
fn stuck(players: u8) -> (State, Seat) {
    for seed in 0..400 {
        let mut s = fresh(players, seed);
        let mut k = seed as usize;
        while let Some(seat) = s.turn {
            let moves = s.moves(seat);
            if moves == [(Action::Pass, 0)] {
                return (s, seat);
            }
            k = k.wrapping_mul(31).wrapping_add(7);
            let (a, _) = moves[k % moves.len()];
            WuHsing::apply(&mut s, seat, &a).unwrap();
        }
    }
    panic!("no stuck position found");
}

#[test]
fn new_game_sets_the_table() {
    let s = fresh(3, 9);
    assert!(s.hands.iter().all(|h| h.len() == 9));
    assert_eq!(s.board.placed().len(), 2);
    assert_eq!(s.turn, Some(s.first));
    assert_eq!(s.scores, vec![0, 0, 0]);
    assert!(s.discarded.is_some());
}

#[test]
fn placing_scores_and_passes_the_turn() {
    let mut s = fresh(2, 3);
    let seat = s.turn.unwrap();
    let (action, points) = s.moves(seat)[0];
    assert!(matches!(action, Action::Place(_)));
    WuHsing::apply(&mut s, seat, &action).unwrap();
    assert_eq!(s.scores[seat.index()], points as i32);
    assert_eq!(s.turn, Some(Seat(1 - seat.0)));
    assert_eq!(s.passes, 0);
    assert_eq!(s.hands[seat.index()].len(), 13);
    assert_eq!(s.last, Some((seat, action, points)));
}

#[test]
fn pass_only_when_stuck() {
    let s = fresh(2, 4);
    let seat = s.turn.unwrap();
    assert!(!WuHsing::legal_actions(&s, seat).contains(&Action::Pass));
    let mut t = s.clone();
    let err = WuHsing::apply(&mut t, seat, &Action::Pass).unwrap_err();
    assert!(err.0.contains("Un coup est possible"), "{}", err.0);
    assert_eq!(t, s);

    let (mut s, seat) = stuck(2);
    assert_eq!(WuHsing::legal_actions(&s, seat), vec![Action::Pass]);
    WuHsing::apply(&mut s, seat, &Action::Pass).unwrap();
    assert_eq!(s.passes, 1);
}

#[test]
fn all_passing_ends_the_game() {
    let (mut s, seat) = stuck(2);
    let other = Seat(1 - seat.0);
    // The same values in the other hand: stuck too.
    s.hands[other.index()] = s.hands[seat.index()].iter().map(|&i| twin(i)).collect();
    s.hands[other.index()].sort_unstable();
    WuHsing::apply(&mut s, seat, &Action::Pass).unwrap();
    assert_eq!(s.turn, Some(other));
    WuHsing::apply(&mut s, other, &Action::Pass).unwrap();
    assert_eq!(s.turn, None);
    assert!(s.finish.is_some());
    assert!(WuHsing::outcome(&s).is_some());
}

#[test]
fn twins_and_doubles_are_listed_once() {
    let mut hand = vec![id(2, 3), twin(id(2, 3)), id(4, 4)];
    hand.sort_unstable();
    let s = on_square(2, vec![hand, vec![id(1, 1)]], 0, 0);
    let legal = WuHsing::legal_actions(&s, Seat(0));
    assert!(!legal.is_empty());
    let mut seen = std::collections::HashSet::new();
    for a in &legal {
        let Action::Place(p) = a else {
            panic!("a placement exists")
        };
        assert_ne!(p.domino, twin(id(2, 3)));
        assert!(!(p.domino == id(4, 4) && p.flipped));
        assert!(seen.insert(*p));
    }
    let Some(Action::Place(p)) = legal
        .iter()
        .find(|a| matches!(a, Action::Place(p) if p.domino == id(2, 3)))
    else {
        panic!("2-3 has a placement")
    };
    let mut t = s.clone();
    let mut other = *p;
    other.domino = twin(id(2, 3));
    let err = WuHsing::apply(&mut t, Seat(0), &Action::Place(other)).unwrap_err();
    assert!(err.0.contains("identiques"), "{}", err.0);
    assert_eq!(t, s);
}

#[test]
fn the_round_ends_at_the_first_player() {
    // Seat 1 lays its last domino: seat 2 plays once more, then it is over.
    let mut s = on_square(
        3,
        vec![vec![0, 1], vec![id(5, 5)], vec![twin(id(5, 5))]],
        0,
        1,
    );
    let (a, _) = s.moves(Seat(1))[0];
    WuHsing::apply(&mut s, Seat(1), &a).unwrap();
    assert!(s.last_round);
    assert_eq!(s.turn, Some(Seat(2)));
    let (a, _) = s.moves(Seat(2))[0];
    WuHsing::apply(&mut s, Seat(2), &a).unwrap();
    assert_eq!(s.turn, None);
    assert!(s.finish.is_some());

    // Seat 2, last of the round, empties its hand: over at once.
    let mut s = on_square(3, vec![vec![0, 1], vec![2, 3], vec![id(5, 5)]], 0, 2);
    let (a, _) = s.moves(Seat(2))[0];
    WuHsing::apply(&mut s, Seat(2), &a).unwrap();
    assert_eq!(s.turn, None);

    // Seat 0, the first player, empties its hand: seats 1 and 2 still play.
    let mut s = on_square(3, vec![vec![id(5, 5)], vec![2, 3], vec![4, 6]], 0, 0);
    let (a, _) = s.moves(Seat(0))[0];
    WuHsing::apply(&mut s, Seat(0), &a).unwrap();
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn final_penalties_and_ranking() {
    // Seat 0 keeps a double and a single (−30); seat 1 lays 5-5 for 18.
    let mut s = on_square(2, vec![vec![id(1, 1), id(1, 2)], vec![id(5, 5)]], 0, 1);
    s.scores = vec![100, 50];
    let (a, points) = s.moves(Seat(1))[0];
    assert_eq!(points, 18);
    WuHsing::apply(&mut s, Seat(1), &a).unwrap();
    let f = s.finish.clone().expect("over");
    assert_eq!(f.penalties, vec![30, 0]);
    assert_eq!(f.totals, vec![70, 68]);
    assert_eq!(WuHsing::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));

    let mut s = on_square(2, vec![vec![id(1, 1), id(1, 2)], vec![id(5, 5)]], 0, 1);
    s.scores = vec![98, 50];
    WuHsing::apply(&mut s, Seat(1), &a).unwrap();
    assert_eq!(WuHsing::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn view_hides_other_hands() {
    let s = fresh(3, 5);
    let turn = s.turn.unwrap();
    for seat in 0..3u8 {
        let v = WuHsing::view(&s, Viewer::Seat(Seat(seat)));
        let ids: Vec<u8> = v.hand.iter().map(|d| d.id).collect();
        assert_eq!(ids, s.hands[usize::from(seat)]);
        assert_eq!(v.hand_sizes, vec![9, 9, 9]);
        assert_eq!(v.legal.is_empty(), Seat(seat) != turn);
        assert_eq!(v.placed.len(), 2);
    }
    let v = WuHsing::view(&s, Viewer::Spectator);
    assert!(v.hand.is_empty() && v.legal.is_empty() && v.me.is_none());
}

#[test]
fn determinize_keeps_hand_sizes() {
    let s = fresh(3, 6);
    let d = WuHsing::determinize(&s, Seat(0), &mut ChaCha8Rng::seed_from_u64(2));
    assert_eq!(d.hands[0], s.hands[0]);
    assert_eq!(d.board, s.board);
    let sizes = |s: &State| s.hands.iter().map(Vec::len).collect::<Vec<_>>();
    assert_eq!(sizes(&d), sizes(&s));
    assert_eq!(d.discarded.is_some(), s.discarded.is_some());
    let hidden = |s: &State| {
        let mut v: Vec<u8> = s.hands[1..].concat();
        v.extend(s.discarded);
        v.sort_unstable();
        v
    };
    assert_eq!(hidden(&d), hidden(&s));
}
