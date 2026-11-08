use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::layout::{tables, CITY_H, CITY_W, FACTORY_AT, SLOTS};
use crate::{
    hood_of, price, Ability, Action, End, Event, Ingredient, LastTurn, Placement, State, RECIPES,
    ROUND_LIMIT, TARGET_POINTS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all_fields = "camelCase")]
pub enum HoodStatus {
    /// [R] Open for purchase.
    Open,
    /// [R] Every house owned by one player.
    Closed { by: Seat },
    /// [R] Touches neither the Factory nor a closed neighborhood.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoodView {
    /// Where to find its shape in `board.json`.
    pub section: u8,
    pub index: u8,
    pub color: Option<Ingredient>,
    pub dollars: u8,
    pub points: u8,
    pub ability: Option<Ability>,
    pub status: HoodStatus,
    pub houses: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseView {
    pub hood: u8,
    pub price: u8,
    pub owner: Option<Seat>,
}

/// A house the viewer may buy now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    pub house: u8,
    pub cost: u32,
    /// Bought past the turn's limit: costs a Rain Check.
    pub rain_check: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeView {
    pub name: &'static str,
    /// Chocolate, sugar, nuts, love.
    pub need: [u8; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub money: u32,
    pub points: u32,
    pub rain_checks: u32,
    /// What the seat would collect if its turn began now.
    pub income_dollars: u32,
    pub income_points: u32,
    pub abilities: Vec<Ability>,
    /// Chocolate, sugar, nuts, love.
    pub ingredients: [u8; 4],
    /// Its own recipe, everyone's once the game is over.
    pub recipe: Option<RecipeView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndView {
    pub end: End,
    pub ranks: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    pub players: u8,
    /// The city's size and where each slot and the Factory lie, in pixels
    /// at 100 dpi (`board.json`'s unit).
    pub width: u32,
    pub height: u32,
    pub slots: Vec<(u32, u32)>,
    pub factory_at: (u32, u32),
    pub layout: Vec<Placement>,
    pub hoods: Vec<HoodView>,
    pub houses: Vec<HouseView>,
    pub seats: Vec<SeatView>,
    pub turn: Seat,
    /// Houses bought this turn and the turn's limit (Rain Checks apart).
    pub bought: u8,
    pub limit: u8,
    /// What the viewer may buy, when it is its turn.
    pub offers: Vec<Offer>,
    pub events: Vec<Event>,
    pub last_turn: Option<LastTurn>,
    pub target_points: u32,
    pub rounds_left: u32,
    pub end: Option<EndView>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.0 < s.players => Some(seat),
            _ => None,
        };
        let t = tables();
        let closures = s.closures();
        let hoods = t
            .hoods
            .iter()
            .enumerate()
            .map(|(id, h)| HoodView {
                section: h.section,
                index: h.index,
                color: h.data.color,
                dollars: h.data.dollars,
                points: h.data.points,
                ability: h.data.ability,
                status: s.status(id as u8, &closures),
                houses: crate::houses_of(id as u8).collect(),
            })
            .collect();
        let houses = s
            .owner
            .iter()
            .enumerate()
            .map(|(house, &owner)| HouseView {
                hood: hood_of(house as u8),
                price: price(house as u8),
                owner,
            })
            .collect();
        let over = s.end.is_some();
        let seats = (0..s.players)
            .map(|i| {
                let seat = Seat(i);
                let (income_dollars, income_points) = s.income(seat, &closures);
                let mut abilities: Vec<Ability> = t
                    .hoods
                    .iter()
                    .zip(&closures)
                    .filter(|(_, c)| **c == Some(seat))
                    .filter_map(|(h, _)| h.data.ability)
                    .collect();
                abilities.sort_by_key(|a| *a as u8);
                let recipe = (over || me == Some(seat)).then(|| {
                    let r = &RECIPES[usize::from(s.recipes[seat.index()])];
                    RecipeView {
                        name: r.name,
                        need: r.need,
                    }
                });
                SeatView {
                    money: s.money[seat.index()],
                    points: s.points[seat.index()],
                    rain_checks: s.rain_checks[seat.index()],
                    income_dollars,
                    income_points,
                    abilities,
                    ingredients: s.ingredients(seat),
                    recipe,
                }
            })
            .collect();
        let offers = match me {
            Some(seat) if !over && seat == s.turn => {
                let limit = s.buy_limit(seat, &closures);
                s.moves(seat)
                    .into_iter()
                    .filter_map(|a| match a {
                        Action::Buy(house) => Some(Offer {
                            house,
                            cost: s.cost(house, &closures).0,
                            rain_check: s.bought >= limit,
                        }),
                        Action::EndTurn => None,
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        Self {
            me,
            players: s.players,
            width: CITY_W,
            height: CITY_H,
            slots: SLOTS.to_vec(),
            factory_at: FACTORY_AT,
            layout: s.layout.to_vec(),
            hoods,
            houses,
            seats,
            turn: s.turn,
            bought: s.bought,
            limit: s.buy_limit(s.turn, &closures),
            offers,
            events: s.events.clone(),
            last_turn: s.last_turn.clone(),
            target_points: TARGET_POINTS,
            rounds_left: ROUND_LIMIT.saturating_sub(s.rounds),
            end: s.end.map(|end| EndView {
                end,
                ranks: s.ranks(),
            }),
        }
    }
}
