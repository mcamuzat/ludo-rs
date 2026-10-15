use ludo_core::{Game, Outcome, Pending, Seat};
use pairs::rocket::testing::table;
use pairs::rocket::{Action, Config, Event, Phase, Rocket, State, Status};
use pairs::testing::{r, ranks};

fn act(s: &mut State, seat: u8, action: Action) {
    Rocket::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

#[test]
fn a_round_deals_one_card_each_then_the_dealer() {
    for players in 1..=6u8 {
        for seed in 0..20 {
            let s = Rocket::new(&Config::default(), players, seed);
            assert!(s.stacks.iter().all(|st| st.len() == 1) || s.round_no > 1);
            // The dealer never shows a 3: it ends the round at once.
            assert!(s.dealer.iter().all(|c| c.value() != 3));
            assert!(s.turn == Some(Seat(0)) || s.phase == Phase::Over);
        }
    }
}

#[test]
fn a_pair_costs_its_value() {
    let mut s = table(&[&[8], &[2]], &[5], 3, &[9, 8]);
    act(&mut s, 0, Action::Hit);
    assert_eq!(s.status[0], Status::Busted);
    assert_eq!(s.chips[0], -8);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn folding_pays_the_lowest_card_in_sight() {
    let mut s = table(&[&[8, 4], &[2]], &[5], 3, &[9]);
    assert_eq!(s.fold_cost(Seat(0)), 4);
    act(&mut s, 0, Action::Fold);
    assert_eq!(s.chips[0], -4);
    assert_eq!(s.status[0], Status::Folded);
    let s = table(&[&[8, 4], &[2]], &[1, 5], 3, &[9]);
    assert_eq!(s.fold_cost(Seat(0)), 1);
}

#[test]
fn after_the_players_the_dealer_hits() {
    // The dealer draws a 6: no harm; the first player acts again.
    let mut s = table(&[&[8], &[2]], &[5], 3, &[6, 9, 7]);
    act(&mut s, 0, Action::Hit);
    act(&mut s, 1, Action::Hit);
    assert!(s.events.contains(&Event::DealerHit { card: r(6) }));
    assert_eq!(s.dealer, ranks(&[5, 6]));
    assert_eq!(s.turn, Some(Seat(0)));
}

#[test]
fn a_dealer_pair_pays_the_survivors() {
    let mut s = table(&[&[8], &[2], &[4]], &[5, 7], 3, &[9, 7, 2, 6]);
    act(&mut s, 0, Action::Hit); // 6
    act(&mut s, 1, Action::Hit); // 2: pair, pays 2
    act(&mut s, 2, Action::Fold); // pays 4
                                  // The dealer pairs its 7: seat 0 alone is paid 7.
    assert!(s.events.contains(&Event::DealerLost {
        card: r(7),
        seats: vec![Seat(0)],
        amount: 7,
    }));
    assert_eq!(s.chips, vec![7, -2, -4]);
    assert_eq!(s.round_no, 2);
}

#[test]
fn a_3_kills_the_dealer() {
    // The player draws the 9, the dealer the 3.
    let mut s = table(&[&[8]], &[5], 1, &[3, 9]);
    act(&mut s, 0, Action::Hit);
    assert!(s.events.contains(&Event::DealerLost {
        card: r(3),
        seats: vec![Seat(0)],
        amount: 3,
    }));
    assert_eq!(s.chips, vec![3]);
}

#[test]
fn with_everyone_out_the_house_keeps_it_all() {
    let mut s = table(&[&[8], &[2]], &[5], 1, &[9]);
    act(&mut s, 0, Action::Fold);
    act(&mut s, 1, Action::Fold);
    assert_eq!(s.chips, vec![-5, -2]);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Rocket::pending(&s), Pending::none());
    assert_eq!(Rocket::outcome(&s), Some(Outcome::Ranked(vec![1, 0])));
}

#[test]
fn a_whole_game_runs_its_rounds() {
    let mut s = Rocket::new(&Config::default(), 3, 8);
    while let Some(&seat) = Rocket::pending(&s).0.first() {
        act(&mut s, seat.0, Action::Hit);
    }
    assert_eq!(s.round_no, 10);
}
