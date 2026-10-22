use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::pickup::testing::table;
use pairs::pickup::{hand_size, target, Action, Config, Event, Length, Phase, Pickup, State};
use pairs::testing::{r, ranks};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

fn act(s: &mut State, seat: u8, action: Action) {
    Pickup::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn choose(s: &mut State, seat: u8, card: u8) {
    act(s, seat, Action::Choose { card: r(card) });
}

fn total(s: &State) -> usize {
    s.deck.len()
        + s.pool.len()
        + s.discard.len()
        + s.hands.iter().map(Vec::len).sum::<usize>()
        + s.piles.iter().map(Vec::len).sum::<usize>()
        // Revealed cards still to act are on the table.
        + s.waiting.len()
        + usize::from(s.taking.is_some())
}

/// Seats in the order their `Took`/`Stacked` events came.
fn actors(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Took { seat, .. } | Event::Stacked { seat, .. } => Some(seat.0),
            _ => None,
        })
        .collect()
}

#[test]
fn deal_sizes_and_targets() {
    assert_eq!((2..=6).map(hand_size).collect::<Vec<_>>(), [7, 6, 6, 5, 5]);
    assert_eq!(
        (2..=6).map(target).collect::<Vec<_>>(),
        [150, 100, 100, 75, 75]
    );
    for players in 2..=6u8 {
        let s = Pickup::new(&Config::default(), players, u64::from(players));
        assert!(s.hands.iter().all(|h| h.len() == hand_size(players)));
        assert_eq!(s.pool.len(), 5);
        assert_eq!(total(&s), 55);
        assert_eq!(Pickup::pending(&s).0.len(), usize::from(players));
    }
}

#[test]
fn cards_act_from_lowest_to_highest() {
    let mut s = table(&[&[9, 1], &[3, 1], &[6, 1]], &[3, 5, 8, 10, 10], &[]);
    choose(&mut s, 0, 9);
    choose(&mut s, 1, 3);
    choose(&mut s, 2, 6);
    assert_eq!(actors(&s.events), [1, 2, 0]);
    assert_eq!(s.piles, [ranks(&[8]), ranks(&[3]), ranks(&[5])]);
}

#[test]
fn ties_act_from_the_left_of_the_last_actor() {
    // Dealer is seat 2: the first tie starts on its left, seat 0.
    let mut s = table(
        &[&[8, 8], &[8, 8], &[8, 8]],
        &[5, 5, 5, 5, 5],
        &[5, 5, 5, 5, 5],
    );
    for seat in 0..3 {
        choose(&mut s, seat, 8);
    }
    assert_eq!(actors(&s.events), [0, 1, 2]);
    // Seat 2 acted last: next turn starts with seat 0 again.
    for seat in 0..3 {
        choose(&mut s, seat, 8);
    }
    assert_eq!(actors(&s.events), [0, 1, 2]);
    let mut s = table(
        &[&[8, 8], &[8, 8], &[1, 8]],
        &[5, 5, 5, 5, 5],
        &[5, 5, 5, 5, 5],
    );
    for (seat, card) in [(0, 8), (1, 8), (2, 1)] {
        choose(&mut s, seat, card);
    }
    // The 1 (seat 2) acts first; the tied 8s then start on its left.
    assert_eq!(actors(&s.events), [2, 0, 1]);
}

#[test]
fn a_take_knocks_out_the_rank_in_the_pool_and_other_piles() {
    let mut s = table(&[&[8, 1], &[9, 1]], &[10, 9, 8, 8], &[]);
    s.piles[1] = ranks(&[8, 4]);
    choose(&mut s, 0, 8);
    choose(&mut s, 1, 9);
    assert!(s.events.contains(&Event::Took {
        seat: Seat(0),
        played: r(8),
        card: r(8),
        knocked: vec![(Seat(1), 1)],
        pool_knocked: 1,
    }));
    assert_eq!(s.piles[0], ranks(&[8]));
    assert_eq!(s.piles[1], ranks(&[4, 9]));
}

#[test]
fn a_choice_of_ranks_asks_the_player() {
    let mut s = table(&[&[9, 1], &[1, 1]], &[9, 2], &[]);
    choose(&mut s, 0, 9);
    choose(&mut s, 1, 1);
    assert_eq!(s.phase, Phase::Take);
    assert_eq!(Pickup::pending(&s).0, [Seat(0)]);
    assert_eq!(
        Pickup::legal_actions(&s, Seat(0)),
        [Action::Take { card: r(2) }, Action::Take { card: r(9) }]
    );
    act(&mut s, 0, Action::Take { card: r(2) });
    assert_eq!(s.piles[0], ranks(&[2]));
    assert_eq!(s.phase, Phase::Choose);
}

#[test]
fn knockouts_spare_the_taker() {
    let mut s = table(&[&[5, 1], &[1, 1]], &[5], &[]);
    s.piles[0] = ranks(&[5]);
    s.piles[1] = ranks(&[5]);
    choose(&mut s, 0, 5);
    choose(&mut s, 1, 1);
    assert_eq!(s.piles[0], ranks(&[5, 5]));
    assert_eq!(s.piles[1], ranks(&[1]));
}

