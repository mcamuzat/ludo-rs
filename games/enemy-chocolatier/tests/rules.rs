use enemy_chocolatier::layout::tables;
use enemy_chocolatier::{
    hood_count, houses_of, Ability, Action, Config, End, EnemyChocolatier, HoodStatus, Ingredient,
    State, View, RECIPES,
};
use ludo_core::{Determinize, Game, Seat, Viewer};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn new(players: u8, seed: u64) -> State {
    EnemyChocolatier::new(&Config::default(), players, seed)
}

fn hood_with(ability: Ability) -> u8 {
    (0..hood_count() as u8)
        .find(|&h| tables().hoods[usize::from(h)].data.ability == Some(ability))
        .expect("the boards hold every ability")
}

fn give(s: &mut State, hood: u8, seat: Seat) {
    for h in houses_of(hood) {
        s.owner[usize::from(h)] = Some(seat);
    }
}

fn view(s: &State, seat: u8) -> View {
    EnemyChocolatier::view(s, Viewer::Seat(Seat(seat)))
}

/// A hood the seat may reach (touches the Factory), unowned.
fn open_hood(s: &State) -> u8 {
    (0..hood_count() as u8)
        .find(|&h| s.near_factory[usize::from(h)])
        .expect("some hoods touch the Factory")
}

#[test]
fn money_starts_at_four_and_grows_by_seat_and_the_first_income_is_paid() {
    let s = new(6, 1);
    // Seat 0 already collected its $2.
    assert_eq!(s.money, vec![6, 5, 6, 7, 8, 9]);
    assert_eq!(s.points, vec![0; 6]);
}

#[test]
fn at_first_only_the_hoods_touching_the_factory_are_open() {
    let s = new(3, 2);
    let v = view(&s, 0);
    assert!(!v.offers.is_empty());
    for offer in &v.offers {
        let hood = v.houses[usize::from(offer.house)].hood;
        assert!(s.near_factory[usize::from(hood)]);
        assert!(matches!(
            v.hoods[usize::from(hood)].status,
            HoodStatus::Open
        ));
    }
}

#[test]
fn closing_a_hood_opens_the_ones_it_touches() {
    let mut s = new(2, 3);
    let closed = open_hood(&s);
    let far = s.touching[usize::from(closed)]
        .iter()
        .copied()
        .find(|&o| !s.near_factory[usize::from(o)])
        .expect("a hood beyond");
    let before = view(&s, 0);
    assert!(matches!(
        before.hoods[usize::from(far)].status,
        HoodStatus::Unavailable
    ));
    give(&mut s, closed, Seat(1));
    let after = view(&s, 0);
    assert!(matches!(
        after.hoods[usize::from(closed)].status,
        HoodStatus::Closed { by: Seat(1) }
    ));
    assert!(matches!(
        after.hoods[usize::from(far)].status,
        HoodStatus::Open
    ));
    let house = houses_of(far).next().unwrap();
    s.money[0] = 50;
    assert!(EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).is_ok());
}

#[test]
fn a_house_of_another_costs_six_split_between_bank_and_owner() {
    let mut s = new(2, 4);
    let hood = open_hood(&s);
    let house = houses_of(hood).next().unwrap();
    s.owner[usize::from(house)] = Some(Seat(1));
    s.money = vec![20, 0];
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).unwrap();
    assert_eq!(s.money, vec![14, 4]);
    assert_eq!(s.owner[usize::from(house)], Some(Seat(0)));
}

#[test]
fn ten_dollar_houses_cost_ten_and_pay_the_owner_six() {
    let mut s = new(2, 5);
    give(&mut s, hood_with(Ability::TenDollarHouses), Seat(1));
    let hood = open_hood(&s);
    let house = houses_of(hood).next().unwrap();
    s.owner[usize::from(house)] = Some(Seat(1));
    s.money = vec![20, 0];
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).unwrap();
    assert_eq!(s.money, vec![10, 6]);
}

