//! The AI: generic ISMCTS over a dozen sensible purchases, simulations
//! played by a greedy policy and cut after a few rounds.

use ludo_core::{Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::layout::tables;
use crate::{
    color, hood_of, houses_of, Ability, Action, EnemyChocolatier, State, RECIPES, TARGET_POINTS,
};

/// Purchases the search considers.
const WIDTH: usize = 12;
/// Turns simulated per player before the estimate replaces the rest.
const HORIZON: u32 = 3;

/// What closing `hood` is worth to `seat`, in rough dollars.
fn hood_value(s: &State, seat: Seat, hood: u8, closures: &[Option<Seat>]) -> f32 {
    let data = tables().hoods[usize::from(hood)].data;
    let all_dollars = s.abilities(seat, closures, Ability::AllDollars) as f32;
    let all_points = s.abilities(seat, closures, Ability::AllPoints) as f32;
    let mut v = 1.0;
    if data.dollars > 0 {
        v += 2.5 * (f32::from(data.dollars) + all_dollars);
    }
    if data.points > 0 {
        v += 4.0 * (f32::from(data.points) + all_points);
    }
    v += match data.ability {
        None => 0.0,
        Some(Ability::BuyThree) => 3.0,
        Some(Ability::BuyClosed | Ability::BuyAnywhere) => 1.0,
        Some(Ability::TenDollarHouses) => 1.5,
        Some(Ability::FiveDollarsPoint) => 3.0 + (s.money[seat.index()] / 5) as f32,
        Some(Ability::AllDollars) => {
            1.0 + 2.5 * count_closed(seat, closures, |d| d.dollars > 0) as f32
        }
        Some(Ability::AllPoints) => {
            1.0 + 4.0 * count_closed(seat, closures, |d| d.points > 0) as f32
        }
    };
    v
}

fn count_closed(
    seat: Seat,
    closures: &[Option<Seat>],
    keep: impl Fn(&crate::board_data::HoodData) -> bool,
) -> usize {
    tables()
        .hoods
        .iter()
        .zip(closures)
        .filter(|(h, c)| **c == Some(seat) && keep(h.data))
        .count()
}

/// How much the seat still lacks of each ingredient.
fn missing(s: &State, seat: Seat) -> [u8; 4] {
    let need = RECIPES[usize::from(s.recipes[seat.index()])].need;
    let have = s.ingredients(seat);
    std::array::from_fn(|i| need[i].saturating_sub(have[i]))
}

/// The legal moves of `seat`, best first, scored on what it can see.
pub(crate) fn ranked(s: &State, seat: Seat) -> Vec<(f32, Action)> {
    let closures = s.closures();
    let lack = missing(s, seat);
    let lack_total: u8 = lack.iter().sum();
    let limit = s.buy_limit(seat, &closures);
    let mut v: Vec<(f32, Action)> = s
        .moves(seat)
        .into_iter()
        .map(|a| match a {
            Action::EndTurn => (if s.bought == 0 { 1.0 } else { 0.0 }, a),
            Action::Buy(house) => (
                buy_value(s, seat, house, &closures, &lack, lack_total, limit),
                a,
            ),
        })
        .collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    v
}

fn buy_value(
    s: &State,
    seat: Seat,
    house: u8,
    closures: &[Option<Seat>],
    lack: &[u8; 4],
    lack_total: u8,
    limit: u8,
) -> f32 {
    let hood = hood_of(house);
    let houses = houses_of(hood);
    let total = houses.len() as f32;
    let mine = houses
        .clone()
        .filter(|&h| s.owner[usize::from(h)] == Some(seat))
        .count() as f32
        + 1.0;
    let rivals = houses
        .clone()
        .filter(|&h| matches!(s.owner[usize::from(h)], Some(o) if o != seat))
        .count() as f32;
    let value = hood_value(s, seat, hood, closures);
    let mut v = if mine >= total {
        value
    } else {
        // Houses still to take from rivals cost $6 each.
        0.6 * value * (mine / total).powi(2) / (1.0 + rivals)
    };
    if let Some(c) = color(hood) {
        if lack[c.index()] > 0 {
            // Recipes are worth chasing once they are within reach.
            v += 1.0 + 6.0 / (1.0 + f32::from(lack_total));
        }
    }
    if let Some(owner) = s.owner[usize::from(house)] {
        if closures[usize::from(hood)] == Some(owner) {
            v += 0.8 * hood_value(s, owner, hood, closures);
        }
    }
    let (cost, _) = s.cost(house, closures);
    v -= 0.6 * cost as f32;
    if s.bought >= limit {
        v -= 1.5;
    }
    v
}

/// Turns the seat still needs to win, roughly: by points or by recipe.
fn turns_to_win(s: &State, seat: usize, closures: &[Option<Seat>]) -> f64 {
    let (dollars, points) = s.income(Seat(seat as u8), closures);
    let left = f64::from(TARGET_POINTS.saturating_sub(s.points[seat]));
    let by_points = left / f64::from(points).max(0.25);
    let lack: u8 = missing(s, Seat(seat as u8)).iter().sum();
    let pace = (f64::from(dollars) / 2.5).clamp(0.6, 3.0);
    let by_recipe = f64::from(lack) / pace + 1.0;
    by_points.min(by_recipe) - 0.03 * f64::from(s.money[seat])
}

impl Heuristics for EnemyChocolatier {
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let ranked = ranked(s, seat);
        if rng.gen_bool(0.8) {
            ranked[0].1
        } else {
            ranked[..ranked.len().min(4)]
                .choose(rng)
                .expect("a pending seat has a move")
                .1
        }
    }

    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let mut v: Vec<Action> = ranked(s, seat)
            .into_iter()
            .take(WIDTH)
            .map(|(_, a)| a)
            .collect();
        if !v.contains(&Action::EndTurn) {
            v.push(Action::EndTurn);
        }
        v
    }

    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        if sim.turns < root.turns + HORIZON * u32::from(root.players) || sim.bought > 0 {
            return None;
        }
        let closures = sim.closures();
        let n = usize::from(sim.players);
        let t: Vec<f64> = (0..n).map(|i| turns_to_win(sim, i, &closures)).collect();
        Some(
            (0..n)
                .map(|i| {
                    let sum: f64 = (0..n)
                        .filter(|&j| j != i)
                        .map(|j| 1.0 / (1.0 + ((t[i] - t[j]) / 2.0).exp()))
                        .sum();
                    sum / (n - 1) as f64
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use ludo_core::Game;

    #[test]
    fn the_search_keeps_end_turn_and_a_few_purchases() {
        let s = EnemyChocolatier::new(&Config::default(), 3, 7);
        let v = EnemyChocolatier::search_actions(&s, Seat(0));
        assert!(v.contains(&Action::EndTurn));
        assert!(v.len() > 1 && v.len() <= WIDTH + 1);
    }
}
