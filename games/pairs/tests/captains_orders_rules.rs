use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::captains_orders::testing::table;
use pairs::captains_orders::{
    Action, CaptainsOrders, Config, Ending, Event, Order, Phase, State, HAND_SIZE,
};
use pairs::deck::DECK_SIZE;
use pairs::testing::{r, ranks};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn act(s: &mut State, seat: u8, action: Action) {
    CaptainsOrders::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        CaptainsOrders::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!CaptainsOrders::legal_actions(s, Seat(seat)).contains(&action));
}

fn order(card: u8) -> Action {
    Action::Order { card: r(card) }
}

fn follow(card: u8) -> Action {
    Action::Follow { card: r(card) }
}

fn pass(card: u8, to: u8) -> Action {
    Action::Pass {
        card: r(card),
        to: Seat(to),
    }
}

fn accuse(seat: u8) -> Action {
    Action::Accuse { seat: Seat(seat) }
}

fn cards(s: &State) -> usize {
    s.hands.iter().map(Vec::len).sum::<usize>() + s.supply.deck_len() + s.supply.discard_len()
}

fn result(s: &State) -> &pairs::captains_orders::RoundResult {
    s.last_round.as_ref().expect("a finished round")
}

#[test]
fn a_new_game_deals_six_cards_and_the_left_of_the_dealer_starts() {
    for players in 4..=8u8 {
        let s = CaptainsOrders::new(&Config::default(), players, 3);
        assert!(s.hands.iter().all(|h| h.len() == HAND_SIZE));
        assert_eq!(cards(&s), DECK_SIZE);
        assert_eq!(s.dealer, Seat(players - 1));
        assert_eq!(s.turn, Some(Seat(0)));
        assert_eq!(s.phase, Phase::Turn);
    }
}

#[test]
fn without_an_order_one_plays_passes_asks_or_accuses() {
    let s = table(&[&[2, 9, 9], &[5, 6], &[7, 8], &[3, 4]], &[]);
    let legal = CaptainsOrders::legal_actions(&s, Seat(0));
    // 2 orders, 2 cards × 3 seats to pass, 3 asks, 3 accusations.
    assert_eq!(legal.len(), 2 + 6 + 3 + 3);
    refused(&s, 0, follow(9));
    refused(&s, 0, accuse(0));
    refused(&s, 1, order(5));
}

