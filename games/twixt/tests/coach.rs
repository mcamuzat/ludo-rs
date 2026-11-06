use twixt::coach::{advise, grade, review, review_point, Grade, ReviewPoint};
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
    // Your chances: with the best of these moves, measured like them.
    assert_eq!(a.win, a.best[0].win);
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

fn pt(red_win: f32, best: Option<f32>) -> ReviewPoint {
    ReviewPoint { red_win, best }
}

#[test]
fn a_move_is_graded_against_the_best_alternative() {
    let moves = [place(5, 5), place(15, 15), place(6, 7), place(15, 19)];
    // Red's best at ply 0 keeps 0.6, it played into 0.45: −0.15. Black's
    // best at ply 1 is 0.55 and it got 1 − 0.45. Red's second move gives
    // away 0.5 − 0.1; Black's last one 0.93 − 0.9.
    let points = [
        pt(0.5, Some(0.6)),
        pt(0.45, Some(0.55)),
        pt(0.45, Some(0.5)),
        pt(0.1, Some(0.93)),
        pt(0.1, None),
    ];
    let r = review(&moves, &points).unwrap();
    assert_eq!(
        r.grades,
        vec![
            None,
            Some(Grade::Mistake),
            Some(Grade::Best),
            Some(Grade::Blunder),
            Some(Grade::Good)
        ]
    );
    assert_eq!(r.key, vec![1, 3]);
}

#[test]
fn review_rejects_a_short_curve() {
    let moves = [place(5, 5), place(15, 15)];
    assert!(review(&moves, &[pt(0.5, Some(0.5)), pt(0.5, Some(0.5))]).is_err());
    assert!(review(&[place(0, 0)], &[pt(0.5, Some(0.5)), pt(0.5, None)]).is_err());
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
    assert_eq!(review_point(n, &moves, moves.len()).unwrap(), pt(1.0, None));
    let before = review_point(n, &moves, moves.len() - 1).unwrap();
    // Red to move wins at once: its best move is worth 1.
    assert_eq!(before.best, Some(1.0));
    assert!(review_point(n, &moves, moves.len() + 1).is_err());
}

#[test]
fn the_swap_is_a_candidate_for_black() {
    let a = advise(net().unwrap(), &[place(11, 11)]).unwrap();
    assert!(
        a.best.iter().any(|c| c.action == TwixtAction::Swap),
        "{:?}",
        a.best
    );
    let a = advise(net().unwrap(), &[place(11, 11), place(5, 5)]).unwrap();
    assert!(a.best.iter().all(|c| c.action != TwixtAction::Swap));
}

#[test]
fn a_first_move_counts_the_swap() {
    // After Red's first peg, Black may swap: Red is worth no more than that.
    let n = net().unwrap();
    for first in [place(11, 11), place(1, 0), place(6, 3)] {
        let after = review_point(n, std::slice::from_ref(&first), 1)
            .unwrap()
            .red_win;
        let swapped = review_point(n, &[first.clone(), TwixtAction::Swap], 2)
            .unwrap()
            .red_win;
        assert!(after <= swapped + 1e-6, "{first:?}: {after} > {swapped}");
    }
}
