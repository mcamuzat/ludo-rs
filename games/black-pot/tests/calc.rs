use black_pot::board::{kind_of, Kind};
use black_pot::calc::{announce, dice_used, eval, values, Expr, Op};

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

const D: [u8; 3] = [4, 2, 6];

#[test]
fn eval_follows_the_four_operations() {
    assert_eq!(eval(&op(Op::Sub, die(2), die(0)), &D), Ok(2));
    let twelve = op(Op::Div, op(Op::Mul, die(2), die(0)), die(1));
    assert_eq!(eval(&twelve, &D), Ok(12));
    let three = op(Op::Div, die(2), op(Op::Sub, die(0), die(1)));
    assert_eq!(eval(&three, &D), Ok(3));
    let zero = op(Op::Sub, op(Op::Add, die(1), die(0)), die(2));
    assert_eq!(eval(&zero, &D), Ok(0));
    assert_eq!(eval(&die(1), &D), Ok(2));
}

#[test]
fn eval_refuses_negatives_inexact_divisions_and_division_by_zero() {
    let negative = eval(&op(Op::Sub, die(1), die(2)), &D).unwrap_err();
    assert_eq!(negative.0, "2 − 6 donne un nombre négatif.");
    let by_zero = op(Op::Div, die(0), op(Op::Sub, die(1), die(2)));
    assert!(eval(&by_zero, &[6, 3, 3]).is_err());
    let by_zero = eval(
        &op(Op::Div, die(0), op(Op::Sub, die(1), die(2))),
        &[6, 3, 3],
    )
    .unwrap_err();
    assert_eq!(by_zero.0, "On ne divise pas par 0.");
    let inexact = eval(&op(Op::Div, die(0), die(1)), &[5, 2, 1]).unwrap_err();
    assert_eq!(inexact.0, "5 ÷ 2 ne tombe pas juste.");
}

#[test]
fn eval_refuses_unknown_dice_and_more_than_three_dice() {
    assert!(eval(&die(3), &D).is_err());
    let four = op(
        Op::Add,
        op(Op::Add, die(0), die(1)),
        op(Op::Add, die(2), die(0)),
    );
    assert!(eval(&four, &D).is_err());
}

#[test]
fn dice_used_lists_indices_in_reading_order() {
    let e = op(Op::Div, die(2), op(Op::Sub, die(0), die(1)));
    assert_eq!(dice_used(&e), vec![2, 0, 1]);
}

#[test]
fn announce_chains_the_calculation_as_the_rules_do() {
    let twelve = op(Op::Div, op(Op::Mul, die(2), die(0)), die(1));
    assert_eq!(announce(&twelve, &D).unwrap(), "6 × 4 = 24 ÷ 2 = 12");
    let three = op(Op::Div, die(2), op(Op::Sub, die(0), die(1)));
    assert_eq!(announce(&three, &D).unwrap(), "4 − 2 = 2, 6 ÷ 2 = 3");
    let fourteen = op(Op::Add, die(2), op(Op::Mul, die(0), die(1)));
    assert_eq!(announce(&fourteen, &D).unwrap(), "4 × 2 = 8 + 6 = 14");
    assert_eq!(
        announce(&op(Op::Sub, die(2), die(0)), &D).unwrap(),
        "6 − 4 = 2"
    );
    assert_eq!(announce(&die(0), &D).unwrap(), "4");
    assert!(announce(&op(Op::Sub, die(1), die(2)), &D).is_err());
}

#[test]
fn values_lists_what_a_group_reaches() {
    let all = values(&D, &[0, 1, 2]);
    for v in [0, 3, 12, 14] {
        assert!(all.contains_key(&v), "{v}");
    }
    for (&v, e) in &all {
        assert_eq!(eval(e, &D), Ok(v));
        let mut used = dice_used(e);
        used.sort();
        assert_eq!(used, vec![0, 1, 2]);
    }
    assert_eq!(values(&D, &[1]).into_keys().collect::<Vec<_>>(), vec![2]);
}

#[test]
fn no_pair_of_dice_reaches_a_black_pot() {
    for a in 1..=6 {
        for b in 1..=6 {
            for c in 1..=6 {
                let dice = [a, b, c];
                for pair in [[0, 1], [0, 2], [1, 2]] {
                    for v in values(&dice, &pair).into_keys() {
                        let kind = u8::try_from(v).ok().and_then(kind_of);
                        assert_ne!(kind, Some(Kind::BlackPot), "{dice:?} {pair:?} {v}");
                    }
                }
            }
        }
    }
}