#[test]
fn a_passed_card_makes_the_receiver_discard() {
    let mut s = table(&[&[2, 9], &[5, 6], &[7, 8], &[3, 4]], &[1]);
    act(&mut s, 0, pass(9, 2));
    assert_eq!(s.hands[2], ranks(&[7, 8, 9]));
    assert_eq!(s.phase, Phase::Discard { after: Seat(1) });
    assert_eq!(s.turn, Some(Seat(2)));
    refused(&s, 2, Action::Discard { card: r(5) });
    act(&mut s, 2, Action::Discard { card: r(7) });
    assert_eq!(s.hands[2], ranks(&[8, 9]));
    assert_eq!(s.discarded[2], ranks(&[7]));
    assert_eq!(s.phase, Phase::Turn);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn asking_makes_the_asked_seat_give_and_the_asker_discard() {
    let mut s = table(&[&[2, 9], &[5, 6], &[7, 8], &[3, 4]], &[1]);
    act(&mut s, 0, Action::Ask { from: Seat(3) });
    assert_eq!(s.phase, Phase::Give { asker: Seat(0) });
    assert_eq!(s.turn, Some(Seat(3)));
    act(&mut s, 3, Action::Give { card: r(4) });
    assert_eq!(s.hands[0], ranks(&[2, 9, 4]));
    assert_eq!(s.turn, Some(Seat(0)));
    act(&mut s, 0, Action::Discard { card: r(2) });
    assert_eq!(s.hands[0], ranks(&[9, 4]));
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn an_order_goes_round_collecting_followed_cards() {
    let mut s = table(&[&[5, 9], &[5, 6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, order(5));
    assert_eq!(s.phase, Phase::Respond);
    assert_eq!(s.turn, Some(Seat(1)));
    // Seat 1 may follow with 5 or 6 but not belay: nothing lower.
    assert!(!CaptainsOrders::legal_actions(&s, Seat(1))
        .iter()
        .any(|a| matches!(a, Action::Order { .. })));
    act(&mut s, 1, follow(5));
    act(&mut s, 2, follow(8));
    refused(&s, 3, follow(3));
    act(&mut s, 3, follow(6));
    assert_eq!(s.hands[0], ranks(&[9, 5, 8, 6]));
    // Back to its owner: discarded face up, and the seat on its left plays.
    assert_eq!(s.order, None);
    assert_eq!(s.supply.seen(), &[r(5)]);
    assert_eq!(s.phase, Phase::Turn);
    assert_eq!(s.turn, Some(Seat(1)));
    assert!(s.log.contains(&Event::OrderDone {
        owner: Seat(0),
        card: r(5)
    }));
}

#[test]
fn a_lower_order_belays_and_starts_round_again() {
    let mut s = table(&[&[5, 9, 9], &[2, 6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, order(5));
    act(&mut s, 1, order(2));
    assert_eq!(
        s.order,
        Some(Order {
            card: r(2),
            owner: Seat(1)
        })
    );
    assert_eq!(s.supply.seen(), &[r(5)]);
    act(&mut s, 2, follow(7));
    act(&mut s, 3, follow(3));
    // Seat 0, the owner of the belayed Order, answers the new one.
    assert_eq!(s.turn, Some(Seat(0)));
    act(&mut s, 0, follow(9));
    assert_eq!(s.order, None);
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(s.hands[1], ranks(&[6, 7, 3, 9]));
}

#[test]
fn a_mutineer_naming_the_captain_wins_the_round() {
    let mut s = table(&[&[5, 9], &[2, 6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, accuse(1));
    let res = result(&s);
    assert_eq!(
        res.ending,
        Ending::Accused {
            accuser: Seat(0),
            target: Seat(1),
            right: true
        }
    );
    assert_eq!(res.captains, vec![Seat(1)]);
    assert_eq!(res.winners, vec![Seat(0)]);
    assert_eq!(s.wins, vec![1, 0, 0, 0]);
    // The next round: dealt by seat 0, seat 1 plays first.
    assert_eq!(s.round, 2);
    assert_eq!(s.dealer, Seat(0));
    assert_eq!(s.turn, Some(Seat(1)));
    assert!(s.log.is_empty());
}

#[test]
fn a_wrong_accusation_gives_the_round_to_the_richest_other_mutineer() {
    // Captain: seat 1 (2). Points: 14, –, 15, 9.
    let mut s = table(&[&[5, 9], &[2, 6], &[7, 8], &[3, 6]], &[1]);
    s.turn = Some(Seat(2));
    act(&mut s, 2, accuse(3));
    let res = result(&s);
    assert_eq!(res.points, vec![14, 8, 15, 9]);
    // Seat 2 was wrong and cannot win, even with the most points.
    assert_eq!(res.winners, vec![Seat(0)]);
}

#[test]
fn a_captain_naming_another_captain_does_not_win() {
    let mut s = table(&[&[2, 9], &[2, 6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, accuse(1));
    let res = result(&s);
    assert_eq!(res.captains, vec![Seat(0), Seat(1)]);
    assert_eq!(res.winners, vec![Seat(2)]);
}

#[test]
fn tied_mutineers_each_win_the_round() {
    let mut s = table(&[&[2, 9], &[5, 9], &[6, 8], &[3, 6]], &[1]);
    s.turn = Some(Seat(3));
    act(&mut s, 3, accuse(1));
    assert_eq!(result(&s).winners, vec![Seat(1), Seat(2)]);
    assert_eq!(s.wins, vec![0, 1, 1, 0]);
}

#[test]
fn a_captain_running_out_of_cards_wins() {
    let mut s = table(&[&[2], &[5, 6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, pass(2, 1));
    let res = result(&s);
    assert_eq!(
        res.ending,
        Ending::Emptied {
            seat: Seat(0),
            card: r(2),
            right: true
        }
    );
    assert_eq!(res.winners, vec![Seat(0)]);
    // The passed card stays with its receiver, who does not discard.
    assert_eq!(res.hands[1], ranks(&[5, 6, 2]));
}

#[test]
fn a_mutineer_running_out_of_cards_loses_to_the_points() {
    let mut s = table(&[&[4], &[5, 6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, order(4));
    let res = result(&s);
    assert_eq!(
        res.ending,
        Ending::Emptied {
            seat: Seat(0),
            card: r(4),
            right: false
        }
    );
    assert_eq!(res.captains, vec![Seat(3)]);
    assert_eq!(res.winners, vec![Seat(2)]);
}

#[test]
fn a_tie_for_lowest_still_lets_the_runner_win() {
    let mut s = table(&[&[3], &[5, 6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, order(3));
    assert_eq!(result(&s).winners, vec![Seat(0)]);
}

#[test]
fn following_with_the_last_card_also_ends_the_round() {
    let mut s = table(&[&[5, 9], &[6], &[7, 8], &[3, 6]], &[1]);
    act(&mut s, 0, order(5));
    act(&mut s, 1, follow(6));
    let res = result(&s);
    assert_eq!(
        res.ending,
        Ending::Emptied {
            seat: Seat(1),
            card: r(6),
            right: false
        }
    );
    assert_eq!(res.hands[0], ranks(&[9, 6]));
}

#[test]
fn others_see_that_a_card_moved_but_not_which() {
    let mut s = table(&[&[2, 9], &[5, 6], &[7, 8], &[3, 4]], &[1]);
    act(&mut s, 0, pass(9, 2));
    act(&mut s, 2, Action::Discard { card: r(7) });
    let passed = |viewer: Viewer| CaptainsOrders::view(&s, viewer).log[0].clone();
    let hidden = Event::Passed {
        from: Seat(0),
        to: Seat(2),
        card: None,
    };
    let shown = Event::Passed {
        from: Seat(0),
        to: Seat(2),
        card: Some(r(9)),
    };
    assert_eq!(passed(Viewer::Seat(Seat(0))), shown);
    assert_eq!(passed(Viewer::Seat(Seat(2))), shown);
    assert_eq!(passed(Viewer::Seat(Seat(1))), hidden);
    assert_eq!(passed(Viewer::Spectator), hidden);
    let discard = |seat: u8| CaptainsOrders::view(&s, Viewer::Seat(Seat(seat))).log[1].clone();
    assert_eq!(
        discard(2),
        Event::Discarded {
            seat: Seat(2),
            card: Some(r(7))
        }
    );
    assert_eq!(
        discard(0),
        Event::Discarded {
            seat: Seat(2),
            card: None
        }
    );
    assert!(CaptainsOrders::view(&s, Viewer::Seat(Seat(0)))
        .discarded
        .is_empty());
    assert_eq!(
        CaptainsOrders::view(&s, Viewer::Seat(Seat(2))).discarded,
        ranks(&[7])
    );
}

#[test]
fn three_round_wins_end_the_game() {
    let mut s = table(&[&[5, 9], &[2, 6], &[7, 8], &[3, 6]], &[1]);
    s.wins = vec![2, 0, 2, 0];
    act(&mut s, 0, accuse(1));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.turn, None);
    assert_eq!(
        CaptainsOrders::outcome(&s),
        Some(Outcome::Ranked(vec![0, 2, 1, 2]))
    );
    refused(&s, 0, accuse(1));
}

#[test]
fn the_cards_are_all_accounted_for_and_games_end() {
    let mut rng = SmallRng::seed_from_u64(5);
    for players in 4..=8u8 {
        for _ in 0..10 {
            let mut s = CaptainsOrders::new(&Config::default(), players, rng.gen());
            let mut steps = 0;
            while let Some(seat) = s.turn {
                let legal = CaptainsOrders::legal_actions(&s, seat);
                // Accusations end rounds fast: play them rarely.
                let quiet: Vec<&Action> = legal
                    .iter()
                    .filter(|a| !matches!(a, Action::Accuse { .. }))
                    .collect();
                let action = if rng.gen_bool(0.02) || quiet.is_empty() {
                    legal.choose(&mut rng).unwrap().clone()
                } else {
                    (*quiet.choose(&mut rng).unwrap()).clone()
                };
                act(&mut s, seat.0, action);
                if s.phase != Phase::Over {
                    assert_eq!(cards(&s) + usize::from(s.order.is_some()), DECK_SIZE);
                }
                steps += 1;
                assert!(steps < 10_000);
            }
            assert!(s.wins.iter().any(|&w| w >= 3));
        }
    }
}
