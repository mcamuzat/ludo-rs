use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::blackstone::testing::table;
use pairs::blackstone::{Action, Blackstone, Config, Event, Phase, State, Status, ANTE};
use pairs::deck::DECK_SIZE;
use pairs::testing::ranks;

fn act(s: &mut State, seat: u8, action: Action) {
    Blackstone::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn take(count: u8) -> Action {
    Action::Take { count }
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot) + i64::from(s.in_front())
}

#[test]
fn the_opening_keeps_one_black_stone_and_whites_go_to_the_middle() {
    for players in 2..=8u8 {
        for seed in 0..20 {
            let s = Blackstone::new(&Config::default(), players, seed);
            assert!(s.blacks.iter().all(|&b| b <= 1));
            assert!(s.whites.iter().all(|c| c.value() != 10));
            let held: usize = s.blacks.iter().map(|&b| usize::from(b)).sum();
            assert_eq!(s.whites.len() + held + s.supply.deck_len(), DECK_SIZE);
            assert_eq!(balance(&s), 0);
            let Some(Event::Opened { cards, first, .. }) = s.events.first() else {
                panic!("{:?}", s.events);
            };
            let low = cards.iter().map(|c| c[0]).min().unwrap();
            assert!(cards[first.index()][0] == low);
            assert_eq!(s.turn, Some(*first));
        }
    }
}

#[test]
fn each_player_takes_at_least_as_many_as_the_last() {
    let mut s = table(&[0, 0, 0], &[], &[2, 3, 4, 5, 6, 7, 8]);
    assert!(Blackstone::legal_actions(&s, Seat(0)).contains(&take(1)));
    act(&mut s, 0, take(2));
    assert_eq!(s.whites, ranks(&[8, 7]));
    let mut copy = s.clone();
    assert!(Blackstone::apply(&mut copy, Seat(1), &take(1)).is_err());
    assert_eq!(copy, s);
    act(&mut s, 1, Action::Fold);
    // A fold leaves the minimum where it was.
    assert_eq!(s.min_take, 2);
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn folding_costs_the_ante_and_half_the_whites_rounded_up() {
    let mut s = table(&[0, 0, 0], &[2, 3, 4, 5, 6], &[9]);
    act(&mut s, 0, Action::Fold);
    assert!(s.events.contains(&Event::Folded {
        seat: Seat(0),
        paid: ANTE + 3
    }));
    assert_eq!(s.pot, 8);
    assert_eq!(s.chips[0], -8);
    assert_eq!(balance(&s), 0);
}

#[test]
fn a_second_black_stone_loses_and_stops_the_deal() {
    let mut s = table(&[1, 0, 0], &[2, 3, 4], &[6, 6, 10, 5]);
    act(&mut s, 0, take(3));
    // 5 is white, then the 10: lost, and the 6s are never dealt.
    assert!(s.events.contains(&Event::Took {
        seat: Seat(0),
        count: 3,
        cards: ranks(&[5, 10]),
    }));
    assert!(s.events.contains(&Event::Lost {
        seat: Seat(0),
        paid: ANTE + 4
    }));
    // Seats 1 and 2 take back their antes and split 9 chips: 4 each.
    assert!(s.events.contains(&Event::PotWon {
        seats: vec![Seat(1), Seat(2)],
        share: 4,
        left: 1,
    }));
    assert_eq!(s.hand_no, 2);
    assert_eq!(balance(&s), 0);
}

#[test]
fn the_last_player_in_wins() {
    let mut s = table(&[0, 0, 0], &[2, 3], &[9]);
    act(&mut s, 0, Action::Fold);
    act(&mut s, 1, Action::Fold);
    // 6 + 6 chips in the pot: seat 2 takes them all.
    assert!(s.events.contains(&Event::PotWon {
        seats: vec![Seat(2)],
        share: 12,
        left: 0,
    }));
    // +12, less the ante of hand 2.
    assert_eq!(s.chips[2], 12 - 5);
}

#[test]
fn the_game_ends_after_its_hands() {
    let mut s = table(&[0, 0], &[], &[9]);
    s.hands_total = 1;
    act(&mut s, 0, Action::Fold);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Blackstone::pending(&s), Pending::none());
    assert_eq!(Blackstone::outcome(&s), Some(Outcome::Ranked(vec![1, 0])));
    assert_eq!(s.status[0], Status::Folded);
    let v = Blackstone::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(v, Blackstone::view(&s, Viewer::Spectator));
}
