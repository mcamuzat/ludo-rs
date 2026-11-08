//! Enemy Chocolatier (James Ernest / Cheapass Games, 2006). Rules and their
//! sources: RULES.md.

#[rustfmt::skip]
mod board_data;
mod heuristics;
pub mod layout;
mod view;

pub use layout::{Placement, CITY_H, CITY_W, FACTORY_AT, SECTION_H, SECTION_W, SLOTS};
pub use view::{EndView, HoodStatus, HoodView, HouseView, Offer, RecipeView, SeatView, View};

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use layout::{is_side, neighbors, tables};

pub struct EnemyChocolatier;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

/// [R] Pink houses are Sugar, brown Chocolate, tan Nuts, red Love; gray
/// houses are no ingredient.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Ingredient {
    Chocolate,
    Sugar,
    Nuts,
    Love,
}

/// The order of the recipe cards: chocolate, sugar, nuts, love.
pub const INGREDIENTS: [Ingredient; 4] = [
    Ingredient::Chocolate,
    Ingredient::Sugar,
    Ingredient::Nuts,
    Ingredient::Love,
];

impl Ingredient {
    pub fn index(self) -> usize {
        self as usize
    }
}

/// [R] "Abilities".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    /// Buy up to 3 houses each turn.
    BuyThree,
    /// Buy houses in closed neighborhoods.
    BuyClosed,
    /// Buy houses in unavailable neighborhoods.
    BuyAnywhere,
    /// Each money-producing neighborhood gives one dollar more.
    AllDollars,
    /// Each point-producing neighborhood gives one point more.
    AllPoints,
    /// One point per $5 held, at the start of the turn.
    FiveDollarsPoint,
    /// Houses bought from this player cost $10.
    TenDollarHouses,
}

pub struct Recipe {
    pub name: &'static str,
    /// Chocolate, sugar, nuts, love.
    pub need: [u8; 4],
}

/// [P] The eight Secret Recipes, 19 ingredients each.
pub const RECIPES: [Recipe; 8] = [
    Recipe {
        name: "Alice Corwin's High-Fat Toffee Sludge",
        need: [3, 8, 6, 2],
    },
    Recipe {
        name: "Aznee's Long-Lost Fruit Shavings in Heavy Syrup",
        need: [7, 4, 5, 3],
    },
    Recipe {
        name: "Bairstowe's Almond-Crusted Sweet Nothings",
        need: [2, 3, 8, 6],
    },
    Recipe {
        name: "Flaxworth and Gorman's Wrinkly Sugar Sheets",
        need: [5, 7, 4, 3],
    },
    Recipe {
        name: "Headley's Incomparable Poly-Melon Balls",
        need: [3, 6, 2, 8],
    },
    Recipe {
        name: "Jolly John Henry's Bitter Chocolate Farm Animals",
        need: [8, 3, 3, 5],
    },
    Recipe {
        name: "Molly Bender's Housewarming Truffle Mix",
        need: [6, 2, 7, 4],
    },
    Recipe {
        name: "Sharpington's Worthwhile Sweet Icing Lumps",
        need: [4, 5, 3, 7],
    },
];

/// [R] "If you score 20 points, you win!"
pub const TARGET_POINTS: u32 = 20;
/// [R] Base income, printed on the Factory.
pub const BASE_INCOME: u32 = 2;
/// [R] Houses a player may buy each turn, Rain Checks apart.
pub const BUYS: u8 = 2;
/// ❓ Site limit: after this many rounds the game stops.
pub const ROUND_LIMIT: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Buy(u8),
    /// Ends the turn; without any purchase, earns a Rain Check.
    EndTurn,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all_fields = "camelCase")]
pub enum Event {
    Income {
        dollars: u32,
        points: u32,
    },
    Bought {
        house: u8,
        from: Option<Seat>,
        cost: u32,
        rain_check: bool,
    },
    Closed {
        hood: u8,
    },
    RainCheck,
}

