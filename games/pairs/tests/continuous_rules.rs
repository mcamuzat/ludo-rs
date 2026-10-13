use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::continuous::testing::{r, ranks, supply, table, table_with_rest};
use pairs::continuous::{default_target, Action, Config, ContinuousPairs, Event, State, BURN};
use pairs::deck::{Supply, DECK_SIZE};
use serde_json::json;

fn hit(s: &mut State, seat: u8) {
    ContinuousPairs::apply(s, Seat(seat), &Action::Hit).unwrap();
}

fn fold(s: &mut State, seat: u8, from: u8, card: u8) {
    let action = Action::Fold {
        seat: Seat(from),
        card: r(card),
    };
    ContinuousPairs::apply(s, Seat(seat), &action).unwrap();
}

#[test]
fn targets_follow_the_booklet() {
    let targets: Vec<u8> = (2..=8).map(default_target).collect();
    assert_eq!(targets, vec![31, 21, 16, 13, 11, 11, 11]);
}

#[test]
fn a_new_game_deals_one_card_each_and_keeps_55_cards() {
    for players in 2..=8u8 {
        for seed in 0..20 {
            let s = ContinuousPairs::new(&Config::default(), players, seed);
            assert!(s.stacks.iter().all(|stack| !stack.is_empty()));
            let in_play: usize = s.stacks.iter().map(Vec::len).sum();
            assert_eq!(
                in_play + s.supply.deck_len() + s.supply.discard_len(),
                DECK_SIZE
            );
            assert_eq!(s.target, default_target(players));
            assert_eq!(ContinuousPairs::pending(&s).0.len(), 1);
        }
    }
}

#[test]
fn a_zero_target_falls_back_to_the_booklet() {
    let s = ContinuousPairs::new(&Config { target: Some(0) }, 2, 1);
    assert_eq!(s.target, 31);
    assert!(!ContinuousPairs::pending(&s).is_empty());
}

#[test]
fn a_custom_target_is_kept() {
    let s = ContinuousPairs::new(&Config { target: Some(9) }, 3, 1);
    assert_eq!(s.target, 9);
}

