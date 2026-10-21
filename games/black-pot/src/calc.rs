//! [R] The three dice combined with the four operations + − × ÷.

use std::collections::BTreeMap;

use ludo_core::Illegal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

pub const OPS: [Op; 4] = [Op::Add, Op::Sub, Op::Mul, Op::Div];

impl Op {
    pub fn sign(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "−",
            Op::Mul => "×",
            Op::Div => "÷",
        }
    }

    fn commutative(self) -> bool {
        matches!(self, Op::Add | Op::Mul)
    }

    /// ❓ Every result stays a whole number, zero or more: a division must
    /// fall right, and nothing is divided by 0.
    fn apply(self, l: u32, r: u32) -> Result<u32, Illegal> {
        match self {
            Op::Add => Ok(l + r),
            Op::Sub => l
                .checked_sub(r)
                .ok_or_else(|| Illegal::new(format!("{l} − {r} donne un nombre négatif."))),
            Op::Mul => Ok(l * r),
            Op::Div if r == 0 => Err(Illegal::new("On ne divise pas par 0.")),
            Op::Div if l % r != 0 => Err(Illegal::new(format!("{l} ÷ {r} ne tombe pas juste."))),
            Op::Div => Ok(l / r),
        }
    }
}

/// A calculation over the dice of the roll: `Die(i)` is the face of die `i`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expr {
    Die(u8),
    Op {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

impl Expr {
    fn op(op: Op, left: Expr, right: Expr) -> Self {
        Expr::Op {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn leaves(&self) -> usize {
        match self {
            Expr::Die(_) => 1,
            Expr::Op { left, right, .. } => left.leaves() + right.leaves(),
        }
    }
}

/// The dice `expr` uses, in reading order (repeats included).
pub fn dice_used(expr: &Expr) -> Vec<u8> {
    match expr {
        Expr::Die(i) => vec![*i],
        Expr::Op { left, right, .. } => {
            let mut used = dice_used(left);
            used.extend(dice_used(right));
            used
        }
    }
}

fn value(expr: &Expr, dice: &[u8; 3]) -> Result<u32, Illegal> {
    match expr {
        Expr::Die(i) => dice
            .get(usize::from(*i))
            .map(|&face| u32::from(face))
            .ok_or_else(|| Illegal::new("Ce dé n'existe pas.")),
        Expr::Op { op, left, right } => op.apply(value(left, dice)?, value(right, dice)?),
    }
}

/// The value of `expr` with these dice.
pub fn eval(expr: &Expr, dice: &[u8; 3]) -> Result<u32, Illegal> {
    if expr.leaves() > 3 {
        return Err(Illegal::new("Un calcul emploie au plus 3 dés."));
    }
    value(expr, dice)
}

/// [R] The calculation as it is announced aloud, chained the way the rules
/// write it: « 6 × 4 = 24 ÷ 2 = 12 ».
pub fn announce(expr: &Expr, dice: &[u8; 3]) -> Result<String, Illegal> {
    let total = eval(expr, dice)?;
    let Expr::Op { op, left, right } = expr else {
        return Ok(total.to_string());
    };
    let text = |e: &Expr| -> Result<String, Illegal> { Ok(value(e, dice)?.to_string()) };
    Ok(match (left.as_ref(), right.as_ref()) {
        (Expr::Die(_), Expr::Die(_)) => {
            format!("{} {} {} = {total}", text(left)?, op.sign(), text(right)?)
        }
        (inner @ Expr::Op { .. }, single @ Expr::Die(_)) => {
            chained(inner, *op, single, total, dice)?
        }
        (single, inner) if op.commutative() => chained(inner, *op, single, total, dice)?,
        (single, inner) => format!(
            "{}, {} {} {} = {total}",
            announce(inner, dice)?,
            text(single)?,
            op.sign(),
            value(inner, dice)?
        ),
    })
}

/// « inner = r op single = total ».
fn chained(
    inner: &Expr,
    op: Op,
    single: &Expr,
    total: u32,
    dice: &[u8; 3],
) -> Result<String, Illegal> {
    Ok(format!(
        "{} {} {} = {total}",
        announce(inner, dice)?,
        op.sign(),
        value(single, dice)?
    ))
}

/// Every pair calculation over dice `a` and `b`, in a fixed order.
fn pairs(a: u8, b: u8) -> impl Iterator<Item = Expr> {
    OPS.into_iter().flat_map(move |op| {
        [(a, b), (b, a)]
            .into_iter()
            .map(move |(l, r)| Expr::op(op, Expr::Die(l), Expr::Die(r)))
    })
}

/// Each value the dice `group` (indices, each used once) reach, with one
/// calculation for it, the first found in a fixed order: operations
/// + − × ÷, left before right.
pub fn values(dice: &[u8; 3], group: &[u8]) -> BTreeMap<u32, Expr> {
    let candidates: Vec<Expr> = match *group {
        [i] => vec![Expr::Die(i)],
        [i, j] => pairs(i, j).collect(),
        [i, j, k] => [(i, j, k), (i, k, j), (j, k, i)]
            .into_iter()
            .flat_map(|(a, b, c)| {
                pairs(a, b).flat_map(move |inner| {
                    OPS.into_iter().flat_map(move |op| {
                        [
                            Expr::op(op, inner.clone(), Expr::Die(c)),
                            Expr::op(op, Expr::Die(c), inner.clone()),
                        ]
                    })
                })
            })
            .collect(),
        _ => Vec::new(),
    };
    let mut found = BTreeMap::new();
    for expr in candidates {
        if let Ok(v) = eval(&expr, dice) {
            found.entry(v).or_insert(expr);
        }
    }
    found
}