#[test]
fn closed_hoods_need_buy_closed() {
    let mut s = new(2, 6);
    let hood = open_hood(&s);
    give(&mut s, hood, Seat(1));
    s.money[0] = 50;
    let house = houses_of(hood).next().unwrap();
    let before = s.clone();
    assert!(EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).is_err());
    assert_eq!(s, before, "a refused action changes nothing");
    let bc = hood_with(Ability::BuyClosed);
    if bc != hood {
        give(&mut s, bc, Seat(0));
        assert!(EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).is_ok());
        assert_ne!(
            s.closed_by(hood),
            Some(Seat(1)),
            "the purchase opens it again"
        );
    }
}

#[test]
fn buy_anywhere_reaches_unavailable_hoods() {
    let mut s = new(2, 7);
    s.money[0] = 50;
    let far = (0..hood_count() as u8)
        .find(|&h| {
            !s.near_factory[usize::from(h)]
                && tables().hoods[usize::from(h)].data.ability != Some(Ability::BuyAnywhere)
        })
        .unwrap();
    let house = houses_of(far).next().unwrap();
    assert!(EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).is_err());
    give(&mut s, hood_with(Ability::BuyAnywhere), Seat(0));
    // The Buy Anywhere hood may itself open `far`: either way it is legal now.
    assert!(EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).is_ok());
}

/// The cheapest unowned houses the seat may buy, `n` of them.
fn cheap_offers(s: &State, n: usize) -> Vec<u8> {
    let mut offers = view(s, s.turn.0).offers;
    offers.sort_by_key(|o| o.cost);
    offers.into_iter().take(n).map(|o| o.house).collect()
}

#[test]
fn two_buys_a_turn_then_rain_checks() {
    let mut s = new(2, 8);
    s.money[0] = 100;
    let houses = cheap_offers(&s, 3);
    assert_eq!(houses.len(), 3);
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(houses[0])).unwrap();
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(houses[1])).unwrap();
    assert!(view(&s, 0).offers.is_empty());
    assert!(EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(houses[2])).is_err());
    s.rain_checks[0] = 1;
    assert!(view(&s, 0).offers.iter().all(|o| o.rain_check));
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(houses[2])).unwrap();
    assert_eq!(s.rain_checks[0], 0);
}

#[test]
fn a_turn_without_purchase_earns_a_rain_check() {
    let mut s = new(2, 9);
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::EndTurn).unwrap();
    assert_eq!(s.rain_checks, vec![1, 0]);
    assert_eq!(s.turn, Seat(1));
    let house = cheap_offers(&s, 1)[0];
    EnemyChocolatier::apply(&mut s, Seat(1), &Action::Buy(house)).unwrap();
    EnemyChocolatier::apply(&mut s, Seat(1), &Action::EndTurn).unwrap();
    assert_eq!(s.rain_checks, vec![1, 0]);
}

#[test]
fn buy_three_allows_a_third_house() {
    let mut s = new(2, 10);
    give(&mut s, hood_with(Ability::BuyThree), Seat(0));
    s.money[0] = 100;
    for house in cheap_offers(&s, 3) {
        EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).unwrap();
    }
    assert_eq!(s.bought, 3);
    assert!(view(&s, 0).offers.is_empty());
}

#[test]
fn income_counts_five_dollar_points_first_then_stars_and_dollars() {
    let mut s = new(2, 11);
    let five = hood_with(Ability::FiveDollarsPoint);
    let all_points = hood_with(Ability::AllPoints);
    give(&mut s, five, Seat(1));
    give(&mut s, all_points, Seat(1));
    let data = |h: u8| tables().hoods[usize::from(h)].data;
    s.money[1] = 12;
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::EndTurn).unwrap();
    // A star counts with its +1 bonus; a hood without a star gets none.
    let star = |h: u8| match data(h).points {
        0 => 0,
        p => u32::from(p) + 1,
    };
    let bill = |h: u8| u32::from(data(h).dollars);
    let stars = star(five) + star(all_points);
    let dollars = 2 + bill(five) + bill(all_points);
    assert_eq!(s.points[1], 2 + stars, "$12 held: 2 points");
    assert_eq!(s.money[1], 12 + dollars);
}