#[test]
fn nothing_to_take_stacks_the_card() {
    let mut s = table(&[&[2, 1], &[3, 1]], &[9, 8, 7], &[]);
    s.piles[1] = ranks(&[2]);
    choose(&mut s, 0, 2);
    choose(&mut s, 1, 3);
    assert_eq!(s.piles[0], ranks(&[2]));
    assert_eq!(s.piles[1], ranks(&[2, 3]));
    assert!(s.events.contains(&Event::Stacked {
        seat: Seat(0),
        card: r(2)
    }));
}

#[test]
fn the_turn_ends_and_a_new_pool_is_dealt() {
    let mut s = table(&[&[5, 1], &[6, 1]], &[9, 8, 7, 5, 6], &[1, 2, 3, 4, 10, 10]);
    choose(&mut s, 0, 5);
    choose(&mut s, 1, 6);
    // 9, 8, 7 left in the pool are discarded; five new cards come.
    assert_eq!(s.pool, ranks(&[1, 2, 3, 4, 10]));
    assert_eq!(s.phase, Phase::Choose);
    assert!(s.discard.contains(&r(9)));
}

#[test]
fn a_short_deck_deals_a_short_pool() {
    let mut s = table(&[&[5, 1], &[6, 1]], &[9], &[2, 3]);
    choose(&mut s, 0, 5);
    choose(&mut s, 1, 6);
    assert_eq!(s.pool, ranks(&[2, 3]));
}

#[test]
fn one_deal_ends_when_hands_are_empty() {
    let mut s = table(&[&[2], &[6]], &[5, 2], &[]);
    choose(&mut s, 0, 2);
    choose(&mut s, 1, 6);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.scores, [2, 5]);
    assert_eq!(Pickup::outcome(&s), Some(Outcome::Ranked(vec![1, 0])));
}

#[test]
fn target_redeals_until_someone_reaches_it() {
    let config = Config {
        length: Length::Target,
    };
    let mut rng = SmallRng::seed_from_u64(3);
    let mut s = Pickup::new(&config, 4, 11);
    let mut deals = 1;
    while let Some(&seat) = Pickup::pending(&s).0.first() {
        let a = Pickup::legal_actions(&s, seat)
            .choose(&mut rng)
            .cloned()
            .unwrap();
        Pickup::apply(&mut s, seat, &a).unwrap();
        deals += s
            .events
            .iter()
            .filter(|e| matches!(e, Event::Dealt { .. }))
            .count();
    }
    assert!(deals > 1, "{deals}");
    assert!(s.scores.iter().any(|&x| x >= target(4)));
}

#[test]
fn a_random_one_deal_game_keeps_every_card() {
    for players in 2..=6u8 {
        let mut rng = SmallRng::seed_from_u64(u64::from(players));
        let mut s = Pickup::new(&Config::default(), players, 40 + u64::from(players));
        while let Some(&seat) = Pickup::pending(&s).0.first() {
            let a = Pickup::legal_actions(&s, seat)
                .choose(&mut rng)
                .cloned()
                .unwrap();
            Pickup::apply(&mut s, seat, &a).unwrap();
            assert_eq!(total(&s), 55);
        }
        assert_eq!(s.phase, Phase::Over);
    }
}

#[test]
fn nobody_sees_who_has_chosen() {
    let mut s = table(&[&[5, 1], &[6, 1], &[7, 1]], &[9], &[]);
    choose(&mut s, 0, 5);
    assert_eq!(
        Pickup::visible_pending(&s, Viewer::Seat(Seat(1))).0,
        [Seat(1)]
    );
    assert!(Pickup::visible_pending(&s, Viewer::Seat(Seat(0)))
        .0
        .is_empty());
    assert!(Pickup::visible_pending(&s, Viewer::Spectator).0.is_empty());
    let v1 = Pickup::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(v1.chosen, None);
    assert_eq!(Pickup::view(&s, Viewer::Seat(Seat(0))).chosen, Some(r(5)));
    let json = serde_json::to_string(&v1).unwrap();
    assert!(
        !json.contains("\"deck\"") && !json.contains("seed"),
        "{json}"
    );
}

#[test]
fn illegal_moves_are_refused() {
    let mut s = table(&[&[5, 1], &[6, 1]], &[9], &[]);
    choose(&mut s, 0, 5);
    for (seat, a) in [
        (0, Action::Choose { card: r(1) }),
        (1, Action::Choose { card: r(9) }),
        (1, Action::Take { card: r(9) }),
    ] {
        let mut copy = s.clone();
        assert!(Pickup::apply(&mut copy, Seat(seat), &a).is_err(), "{a:?}");
        assert_eq!(copy, s);
    }
}

#[test]
fn a_new_deal_keeps_the_last_actor_for_ties() {
    let mut s = table(&[&[5], &[6]], &[5, 6], &[]);
    s.config = Config {
        length: Length::Target,
    };
    choose(&mut s, 0, 5);
    choose(&mut s, 1, 6);
    // Seat 1 acted last in deal 1; only the game's first play starts left
    // of the dealer.
    assert_eq!(s.deal_no, 2);
    assert_eq!(s.dealer, Seat(0));
    assert_eq!(s.last_actor, Seat(1));
}
