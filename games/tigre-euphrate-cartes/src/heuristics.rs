//! The search looks at a short list of sensible actions, ranked by an
//! estimate of what they bring at once; simulations play the same ranking
//! with some noise and stop after two rounds of the table.

use ludo_core::{Heuristics, Seat};
use rand::Rng;

use crate::{
    sorted_points, Action, Color, Face, Phase, Pos, Span, State, TigreEuphrateCartes, COLORS,
    COLUMNS, GAPS,
};

/// How many actions Hard searches in the action phase (end of turn apart).
const SEARCH_WIDTH: usize = 12;
/// Rounds a simulation plays before its standing is estimated.
const HORIZON: u32 = 2;
/// Points of the weakest colour worth about 73 % against one opponent.
const SCALE: f64 = 1.5;

fn mine(s: &State, seat: Seat, color: Color) -> u8 {
    s.hands[seat.index()][color.index()]
}

/// One more point of `color` is worth more where `seat` is weak.
fn point(s: &State, seat: Seat, color: Color) -> f64 {
    let sc = &s.scores[seat.index()];
    let low = *sc.iter().min().expect("four colours");
    if sc[color.index()] == low {
        2.0
    } else if sc[color.index()] <= low + 1 {
        1.4
    } else {
        1.0
    }
}

/// The best free card of a kingdom for a leader of `color`: [R] a king on
/// a red card wins internal conflicts; a leader on a card of its colour
/// keeps that card from being taken.
fn spot(s: &State, span: Span, color: Color) -> Option<Pos> {
    let free: Vec<Pos> = s
        .table
        .positions(span)
        .into_iter()
        .filter(|&p| s.leader_may_stand(p))
        .collect();
    let wanted = if color == Color::Black {
        Color::Red
    } else {
        color
    };
    free.iter()
        .copied()
        .find(|&p| s.table.slot(p).and_then(|x| x.face.color()) == Some(wanted))
        .or_else(|| free.first().copied())
}

/// The kingdoms of the table.
fn kingdoms(s: &State) -> Vec<Span> {
    let mut out = Vec::new();
    let mut c = 0u8;
    while usize::from(c) < COLUMNS {
        let span = s.table.span(c, None);
        out.push(span);
        c = span.1 + 1;
    }
    out
}

/// Sensible actions of the action phase with an estimate of their worth:
/// one spot per leader and kingdom, every card, catastrophes on junctions.
fn candidates(s: &State, seat: Seat) -> Vec<(Action, f64)> {
    let mut out = Vec::new();
    for span in kingdoms(s) {
        let leaders = s.table.leaders_in(span);
        for color in COLORS {
            let from = s.table.find_leader(seat, color);
            if from.is_some_and(|p| s.table.span(p.column(), None) == span) {
                continue;
            }
            let Some(to) = spot(s, span, color) else {
                continue;
            };
            let mut v = if from.is_none() { 1.5 } else { 0.3 };
            v += 0.15 * f64::from(s.table.count(span, color));
            if leaders.iter().any(|(_, l)| l.color == color) {
                let mine = f64::from(mine(s, seat, Color::Red))
                    + f64::from(color == Color::Black || s.king_bonus(seat, span) > 0);
                v += if mine >= 3.0 { 0.5 } else { -2.5 };
            }
            out.push((Action::Leader { color, to }, v));
        }
    }
    for color in COLORS.into_iter().filter(|&c| mine(s, seat, c) > 0) {
        for column in 0..COLUMNS as u8 {
            if !s.may_extend(column) {
                continue;
            }
            let span = s.table.span(column, None);
            let mut v = match s.table.scorer(span, color) {
                Some(o) if o == seat => {
                    if mine(s, seat, color) >= 2 {
                        2.0 * point(s, seat, color)
                    } else {
                        0.2
                    }
                }
                Some(_) => -1.0,
                None => 0.0,
            };
            let cards = &s.table.columns[usize::from(column)];
            if cards.len() >= 3
                && cards[cards.len() - 3..]
                    .iter()
                    .all(|x| x.face == Face::Civ(color))
                && (color == Color::Blue || s.ships.contains(&color))
                && !s.ships.is_empty()
            {
                v += 1.0;
            }
            out.push((Action::Extend { color, column }, v));
        }
        for gap in 0..GAPS as u8 {
            if s.may_join(gap) {
                out.push((Action::Join { color, gap }, join_value(s, seat, gap, color)));
            }
        }
    }
    if s.catastrophes[seat.index()] {
        for gap in 0..GAPS as u8 {
            let at = Pos::Gap(gap);
            if s.may_destroy(at) {
                out.push((Action::Catastrophe { at }, -1.0));
            }
        }
    }
    out.push((Action::EndTurn, -0.8));
    out
}

