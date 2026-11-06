use twixt::coach::{advise, grade, red_win_at, review, Grade};
use twixt::net::net;
use twixt::{Point, TwixtAction};

fn place(x: u8, y: u8) -> TwixtAction {
    TwixtAction::Place(Point { x, y })
}

#[test]
fn grade_thresholds() {
    assert_eq!(grade(0.0), Grade::Best);
    assert_eq!(grade(0.01), Grade::Best);
    assert_eq!(grade(0.02), Grade::Good);
    assert_eq!(grade(0.07), Grade::Inaccuracy);
    assert_eq!(grade(0.15), Grade::Mistake);
    assert_eq!(grade(0.4), Grade::Blunder);
}

#[test]
fn advice_on_the_empty_board() {
    let a = advise(net().unwrap(), &[]).unwrap();
    assert_eq!(a.best.len(), 3);
    assert!(a.last.is_none());
    assert!(a.win > 0.0 && a.win < 1.0);
    // Red may play every hole but the 48 of Black's border columns.
    assert_eq!(a.heat.len(), 24 * 22);
    assert!(a.best.windows(2).all(|w| w[0].win >= w[1].win));
}

#[test]
fn advice_after_a_swap() {
    let moves = [place(3, 7), TwixtAction::Swap];
    let a = advise(net().unwrap(), &moves).unwrap();
    let last = a.last.expect("Red's first move is graded");
    assert_eq!(last.ply, 0);
    // Black to move after the swap's reply: its last move is the swap.
    let moves = [place(3, 7), TwixtAction::Swap, place(12, 12)];
    let a = advise(net().unwrap(), &moves).unwrap();
    assert_eq!(a.last.expect("the swap is graded").ply, 1);
}

#[test]
fn a_blunder_is_graded() {
    let moves = [place(5, 5), place(15, 15), place(6, 7), place(15, 19)];
    // Red's win: steady, then Red's second move throws it away.
    let curve = [0.5, 0.5, 0.48, 0.1, 0.12];
    let r = review(&moves, &curve).unwrap();
    assert_eq!(
        r.grades,
        vec![
            None,
            Some(Grade::Best),
            Some(Grade::Best),
            Some(Grade::Blunder),
            Some(Grade::Good)
        ]
    );
    assert_eq!(r.key, vec![3]);
}

#[test]
fn review_rejects_a_short_curve() {
    let moves = [place(5, 5), place(15, 15)];
    assert!(review(&moves, &[0.5, 0.5]).is_err());
    assert!(review(&[place(0, 0)], &[0.5, 0.5]).is_err());
}

#[test]
fn red_win_at_a_finished_game_is_the_result() {
    let mut moves = Vec::new();
    for i in 0..12u8 {
        moves.push(place(5 + i % 2, 2 * i));
        moves.push(place(20, 1 + i));
    }
    moves.push(place(4, 23));
    let n = net().unwrap();
    assert_eq!(red_win_at(n, &moves, moves.len()).unwrap(), 1.0);
    let before = red_win_at(n, &moves, moves.len() - 1).unwrap();
    assert!(before > 0.5, "Red wins in one: {before}");
    assert!(red_win_at(n, &moves, moves.len() + 1).is_err());
}
