use black_hole::{neighbours, Action, BlackHole, Config, Order, State, Tile};
use ludo_core::{Determinize, Game, Outcome, Seat, Viewer};
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn config(order: Order) -> Config {
    Config { order }
}

fn place(s: &mut State, seat: u8, cell: u8, tile: u8) {
    BlackHole::apply(s, Seat(seat), &Action::Place { cell, tile }).unwrap();
}

#[test]
fn red_starts_and_tiles_go_up() {
    let mut s = BlackHole::new(&Config::default(), 2, 1);
    assert_eq!(s.turn, Some(Seat(0)));
    assert!(BlackHole::legal_actions(&s, Seat(0))
        .iter()
        .all(|&Action::Place { tile, .. }| tile == 1));
    assert_eq!(BlackHole::legal_actions(&s, Seat(0)).len(), 21);
    assert!(BlackHole::apply(&mut s, Seat(0), &Action::Place { cell: 0, tile: 2 }).is_err());
    place(&mut s, 0, 0, 1);
    assert!(BlackHole::apply(&mut s, Seat(1), &Action::Place { cell: 0, tile: 1 }).is_err());
    place(&mut s, 1, 1, 1);
    assert_eq!(s.next_tile(Seat(0)), Some(2));
    assert_eq!(BlackHole::legal_actions(&s, Seat(0)).len(), 19);
}

/// [B] The rulebook's endgame: red wins 8 to 11.
#[test]
fn the_rulebook_example() {
    // Row by row from the apex; `r`/`b` for red/blue, `.` for the hole.
    let layout = "b4 r9 r7 b3 r1 r8 b5 b9 r6 r3 r10 r2 b2 b1 . b8 b6 r4 b7 b10 r5";
    let mut s = BlackHole::new(&Config::default(), 2, 0);
    for (i, t) in layout.split_whitespace().enumerate() {
        s.cells[i] = (t != ".").then(|| Tile {
            owner: u8::from(t.starts_with('b')),
            value: t[1..].parse().unwrap(),
        });
    }
    assert_eq!(s.sums_around(14), [8, 11]);
    assert_eq!(neighbours(14), vec![13, 9, 19, 20]);
}

#[test]
fn the_last_tile_finds_the_hole_and_the_lower_total_wins() {
    let mut s = BlackHole::new(&Config::default(), 2, 0);
    // Fill cells 0..19 alternately; cell 20 stays empty.
    for k in 0..20u8 {
        place(&mut s, k % 2, k, k / 2 + 1);
    }
    let f = s.finish.clone().unwrap();
    assert_eq!(f.hole, 20);
    // Neighbours of 20: 19 (blue 10) and 14 (red 8).
    assert_eq!(f.sums, [8, 10]);
    assert_eq!(BlackHole::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
    assert!(BlackHole::pending(&s).is_empty());
}

#[test]
fn free_order_offers_every_tile() {
    let s = BlackHole::new(&config(Order::Free), 2, 0);
    assert_eq!(BlackHole::legal_actions(&s, Seat(0)).len(), 21 * 10);
    assert_eq!(BlackHole::view(&s, Viewer::Seat(Seat(0))).next, None);
}

#[test]
fn random_order_shows_only_the_drawn_tile() {
    let s = BlackHole::new(&config(Order::Random), 2, 7);
    let drawn = s.hands[0][0];
    let v = BlackHole::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(v.next, Some(drawn));
    assert_eq!(v.remaining[0], (1..=10).collect::<Vec<u8>>());
    let mut orders = Vec::new();
    for seed in 0..20 {
        let d = BlackHole::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
        assert_eq!(d.hands[0][0], drawn);
        orders.push(d.hands[1].clone());
    }
    orders.dedup();
    assert!(orders.len() > 1);
}

#[test]
fn equal_totals_draw() {
    let mut s = BlackHole::new(&Config::default(), 2, 0);
    for k in 0..20u8 {
        place(&mut s, k % 2, k, k / 2 + 1);
    }
    // Swap the owners around the hole to make 9 against 9.
    s.cells[14] = Some(Tile { owner: 0, value: 9 });
    s.cells[19] = Some(Tile { owner: 1, value: 9 });
    s.finish.as_mut().unwrap().sums = s.sums_around(20);
    assert_eq!(BlackHole::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}