/// A junction: the conflicts `seat` would likely win or lose, and the
/// treasure its merchant would take.
fn join_value(s: &State, seat: Seat, gap: u8, color: Color) -> f64 {
    let left = s.table.span(gap, None);
    let right = s.table.span(gap + 1, None);
    let merged = (left.0, right.1);
    let leaders = s.table.leaders_in(merged);
    let mut v = 0.3;
    for c in COLORS {
        let both: Vec<_> = leaders.iter().filter(|(_, l)| l.color == c).collect();
        if both.len() < 2 {
            continue;
        }
        let Some((p, _)) = both.iter().find(|(_, l)| l.seat == seat) else {
            continue;
        };
        let (q, _) = both.iter().find(|(_, l)| l.seat != seat).expect("two");
        let side = |x: &Pos| {
            if x.column() <= gap {
                left
            } else {
                right
            }
        };
        let spare = mine(s, seat, c).saturating_sub(u8::from(c == color));
        let strength = i32::from(s.table.count(side(p), c)) + i32::from(spare);
        let theirs = i32::from(s.table.count(side(q), c));
        v += if strength > theirs + 1 {
            1.0 + 0.3 * f64::from(s.table.count(side(q), c))
        } else {
            -1.5
        };
    }
    let merchant = leaders
        .iter()
        .find(|(_, l)| l.color == Color::Green)
        .map(|(_, l)| l.seat);
    if merchant == Some(seat)
        && s.table.treasures(merged).len() >= 2
        && mine(s, seat, Color::Red) > u8::from(color == Color::Red)
    {
        v += 2.5;
    }
    v
}

fn ranked(s: &State, seat: Seat) -> Vec<(Action, f64)> {
    let mut c = candidates(s, seat);
    c.sort_by(|a, b| b.1.total_cmp(&a.1));
    c
}

/// Cards to commit: enough to win if the hand allows, otherwise none.
fn commit<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
    let Phase::Conflict(c) = &s.phase else {
        unreachable!("only in a conflict")
    };
    let have = mine(s, seat, c.support());
    let count = match c.attacker_added {
        None => {
            let need = (i32::from(c.defender_base) - i32::from(c.attacker_base) + 1).max(0);
            let margin = (rng.gen::<u32>() % 3) as i32;
            if need <= i32::from(have) {
                (need + margin).min(i32::from(have))
            } else {
                0
            }
        }
        Some(added) => {
            let need =
                (i32::from(c.attacker_base) + i32::from(added) - i32::from(c.defender_base)).max(0);
            if need <= i32::from(have) {
                need
            } else {
                0
            }
        }
    };
    Action::Commit { count: count as u8 }
}

impl Heuristics for TigreEuphrateCartes {
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        match &s.phase {
            Phase::Action => {
                let moves = ranked(s, seat);
                let top = if rng.gen::<u32>() % 10 < 7 {
                    moves.len().min(3)
                } else {
                    moves.len().min(10)
                };
                moves[(rng.gen::<u32>() % top as u32) as usize].0
            }
            Phase::Conflict(_) => commit(s, seat, rng),
            Phase::ChooseConflict(colors) => Action::Resolve {
                color: colors[(rng.gen::<u32>() % colors.len() as u32) as usize],
            },
            Phase::Ship { options, .. } => Action::Ship { ship: options[0] },
            Phase::Over => unreachable!("nobody decides"),
        }
    }

    /// After two rounds, each seat's share of opponents it leads on the
    /// weakest colour, softened by the gap.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        let n = usize::from(sim.players);
        if sim.phase == Phase::Over || sim.turns < root.turns + HORIZON * u32::from(sim.players) {
            return None;
        }
        let v: Vec<f64> = (0..n)
            .map(|i| {
                let p = sorted_points(&sim.scores[i], sim.treasures[i]);
                f64::from(p[0]) + 0.3 * f64::from(p[1])
            })
            .collect();
        Some(
            (0..n)
                .map(|i| {
                    let sum: f64 = (0..n)
                        .filter(|&j| j != i)
                        .map(|j| 1.0 / (1.0 + (-(v[i] - v[j]) / SCALE).exp()))
                        .sum();
                    sum / (n - 1) as f64
                })
                .collect(),
        )
    }

    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        match &s.phase {
            Phase::Action => {
                let mut out: Vec<Action> = ranked(s, seat)
                    .into_iter()
                    .map(|(a, _)| a)
                    .filter(|a| *a != Action::EndTurn)
                    .take(SEARCH_WIDTH)
                    .collect();
                out.push(Action::EndTurn);
                out
            }
            _ => s.moves(seat),
        }
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::{Config, Phase, TigreEuphrateCartes};

    #[test]
    fn heuristics_only_pick_legal_actions() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        for players in 2..=4u8 {
            for seed in 0..6 {
                let mut s = TigreEuphrateCartes::new(&Config::default(), players, seed);
                while let Some(seat) = s.deciding() {
                    let legal = TigreEuphrateCartes::legal_actions(&s, seat);
                    let search = TigreEuphrateCartes::search_actions(&s, seat);
                    assert!(!search.is_empty());
                    assert!(search.iter().all(|a| legal.contains(a)), "{search:?}");
                    let a = TigreEuphrateCartes::rollout_action(&s, seat, &mut rng);
                    assert!(legal.contains(&a), "{a:?}");
                    TigreEuphrateCartes::apply(&mut s, seat, &a).unwrap();
                }
                assert_eq!(s.phase, Phase::Over);
                assert!(s.scores.iter().any(|sc| sc.iter().sum::<u8>() > 0));
            }
        }
    }

    #[test]
    fn evaluate_cuts_after_two_rounds() {
        let root = TigreEuphrateCartes::new(&Config::default(), 3, 4);
        assert!(TigreEuphrateCartes::evaluate(&root, &root).is_none());
        let mut sim = root.clone();
        sim.turns += 6;
        sim.scores = vec![[5, 5, 5, 5], [1, 1, 1, 1], [1, 1, 1, 1]];
        let v = TigreEuphrateCartes::evaluate(&root, &sim).expect("two rounds played");
        assert!(v[0] > 0.9 && v[1] < 0.5);
    }
}