#[test]
fn the_lowest_card_starts() {
    let s = State::deal(3, 21, supply(&[9, 4, 7]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.stacks, vec![ranks(&[9]), ranks(&[4]), ranks(&[7])]);
}

#[test]
fn the_booklet_tie_break_example() {
    // A 10, B 4, C 4, D 10, E 7; then B 8 and C 9: B starts.
    let s = State::deal(5, 13, supply(&[10, 4, 4, 10, 7, 8, 9]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.stacks[1], ranks(&[4, 8]));
    assert_eq!(s.stacks[2], ranks(&[4, 9]));
    assert_eq!(s.stacks[0], ranks(&[10]));
}

#[test]
fn a_tie_break_card_that_pairs_is_replaced() {
    // A 4, B 4, C 9; A draws a 4 (pair: replaced by a 7), B draws an 8.
    let s = State::deal(3, 21, supply(&[4, 4, 9, 4, 7, 8]));
    assert_eq!(s.stacks[0], ranks(&[4, 7]));
    assert_eq!(s.stacks[1], ranks(&[4, 8]));
    assert_eq!(s.turn, Some(Seat(0)));
    assert_eq!(s.supply.seen(), &[r(4)]);
    assert!(s.events.contains(&Event::Replaced {
        seat: Seat(0),
        card: r(4)
    }));
}

#[test]
fn a_second_tie_deals_a_third_card() {
    // A 3, B 3; then A 6, B 6; then A 9, B 2: B starts.
    let s = State::deal(2, 31, supply(&[3, 3, 6, 6, 9, 2]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.stacks[1], ranks(&[3, 6, 2]));
}

#[test]
fn a_tie_break_that_runs_out_of_cards_still_starts() {
    let s = State::deal(2, 31, supply(&[3, 3]));
    assert_eq!(s.turn, Some(Seat(0)));
}

#[test]
fn a_hit_without_a_pair_grows_the_stack_and_passes_left() {
    let mut s = table(&[&[8], &[5]], &[3]);
    hit(&mut s, 0);
    assert_eq!(s.stacks[0], ranks(&[8, 3]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![Event::Dealt {
            seat: Seat(0),
            card: r(3)
        }]
    );
}

#[test]
fn a_pair_scores_its_rank_and_clears_only_that_stack() {
    let mut s = table(&[&[8, 3], &[5]], &[8]);
    hit(&mut s, 0);
    assert!(s.stacks[0].is_empty());
    assert_eq!(s.scored[0], ranks(&[8]));
    assert_eq!(s.score(Seat(0)), 8);
    assert_eq!(s.supply.seen(), ranks(&[8, 3]).as_slice());
    assert_eq!(s.stacks[1], ranks(&[5]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![Event::Paired {
            seat: Seat(0),
            card: r(8)
        }]
    );
}

#[test]
fn folding_takes_any_card_in_play() {
    let mut s = table(&[&[9, 6], &[2, 7]], &[]);
    fold(&mut s, 0, 1, 2);
    assert_eq!(s.stacks[1], ranks(&[7]));
    assert!(s.stacks[0].is_empty());
    assert_eq!(s.scored[0], ranks(&[2]));
    assert_eq!(s.supply.seen(), ranks(&[9, 6]).as_slice());
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn folding_on_your_own_card_discards_the_rest() {
    let mut s = table(&[&[9, 4], &[7]], &[]);
    fold(&mut s, 0, 0, 4);
    assert!(s.stacks[0].is_empty());
    assert_eq!(s.scored[0], ranks(&[4]));
    assert_eq!(s.supply.seen(), ranks(&[9]).as_slice());
}

#[test]
fn every_card_in_play_is_a_legal_fold() {
    let s = table(&[&[9, 4], &[7]], &[1]);
    let legal = ContinuousPairs::legal_actions(&s, Seat(0));
    assert_eq!(legal.len(), 4);
    assert!(legal.contains(&Action::Hit));
    for (seat, card) in [(0, 9), (0, 4), (1, 7)] {
        assert!(legal.contains(&Action::Fold {
            seat: Seat(seat),
            card: r(card)
        }));
    }
    assert!(ContinuousPairs::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn a_player_without_cards_hits_automatically() {
    let mut s = table(&[&[], &[5]], &[4, 9]);
    s.turn = Some(Seat(1));
    hit(&mut s, 1);
    // Seat 0 had no card: it received one, and that was its turn.
    assert_eq!(s.stacks, vec![ranks(&[9]), ranks(&[5, 4])]);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![
            Event::Dealt {
                seat: Seat(1),
                card: r(4)
            },
            Event::Dealt {
                seat: Seat(0),
                card: r(9)
            },
        ]
    );
}

#[test]
fn folding_on_an_opponents_last_card_refills_both() {
    let mut s = table(&[&[6], &[3]], &[7, 8]);
    fold(&mut s, 0, 1, 3);
    // Both stacks were empty: seat 1 then seat 0 got a card, then seat 1 decides.
    assert_eq!(s.stacks, vec![ranks(&[8]), ranks(&[7])]);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn reaching_the_target_ends_the_game_with_one_loser() {
    let mut s = table(&[&[5, 2], &[9], &[4]], &[5]);
    s.target = 5;
    hit(&mut s, 0);
    assert_eq!(s.loser, Some(Seat(0)));
    assert_eq!(ContinuousPairs::pending(&s), Pending::none());
    assert_eq!(
        ContinuousPairs::outcome(&s),
        Some(Outcome::Ranked(vec![1, 0, 0]))
    );
    assert!(ContinuousPairs::legal_actions(&s, Seat(1)).is_empty());
    assert!(ContinuousPairs::apply(&mut s, Seat(1), &Action::Hit).is_err());
}

#[test]
fn scores_add_up_across_scoring_cards() {
    let mut s = table(&[&[3], &[9]], &[]);
    s.scored[0] = ranks(&[8, 2]);
    assert_eq!(s.score(Seat(0)), 10);
    assert_eq!(ContinuousPairs::outcome(&s), None);
}

#[test]
fn a_refused_action_leaves_the_state_alone() {
    let s = table(&[&[9, 4], &[7]], &[1]);
    let refused = [
        (
            Seat(0),
            Action::Fold {
                seat: Seat(1),
                card: r(4),
            },
        ),
        (Seat(1), Action::Hit),
    ];
    for (seat, action) in refused {
        let mut copy = s.clone();
        assert!(ContinuousPairs::apply(&mut copy, seat, &action).is_err());
        assert_eq!(copy, s);
    }
}

#[test]
fn folding_from_a_seat_outside_the_table_is_refused() {
    let s = table(&[&[9, 4], &[7]], &[1]);
    let mut copy = s.clone();
    let action = Action::Fold {
        seat: Seat(7),
        card: r(4),
    };
    assert!(ContinuousPairs::apply(&mut copy, Seat(0), &action).is_err());
    assert_eq!(copy, s);
}

#[test]
fn no_card_to_draw_means_no_hit() {
    let s = table(&[&[5], &[9]], &[]);
    let legal = ContinuousPairs::legal_actions(&s, Seat(0));
    assert!(!legal.contains(&Action::Hit));
    assert_eq!(legal.len(), 2);
    let mut copy = s.clone();
    assert!(ContinuousPairs::apply(&mut copy, Seat(0), &Action::Hit).is_err());
    assert_eq!(copy, s);
}

#[test]
fn an_empty_deck_is_reshuffled_from_the_discards() {
    let mut s = table(&[&[5], &[9]], &[]);
    s.supply = Supply::from_parts(
        Vec::new(),
        ranks(&[2, 2, 3, 3, 4, 4, 4]),
        Vec::new(),
        BURN,
        3,
    );
    hit(&mut s, 0);
    assert_eq!(s.events[0], Event::Reshuffled);
    assert_eq!(s.stacks[0].len(), 2);
    assert_eq!(s.supply.deck_len(), 1);
    assert_eq!(s.supply.discard_len(), 5);
}

#[test]
fn every_viewer_sees_the_same_table() {
    let s = table_with_rest(&[&[10], &[9]]);
    let a = ContinuousPairs::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(a, ContinuousPairs::view(&s, Viewer::Seat(Seat(1))));
    assert_eq!(a, ContinuousPairs::view(&s, Viewer::Spectator));
    assert_eq!(a.turn, Some(Seat(0)));
    assert_eq!(a.lowest, Some(r(9)));
    assert_eq!(a.deck, 53);
    assert_eq!(a.unlocated, vec![1, 2, 3, 4, 5, 6, 7, 8, 8, 9]);
}

#[test]
fn the_json_the_web_relies_on() {
    assert_eq!(serde_json::to_value(Action::Hit).unwrap(), json!("Hit"));
    let fold = Action::Fold {
        seat: Seat(1),
        card: r(3),
    };
    assert_eq!(
        serde_json::to_value(&fold).unwrap(),
        json!({"Fold": {"seat": 1, "card": 3}})
    );
    assert!(serde_json::from_value::<Action>(json!({"Fold": {"seat": 1, "card": 11}})).is_err());
    assert_eq!(
        serde_json::to_value(Event::Folded {
            seat: Seat(0),
            from: Seat(1),
            card: r(3)
        })
        .unwrap(),
        json!({"type": "Folded", "seat": 0, "from": 1, "card": 3})
    );
    assert_eq!(
        serde_json::to_value(Event::Reshuffled).unwrap(),
        json!({"type": "Reshuffled"})
    );
    let s = table(&[&[8], &[5]], &[3]);
    let view = serde_json::to_value(ContinuousPairs::view(&s, Viewer::Spectator)).unwrap();
    // No risk estimate in the view: weighing the odds is the player's job.
    assert!(view["seats"][0].get("pairOdds").is_none());
    assert_eq!(view["seats"][0]["stack"], json!([8]));
    assert_eq!(view["target"], json!(31));
    assert!(view.get("supply").is_none());
}

#[test]
fn an_oversized_target_is_capped() {
    let s = ContinuousPairs::new(&Config { target: Some(255) }, 2, 1);
    assert_eq!(s.target, 31);
}

#[test]
fn the_odds_count_the_discards_when_the_deck_is_empty() {
    // The next card comes from a reshuffle of the whole discard pile.
    let mut s = table(&[&[10], &[9]], &[]);
    s.supply = Supply::from_parts(Vec::new(), ranks(&[10, 10, 10, 3]), Vec::new(), BURN, 1);
    assert_eq!(s.expected_hit_cost(Seat(0)), 7.5);
    assert_eq!(s.expected_hit_cost(Seat(1)), 0.0);
}

#[test]
fn at_three_players_emptied_stacks_refill_on_their_own_turns() {
    let mut s = table(&[&[6], &[5], &[3]], &[7, 8, 9]);
    fold(&mut s, 0, 2, 3);
    // Seats 0 and 2 are empty, but seat 1 is next and has cards.
    assert_eq!(s.turn, Some(Seat(1)));
    assert!(s.stacks[0].is_empty() && s.stacks[2].is_empty());
    hit(&mut s, 1);
    assert_eq!(s.stacks, vec![ranks(&[9]), ranks(&[5, 7]), ranks(&[8])]);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn a_hit_on_an_empty_deck_with_one_discard_draws_it() {
    let mut s = table(&[&[5], &[9]], &[]);
    s.supply = Supply::from_parts(Vec::new(), ranks(&[2]), Vec::new(), BURN, 1);
    hit(&mut s, 0);
    assert_eq!(s.stacks[0], ranks(&[5, 2]));
}

#[test]
fn a_fold_that_reaches_the_target_loses() {
    let mut s = table(&[&[9], &[4]], &[]);
    s.target = 4;
    fold(&mut s, 0, 1, 4);
    assert_eq!(s.loser, Some(Seat(0)));
    assert_eq!(
        ContinuousPairs::outcome(&s),
        Some(Outcome::Ranked(vec![1, 0]))
    );
    let over = s.clone();
    assert!(ContinuousPairs::apply(&mut s, Seat(1), &Action::Hit).is_err());
    assert_eq!(s, over);
}