/// The events of a finished turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastTurn {
    pub seat: Seat,
    pub events: Vec<Event>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    Points(Seat),
    Recipe(Seat),
    RoundLimit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub layout: [Placement; 6],
    /// Per hood: touches the Factory.
    pub near_factory: Vec<bool>,
    /// Per hood: the hoods it touches.
    pub touching: Vec<Vec<u8>>,
    /// Per house.
    pub owner: Vec<Option<Seat>>,
    pub money: Vec<u32>,
    pub points: Vec<u32>,
    pub rain_checks: Vec<u32>,
    /// Each seat's recipe card.
    pub recipes: Vec<u8>,
    /// The cards nobody got.
    pub spare: Vec<u8>,
    pub turn: Seat,
    /// Houses bought this turn.
    pub bought: u8,
    pub rounds: u32,
    /// Turns played, for the AI's horizon.
    pub turns: u32,
    pub events: Vec<Event>,
    pub last_turn: Option<LastTurn>,
    pub end: Option<End>,
}

pub fn hood_count() -> usize {
    tables().hoods.len()
}

pub fn house_count() -> usize {
    tables().house_hood.len()
}

pub fn hood_of(house: u8) -> u8 {
    tables().house_hood[usize::from(house)]
}

pub fn price(house: u8) -> u8 {
    let t = tables();
    let hood = &t.hoods[usize::from(hood_of(house))];
    hood.data.prices[usize::from(house - hood.first)]
}

pub fn houses_of(hood: u8) -> std::ops::Range<u8> {
    let h = &tables().hoods[usize::from(hood)];
    h.first..h.first + h.data.prices.len() as u8
}

pub fn color(hood: u8) -> Option<Ingredient> {
    tables().hoods[usize::from(hood)].data.color
}

impl State {
    /// [R] "Closed means that every house in the neighborhood is owned, and
    /// all by the same player."
    pub fn closed_by(&self, hood: u8) -> Option<Seat> {
        let mut houses = houses_of(hood);
        let first = self.owner[usize::from(houses.next()?)]?;
        houses
            .all(|h| self.owner[usize::from(h)] == Some(first))
            .then_some(first)
    }

    /// Who closed each hood.
    pub fn closures(&self) -> Vec<Option<Seat>> {
        (0..hood_count() as u8).map(|h| self.closed_by(h)).collect()
    }

    /// [R] A neighborhood touching the Factory or a closed neighborhood.
    fn connected(&self, hood: u8, closures: &[Option<Seat>]) -> bool {
        self.near_factory[usize::from(hood)]
            || self.touching[usize::from(hood)]
                .iter()
                .any(|&o| closures[usize::from(o)].is_some())
    }

    pub fn status(&self, hood: u8, closures: &[Option<Seat>]) -> HoodStatus {
        match closures[usize::from(hood)] {
            Some(by) => HoodStatus::Closed { by },
            None if self.connected(hood, closures) => HoodStatus::Open,
            None => HoodStatus::Unavailable,
        }
    }

    /// How many of `ability` the seat holds: one per neighborhood it closed.
    pub fn abilities(&self, seat: Seat, closures: &[Option<Seat>], ability: Ability) -> u32 {
        let t = tables();
        closures
            .iter()
            .zip(&t.hoods)
            .filter(|(c, h)| **c == Some(seat) && h.data.ability == Some(ability))
            .count() as u32
    }

    /// [R] "On Each Turn": the points then the dollars the seat would
    /// collect now.
    pub fn income(&self, seat: Seat, closures: &[Option<Seat>]) -> (u32, u32) {
        let t = tables();
        let all_dollars = self.abilities(seat, closures, Ability::AllDollars);
        let all_points = self.abilities(seat, closures, Ability::AllPoints);
        let five = self.abilities(seat, closures, Ability::FiveDollarsPoint);
        // [R] "$5 = 1 Point": counted on the money held before this turn's
        // income.
        let mut points = five * (self.money[seat.index()] / 5);
        let mut dollars = BASE_INCOME;
        for (c, h) in closures.iter().zip(&t.hoods) {
            if *c != Some(seat) {
                continue;
            }
            if h.data.points > 0 {
                points += u32::from(h.data.points) + all_points;
            }
            if h.data.dollars > 0 {
                dollars += u32::from(h.data.dollars) + all_dollars;
            }
        }
        (dollars, points)
    }

