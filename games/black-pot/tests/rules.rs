use black_pot::board::{cells, CellId};
use black_pot::calc::{Expr, Op};
use black_pot::testing::{free_cell, position};
use black_pot::{
    score, Action, BlackPot, Config, End, Group, PlayedGroup, State, TurnRecord, QUIET_ROUNDS,
};
use ludo_core::{Game, Outcome, Seat};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn die(i: u8) -> Expr {
    Expr::Die(i)
}

fn op(op: Op, left: Expr, right: Expr) -> Expr {
    Expr::Op {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn on(expr: Expr, cell: CellId) -> Group {
    Group {
        expr,
        cell: Some(cell),
    }
}

fn nothing(expr: Expr) -> Group {
    Group { expr, cell: None }
}

fn play(groups: Vec<Group>) -> Action {
    Action::Play { groups }
}

/// `apply` refuses `action` with `message` and leaves the state unchanged.
fn refused(s: &State, seat: u8, action: &Action, message: &str) {
    let before = serde_json::to_string(s).unwrap();
    let mut after = s.clone();
    let err = BlackPot::apply(&mut after, Seat(seat), action).unwrap_err();
    assert_eq!(err.0, message);
    assert_eq!(serde_json::to_string(&after).unwrap(), before);
}

#[test]
fn the_three_dice_used_separately_take_three_cells() {
    let mut s = position(2, &[], [4, 2, 6], &[0]);
    let (c4, c2, c6) = (free_cell(&s, 4), free_cell(&s, 2), free_cell(&s, 6));
    BlackPot::apply(
        &mut s,
        Seat(0),
        &play(vec![on(die(0), c4), on(die(1), c2), on(die(2), c6)]),
    )
    .unwrap();
    for c in [c4, c2, c6] {
        assert_eq!(s.owner[usize::from(c)], Some(0));
    }
    assert_eq!(s.pawns_left, [37, 40]);
    let Some(TurnRecord::Played { groups, .. }) = &s.last[0] else {
        panic!("played");
    };
    assert_eq!(
        groups[0],
        PlayedGroup {
            text: "4".into(),
            value: 4,
            cell: Some(c4)
        }
    );
}

#[test]
fn every_number_must_find_a_free_cell() {
    // Every 6 taken: 4, 2 and 6 played apart is not a move.
    let sixes: Vec<(CellId, u8)> = cells()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.value == 6)
        .map(|(id, _)| (id as CellId, 1))
        .collect();
    let s = position(2, &sixes, [4, 2, 6], &[0]);
    let (c4, c2) = (free_cell(&s, 4), free_cell(&s, 2));
    refused(
        &s,
        0,
        &play(vec![on(die(0), c4), on(die(1), c2), on(die(2), sixes[0].0)]),
        "Cette case est déjà occupée.",
    );
    refused(
        &s,
        0,
        &play(vec![on(die(0), c4), on(die(1), c2), nothing(die(2))]),
        "Ce calcul donne 6 : choisis une case.",
    );
}

#[test]
fn a_pair_and_a_single_die() {
    let mut s = position(2, &[], [4, 2, 6], &[0]);
    let (c2, c2b) = (free_cell(&s, 2), {
        let first = free_cell(&s, 2);
        cells()
            .iter()
            .enumerate()
            .position(|(id, c)| c.value == 2 && id as CellId != first)
            .unwrap() as CellId
    });
    // 6 − 4 = 2 and the 2: two cells bearing 2.
    BlackPot::apply(
        &mut s,
        Seat(0),
        &play(vec![on(op(Op::Sub, die(2), die(0)), c2), on(die(1), c2b)]),
    )
    .unwrap();
    assert_eq!(s.pawns_left[0], 38);
}

#[test]
fn a_black_pot_takes_the_three_dice() {
    let mut s = position(2, &[], [4, 2, 6], &[0]);
    let c14 = free_cell(&s, 14);
    let fourteen = op(Op::Add, op(Op::Mul, die(0), die(1)), die(2));
    BlackPot::apply(&mut s, Seat(0), &play(vec![on(fourteen, c14)])).unwrap();
    let Some(TurnRecord::Played { groups, .. }) = &s.last[0] else {
        panic!("played");
    };
    assert_eq!(groups[0].text, "4 × 2 = 8 + 6 = 14");
}

#[test]
fn a_wrong_total_is_refused_with_the_announced_calculation() {
    let s = position(2, &[], [4, 2, 6], &[0]);
    let c14 = free_cell(&s, 14);
    let twelve = op(Op::Div, op(Op::Mul, die(2), die(0)), die(1));
    refused(
        &s,
        0,
        &play(vec![on(twelve, c14)]),
        "6 × 4 = 24 ÷ 2 = 12, pas 14",
    );
}

#[test]
fn dice_used_twice_forgotten_or_unknown_and_unknown_cells_are_refused() {
    let s = position(2, &[], [4, 2, 6], &[0]);
    let (c4, c2) = (free_cell(&s, 4), free_cell(&s, 2));
    let all = "Il faut employer chacun des 3 dés, une seule fois.";
    refused(&s, 0, &play(vec![on(die(0), c4), on(die(1), c2)]), all);
    refused(
        &s,
        0,
        &play(vec![
            on(die(0), c4),
            on(die(1), c2),
            on(die(1), free_cell(&s, 2)),
        ]),
        all,
    );
    refused(
        &s,
        0,
        &play(vec![on(die(0), c4), on(die(1), c2), on(die(7), 0)]),
        all,
    );
    refused(&s, 0, &play(vec![]), all);
    let twelve = op(Op::Div, op(Op::Mul, die(2), die(0)), die(1));
    refused(
        &s,
        0,
        &play(vec![on(twelve.clone(), 200)]),
        "Cette case n'existe pas.",
    );
    refused(
        &s,
        1,
        &play(vec![on(twelve, free_cell(&s, 12))]),
        "Ce n'est pas ton tour.",
    );
    refused(
        &s,
        0,
        &play(vec![on(die(0), c4), on(op(Op::Add, die(1), die(2)), c4)]),
        "2 + 6 = 8, pas 4",
    );
}

#[test]
fn zero_places_no_pawn() {
    let mut s = position(2, &[], [4, 2, 6], &[0]);
    let zero = op(Op::Sub, op(Op::Add, die(1), die(0)), die(2));
    BlackPot::apply(&mut s, Seat(0), &play(vec![nothing(zero.clone())])).unwrap();
    assert_eq!(s.pawns_left, [40, 40]);
    assert_eq!(s.quiet, 1);
    let s = position(2, &[], [4, 2, 6], &[0]);
    refused(
        &s,
        0,
        &play(vec![on(zero, free_cell(&s, 1))]),
        "Un calcul qui donne 0 ne pose pas de pion.",
    );
}

#[test]
fn a_pair_worth_zero_with_a_single_die_places_one_pawn() {
    let mut s = position(2, &[], [3, 3, 5], &[0]);
    let c5 = free_cell(&s, 5);
    BlackPot::apply(
        &mut s,
        Seat(0),
        &play(vec![nothing(op(Op::Sub, die(0), die(1))), on(die(2), c5)]),
    )
    .unwrap();
    assert_eq!(s.pawns_left[0], 39);
    assert_eq!(s.quiet, 0);
}

#[test]
fn not_found_is_legal_only_beside_a_move() {
    let s = position(2, &[], [4, 2, 6], &[0]);
    let legal = BlackPot::legal_actions(&s, Seat(0));
    assert_eq!(legal.last(), Some(&Action::NotFound));
    assert!(legal.len() > 1);
    assert!(BlackPot::legal_actions(&s, Seat(1)).is_empty());
    let mut after = s.clone();
    BlackPot::apply(&mut after, Seat(0), &Action::NotFound).unwrap();
    assert_eq!(
        after.last[0],
        Some(TurnRecord::NotFound { dice: [4, 2, 6] })
    );
    assert_eq!(after.pawns_left, [40, 40]);
    assert_eq!(
        BlackPot::timeout_action(&s, Seat(0)),
        Some(Action::NotFound)
    );
}

/// Random games among the moves that place pawns: no pending seat is ever
/// left with « Je ne trouve pas » alone, and every game ends.
#[test]
fn a_pending_seat_always_has_a_move() {
    let mut rng = ChaCha8Rng::seed_from_u64(7);
    for g in 0..40u64 {
        let players = if g % 2 == 0 { 2 } else { 4 };
        let mut s = BlackPot::new(&Config::default(), players, g);
        let mut steps = 0;
        while let Some(&seat) = BlackPot::pending(&s).0.first() {
            let legal = BlackPot::legal_actions(&s, seat);
            assert!(legal.iter().any(|a| matches!(a, Action::Play { .. })));
            let plays = legal.len() - 1;
            let pick = rng.gen::<u32>() as usize % plays;
            BlackPot::apply(&mut s, seat, &legal[pick]).unwrap();
            steps += 1;
            assert!(steps < 5_000, "game {g} does not end");
        }
        assert!(BlackPot::outcome(&s).is_some());
    }
}

#[test]
fn a_player_without_a_move_passes_by_itself() {
    // Every cell taken but a 23, which 1, 2 and 4 cannot reach (nor 0).
    let keep = cells().iter().position(|c| c.value == 23).unwrap() as CellId;
    let owners: Vec<(CellId, u8)> = (0..cells().len() as CellId)
        .filter(|&c| c != keep)
        .map(|c| (c, c % 2))
        .collect();
    let s = position(2, &owners, [1, 2, 4], &[0]);
    assert_eq!(s.last[0], Some(TurnRecord::Passed { dice: [1, 2, 4] }));
    assert!(s.turns >= 1);
}

#[test]
fn a_team_cannot_place_more_pawns_than_it_has() {
    let mut s = position(2, &[], [4, 2, 6], &[0]);
    s.pawns_left[0] = 2;
    let (c4, c2, c6) = (free_cell(&s, 4), free_cell(&s, 2), free_cell(&s, 6));
    refused(
        &s,
        0,
        &play(vec![on(die(0), c4), on(die(1), c2), on(die(2), c6)]),
        "Il ne te reste que 2 pions.",
    );
    let legal = BlackPot::legal_actions(&s, Seat(0));
    for action in &legal {
        if let Action::Play { groups } = action {
            assert!(groups.iter().filter(|g| g.cell.is_some()).count() <= 2);
        }
    }
    assert!(legal
        .iter()
        .any(|a| matches!(a, Action::Play { groups } if groups.iter().all(|g| g.cell.is_none()))));
}

#[test]
fn at_two_players_each_plays_its_own_roll() {
    let mut s = BlackPot::new(&Config::default(), 2, 3);
    let first = s.roller;
    assert_eq!(s.queue, vec![first]);
    let rolls = s.rolls;
    BlackPot::apply(&mut s, first, &Action::NotFound).unwrap();
    assert_eq!(s.roller, Seat(1 - first.0));
    assert_eq!(s.queue, vec![s.roller]);
    assert_eq!(s.rolls, rolls + 1);
}

#[test]
fn at_four_players_the_roller_plays_its_roll_last() {
    let mut s = BlackPot::new(&Config::default(), 4, 5);
    let r = s.roller.0;
    let order: Vec<Seat> = (1..=4).map(|k| Seat((r + k) % 4)).collect();
    assert_eq!(s.queue, order);
    let dice = s.dice;
    for (k, &seat) in order.iter().enumerate() {
        assert_eq!(s.dice, dice, "same roll for all four");
        assert_eq!(BlackPot::pending(&s).0, vec![seat]);
        BlackPot::apply(&mut s, seat, &Action::NotFound).unwrap();
        if k < 3 {
            assert_eq!(s.roller.0, r);
        }
    }
    assert_eq!(s.roller, Seat((r + 1) % 4));
    assert_eq!(s.queue[0], Seat((r + 2) % 4));
}

#[test]
fn the_last_pawn_ends_the_game_even_mid_roll() {
    let mut s = position(4, &[], [4, 2, 6], &[2, 3, 0, 1]);
    s.pawns_left[0] = 1;
    let twelve = op(Op::Div, op(Op::Mul, die(2), die(0)), die(1));
    let c12 = free_cell(&s, 12);
    BlackPot::apply(&mut s, Seat(2), &play(vec![on(twelve, c12)])).unwrap();
    assert_eq!(s.end, Some(End::AllPlaced { team: 0 }));
    assert!(BlackPot::pending(&s).is_empty());
    assert!(BlackPot::legal_actions(&s, Seat(3)).is_empty());
    // Team 0: no line, no pawn left; team 1: 40 pawns, 400 points.
    assert_eq!(score(&s, 0), 0);
    assert_eq!(score(&s, 1), 400);
    assert_eq!(
        BlackPot::outcome(&s),
        Some(Outcome::Ranked(vec![1, 0, 1, 0]))
    );
}

#[test]
fn lines_score_when_completed() {
    // Four pawns of team 0 on row 5, columns 3 to 6; the fifth on column 7
    // (a 22) makes three Black-Pots: 250.
    let row5 = |col: i8| black_pot::board::cell_at(5, col).unwrap();
    let owners: Vec<(CellId, u8)> = (3..7).map(|c| (row5(c), 0)).collect();
    let mut s = position(2, &owners, [4, 2, 6], &[0]);
    // 4 × 6 − 2 = 22.
    let twenty_two = op(Op::Sub, op(Op::Mul, die(0), die(2)), die(1));
    BlackPot::apply(&mut s, Seat(0), &play(vec![on(twenty_two, row5(7))])).unwrap();
    assert_eq!(s.alignments.len(), 1);
    assert_eq!(s.alignments[0].points, 250);
    let Some(TurnRecord::Played { points, .. }) = &s.last[0] else {
        panic!("played");
    };
    assert_eq!(*points, 250);
    assert_eq!(score(&s, 0), 250 + 10 * u32::from(s.pawns_left[0]));
    assert!(s.counted[usize::from(row5(5))][0]);
}

#[test]
fn equal_scores_share_the_rank() {
    let mut s = position(2, &[], [4, 2, 6], &[0]);
    s.end = Some(End::QuietLimit);
    assert_eq!(BlackPot::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn ten_rounds_without_a_pawn_stop_the_game() {
    let mut s = BlackPot::new(&Config::default(), 2, 11);
    let mut turns = 0;
    while let Some(&seat) = BlackPot::pending(&s).0.first() {
        BlackPot::apply(&mut s, seat, &Action::NotFound).unwrap();
        turns += 1;
        assert!(turns <= QUIET_ROUNDS * 2);
    }
    assert_eq!(s.end, Some(End::QuietLimit));
    assert_eq!(s.quiet, QUIET_ROUNDS * 2);
}

#[test]
fn a_pawn_placed_resets_the_quiet_count() {
    let mut s = position(2, &[], [4, 2, 6], &[0]);
    s.quiet = 15;
    let twelve = op(Op::Div, op(Op::Mul, die(2), die(0)), die(1));
    let c12 = free_cell(&s, 12);
    BlackPot::apply(&mut s, Seat(0), &play(vec![on(twelve, c12)])).unwrap();
    assert_eq!(s.quiet, 0);
}

#[test]
fn black_pot_is_for_two_or_four() {
    assert!(BlackPot::check_config(&Config::default(), 2).is_ok());
    assert!(BlackPot::check_config(&Config::default(), 4).is_ok());
    assert_eq!(
        BlackPot::check_config(&Config::default(), 3).unwrap_err().0,
        "Black-Pot se joue à 2 ou à 4."
    );
}

#[test]
fn the_dice_of_a_seed_are_pinned() {
    let mut s = BlackPot::new(&Config::default(), 2, 42);
    let mut rolls = vec![s.dice];
    while rolls.len() < 5 {
        let seat = BlackPot::pending(&s).0[0];
        BlackPot::apply(&mut s, seat, &Action::NotFound).unwrap();
        rolls.push(s.dice);
    }
    assert_eq!(rolls, PINNED);
}

const PINNED: [[u8; 3]; 5] = [[4, 4, 5], [6, 3, 6], [4, 3, 2], [5, 4, 6], [5, 6, 2]];