#[test]
fn twenty_points_at_the_start_of_a_turn_win() {
    let mut s = new(3, 12);
    let starred = (0..hood_count() as u8)
        .find(|&h| tables().hoods[usize::from(h)].data.points > 0)
        .unwrap();
    give(&mut s, starred, Seat(1));
    s.points[1] = 19;
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::EndTurn).unwrap();
    assert_eq!(s.end, Some(End::Points(Seat(1))));
    assert_eq!(s.ranks(), vec![1, 0, 1]);
    assert!(EnemyChocolatier::pending(&s).is_empty());
    let v = view(&s, 0);
    assert!(
        v.seats.iter().all(|seat| seat.recipe.is_some()),
        "recipes shown at the end"
    );
}

#[test]
fn completing_the_recipe_wins_at_once() {
    let mut s = new(2, 13);
    let need = RECIPES[usize::from(s.recipes[0])].need;
    // Give seat 0 every ingredient but one chocolate, then let it buy one.
    let mut left = need;
    let target = (0..hood_count() as u8)
        .find(|&h| {
            s.near_factory[usize::from(h)]
                && tables().hoods[usize::from(h)].data.color == Some(Ingredient::Chocolate)
        })
        .expect("a chocolate hood by the Factory");
    left[Ingredient::Chocolate.index()] -= 1;
    for h in 0..hood_count() as u8 {
        if h == target {
            continue;
        }
        if let Some(c) = tables().hoods[usize::from(h)].data.color {
            for house in houses_of(h) {
                if left[c.index()] > 0 {
                    s.owner[usize::from(house)] = Some(Seat(0));
                    left[c.index()] -= 1;
                }
            }
        }
    }
    assert_eq!(left, [0; 4]);
    assert!(!s.recipe_done(Seat(0)));
    s.money[0] = 50;
    let house = houses_of(target).next().unwrap();
    EnemyChocolatier::apply(&mut s, Seat(0), &Action::Buy(house)).unwrap();
    assert_eq!(s.end, Some(End::Recipe(Seat(0))));
}

#[test]
fn recipes_are_hidden_and_redealt_for_the_ai() {
    let s = new(4, 14);
    let v = view(&s, 2);
    for (i, seat) in v.seats.iter().enumerate() {
        assert_eq!(seat.recipe.is_some(), i == 2);
    }
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let mut moved = false;
    for _ in 0..20 {
        let d = EnemyChocolatier::determinize(&s, Seat(2), &mut rng);
        assert_eq!(d.recipes[2], s.recipes[2]);
        let mut all: Vec<u8> = d.recipes.iter().chain(&d.spare).copied().collect();
        all.sort();
        assert_eq!(all, (0..8).collect::<Vec<u8>>());
        moved |= d.recipes != s.recipes;
    }
    assert!(moved);
}

#[test]
fn the_layout_of_a_seed_is_pinned() {
    let s = new(2, 42);
    let layout: Vec<(u8, u8)> = s.layout.iter().map(|p| (p.section, p.quarters)).collect();
    assert_eq!(layout, PINNED);
    assert_eq!(s.recipes, vec![PINNED_RECIPES.0, PINNED_RECIPES.1]);
}

const PINNED: [(u8, u8); 6] = [(0, 3), (2, 1), (5, 2), (4, 0), (3, 1), (1, 1)];
const PINNED_RECIPES: (u8, u8) = (0, 3);

#[test]
fn the_web_board_matches_the_engine_tables() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../web/src/games/enemy-chocolatier/board.json"
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sections = json["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 6);
    for hood in &tables().hoods {
        let drawn = &sections[usize::from(hood.section)]["hoods"][usize::from(hood.index)];
        assert_eq!(
            drawn["houses"].as_array().unwrap().len(),
            hood.data.prices.len()
        );
        let color = match hood.data.color {
            None => "Gray",
            Some(Ingredient::Chocolate) => "Chocolate",
            Some(Ingredient::Sugar) => "Sugar",
            Some(Ingredient::Nuts) => "Nuts",
            Some(Ingredient::Love) => "Love",
        };
        assert_eq!(drawn["color"], color);
    }
    let drawn: usize = sections
        .iter()
        .map(|s| s["hoods"].as_array().unwrap().len())
        .sum();
    assert_eq!(drawn, hood_count());
}