    /// Ingredients owned, chocolate, sugar, nuts, love.
    pub fn ingredients(&self, seat: Seat) -> [u8; 4] {
        let mut have = [0u8; 4];
        for (house, owner) in self.owner.iter().enumerate() {
            if *owner == Some(seat) {
                if let Some(c) = color(hood_of(house as u8)) {
                    have[c.index()] += 1;
                }
            }
        }
        have
    }

    pub fn recipe_done(&self, seat: Seat) -> bool {
        let need = RECIPES[usize::from(self.recipes[seat.index()])].need;
        self.ingredients(seat)
            .iter()
            .zip(need)
            .all(|(&have, need)| have >= need)
    }

    pub fn buy_limit(&self, seat: Seat, closures: &[Option<Seat>]) -> u8 {
        if self.abilities(seat, closures, Ability::BuyThree) > 0 {
            3
        } else {
            BUYS
        }
    }

    /// What the house costs `buyer`, and how much goes to its owner.
    pub fn cost(&self, house: u8, closures: &[Option<Seat>]) -> (u32, u32) {
        match self.owner[usize::from(house)] {
            None => (u32::from(price(house)), 0),
            Some(owner) => {
                if self.abilities(owner, closures, Ability::TenDollarHouses) > 0 {
                    (10, 6)
                } else {
                    (6, 4)
                }
            }
        }
    }

    /// Why `seat` cannot buy `house` now, if it cannot.
    fn refusal(&self, seat: Seat, house: u8, closures: &[Option<Seat>]) -> Option<&'static str> {
        if usize::from(house) >= self.owner.len() {
            return Some("Cette maison n'existe pas.");
        }
        if self.owner[usize::from(house)] == Some(seat) {
            return Some("Cette maison est déjà à toi.");
        }
        let limit = self.buy_limit(seat, closures);
        if u32::from(self.bought) >= u32::from(limit) + self.rain_checks[seat.index()] {
            return Some("Plus d'achat possible ce tour : il faudrait un bon.");
        }
        let hood = hood_of(house);
        let closed = closures[usize::from(hood)].is_some();
        if closed && self.abilities(seat, closures, Ability::BuyClosed) == 0 {
            return Some("Ce quartier est fermé.");
        }
        if !self.connected(hood, closures)
            && self.abilities(seat, closures, Ability::BuyAnywhere) == 0
        {
            return Some("Ce quartier ne touche ni l'usine ni un quartier fermé.");
        }
        if self.cost(house, closures).0 > self.money[seat.index()] {
            return Some("Pas assez d'argent.");
        }
        None
    }

    pub fn moves(&self, seat: Seat) -> Vec<Action> {
        if self.end.is_some() || seat != self.turn {
            return Vec::new();
        }
        let closures = self.closures();
        let mut v: Vec<Action> = (0..self.owner.len() as u8)
            .filter(|&h| self.refusal(seat, h, &closures).is_none())
            .map(Action::Buy)
            .collect();
        v.push(Action::EndTurn);
        v
    }

    fn buy(&mut self, seat: Seat, house: u8) {
        let closures = self.closures();
        let (cost, to_owner) = self.cost(house, &closures);
        let rain_check = self.bought >= self.buy_limit(seat, &closures);
        if rain_check {
            self.rain_checks[seat.index()] -= 1;
        }
        let from = self.owner[usize::from(house)];
        self.money[seat.index()] -= cost;
        if let Some(o) = from {
            self.money[o.index()] += to_owner;
        }
        self.owner[usize::from(house)] = Some(seat);
        self.bought += 1;
        self.events.push(Event::Bought {
            house,
            from,
            cost,
            rain_check,
        });
        let hood = hood_of(house);
        if self.closed_by(hood) == Some(seat) {
            self.events.push(Event::Closed { hood });
        }
        if self.recipe_done(seat) {
            self.finish(End::Recipe(seat));
        }
    }

    fn end_turn(&mut self, seat: Seat) {
        if self.bought == 0 {
            // [R] "If you take a turn without buying anything, you earn a
            // Rain Check."
            self.rain_checks[seat.index()] += 1;
            self.events.push(Event::RainCheck);
        }
        self.close_turn();
        self.turns += 1;
        let next = Seat((seat.0 + 1) % self.players);
        if next.0 == 0 {
            self.rounds += 1;
            if self.rounds >= ROUND_LIMIT {
                self.end = Some(End::RoundLimit);
                return;
            }
        }
        self.start_turn(next);
    }

    fn close_turn(&mut self) {
        self.last_turn = Some(LastTurn {
            seat: self.turn,
            events: std::mem::take(&mut self.events),
        });
    }

    /// [R] First collect your income: points, then dollars.
    fn start_turn(&mut self, seat: Seat) {
        self.turn = seat;
        self.bought = 0;
        let closures = self.closures();
        let (dollars, points) = self.income(seat, &closures);
        self.points[seat.index()] += points;
        self.money[seat.index()] += dollars;
        self.events.push(Event::Income { dollars, points });
        if self.points[seat.index()] >= TARGET_POINTS {
            self.finish(End::Points(seat));
        }
    }

    fn finish(&mut self, end: End) {
        self.end = Some(end);
        self.close_turn();
    }

    /// The winner first, everyone else tied; at the round limit, by points
    /// then money.
    pub fn ranks(&self) -> Vec<u8> {
        let n = usize::from(self.players);
        match self.end {
            Some(End::Points(w) | End::Recipe(w)) => {
                (0..n).map(|i| u8::from(i != w.index())).collect()
            }
            _ => {
                let key = |i: usize| (self.points[i], self.money[i]);
                (0..n)
                    .map(|i| (0..n).filter(|&j| key(j) > key(i)).count() as u8)
                    .collect()
            }
        }
    }
}

impl Game for EnemyChocolatier {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "enemy-chocolatier",
            name: "Enemy Chocolatier",
            min_players: 2,
            max_players: 6,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        // ChaCha, not SmallRng: a browser log must replay natively.
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        // [R] "Place the other 6 pieces randomly."
        let mut sections: Vec<u8> = (0..6).collect();
        sections.shuffle(&mut rng);
        let layout: [Placement; 6] = std::array::from_fn(|slot| {
            let turned: bool = rng.gen();
            Placement {
                section: sections[slot],
                quarters: match (is_side(slot), turned) {
                    (true, false) => 1,
                    (true, true) => 3,
                    (false, false) => 0,
                    (false, true) => 2,
                },
            }
        });
        let (near_factory, touching) = neighbors(&layout);
        // [R] "Shuffle the Secret Recipe cards and give one to each player."
        let mut cards: Vec<u8> = (0..RECIPES.len() as u8).collect();
        cards.shuffle(&mut rng);
        let n = usize::from(players);
        let spare = cards.split_off(n);
        let mut s = State {
            players,
            layout,
            near_factory,
            touching,
            owner: vec![None; house_count()],
            // [R] The first player begins with $4, the next with $5, and so on.
            // ❓ Seat 0 goes first.
            money: (0..players).map(|i| 4 + u32::from(i)).collect(),
            points: vec![0; n],
            rain_checks: vec![0; n],
            recipes: cards,
            spare,
            turn: Seat(0),
            bought: 0,
            rounds: 0,
            turns: 0,
            events: Vec::new(),
            last_turn: None,
            end: None,
        };
        s.start_turn(Seat(0));
        s
    }

    fn pending(s: &State) -> Pending {
        if s.end.is_some() {
            Pending::none()
        } else {
            Pending::one(s.turn)
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat)
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.end.is_some() {
            return Err(Illegal::new("La partie est terminée."));
        }
        if seat != s.turn {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match *action {
            Action::Buy(house) => {
                if let Some(why) = s.refusal(seat, house, &s.closures()) {
                    return Err(Illegal::new(why));
                }
                s.buy(seat, house);
            }
            Action::EndTurn => s.end_turn(seat),
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.end.map(|_| Outcome::Ranked(s.ranks()))
    }
}

impl Determinize for EnemyChocolatier {
    /// Redeals the recipes the observer has not seen: the others' and the
    /// spare ones. Nothing is hidden once the game is over.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        if s.end.is_some() {
            return d;
        }
        let mut pool = d.spare.clone();
        let others: Vec<usize> = (0..usize::from(s.players))
            .filter(|&i| i != observer.index())
            .collect();
        pool.extend(others.iter().map(|&i| d.recipes[i]));
        pool.shuffle(rng);
        for &i in &others {
            d.recipes[i] = pool.pop().expect("one card per seat");
        }
        d.spare = pool;
        d
    }
}
