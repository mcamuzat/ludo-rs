use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::goblin_poker::testing::{setup, table};
use pairs::goblin_poker::{Action, Config, Event, GoblinPoker, Phase, State};
use pairs::testing::{r, ranks};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn act(s: &mut State, seat: u8, action: Action) {
    GoblinPoker::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn place(card: u8) -> Action {
    Action::Place { card: r(card) }
}

fn pitch(card: u8) -> Action {
    Action::Pitch { card: r(card) }
}

fn caught(seat: u8, card: u8, paired: bool) -> Event {
    Event::Caught {
        seat: Seat(seat),
        card: r(card),
        paired,
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        GoblinPoker::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!GoblinPoker::legal_actions(s, Seat(seat)).contains(&action));
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot)
}

fn cards(s: &State) -> usize {
    s.hands.iter().chain(&s.stacks).map(Vec::len).sum::<usize>()
        + s.pitched.iter().flatten().count()
        + s.supply.deck_len()
        + s.supply.discard_len()
}

/// Three seats that each hold an 8 and a 9.
fn three(stacks: &[&[u8]], draws: &[u8]) -> State {
    table(&[&[8, 9], &[8, 9], &[8, 9]], stacks, draws)
}

fn all_pitch(s: &mut State, card: u8) {
    while s.pitched.iter().any(Option::is_none) && s.round > 0 {
        let Some(seat) = s.turn else { break };
        let before = (s.hand_no, s.round);
        act(s, seat.0, pitch(card));
        if (s.hand_no, s.round) != before {
            break;
        }
    }
}

#[test]
fn a_new_game_antes_and_deals_six_cards() {
    for players in 3..=8u8 {
        let s = GoblinPoker::new(&Config::default(), players, 3);
        assert_eq!(s.phase, Phase::Setup);
        assert!(s.hands.iter().all(|h| h.len() == 6));
        assert_eq!(s.pot, u32::from(players));
        assert!(s.chips.iter().all(|&c| c == -1));
        assert_eq!(GoblinPoker::pending(&s).0.len(), usize::from(players));
        assert_eq!((s.hand_no, s.hands_total, s.round), (1, 5, 0));
        assert_eq!(cards(&s), DECK_SIZE);
    }
}

#[test]
fn the_number_of_hands_comes_from_the_config_within_bounds() {
    let hands = |h| GoblinPoker::new(&Config { hands: h }, 3, 1).hands_total;
    assert_eq!(hands(None), 5);
    assert_eq!(hands(Some(0)), 5);
    assert_eq!(hands(Some(2)), 2);
    assert_eq!(hands(Some(255)), 15);
}

fn opening() -> State {
    setup(
        &[
            &[5, 6, 7, 8, 9, 9],
            &[2, 3, 4, 8, 9, 10],
            &[4, 4, 6, 10, 10, 10],
        ],
        &[1],
    )
}

#[test]
fn placed_cards_start_the_stacks_and_the_lowest_starts() {
    let mut s = opening();
    act(&mut s, 0, place(5));
    act(&mut s, 2, place(4));
    assert_eq!(s.phase, Phase::Setup);
    assert!(s.stacks.iter().all(Vec::is_empty));
    act(&mut s, 1, place(2));
    assert_eq!(s.stacks, vec![ranks(&[5]), ranks(&[2]), ranks(&[4])]);
    assert!(s.hands.iter().all(|h| h.len() == 5));
    assert_eq!((s.phase, s.round, s.turn), (Phase::Play, 1, Some(Seat(1))));
    assert_eq!(
        s.events,
        vec![
            Event::Placed {
                seat: Seat(0),
                card: r(5)
            },
            Event::Placed {
                seat: Seat(1),
                card: r(2)
            },
            Event::Placed {
                seat: Seat(2),
                card: r(4)
            },
            Event::Starts { seat: Seat(1) },
        ]
    );
}

#[test]
fn setup_shows_each_seat_only_itself_pending() {
    let mut s = opening();
    act(&mut s, 0, place(5));
    assert_eq!(GoblinPoker::pending(&s), Pending(vec![Seat(1), Seat(2)]));
    let visible = |v| GoblinPoker::visible_pending(&s, v);
    assert_eq!(visible(Viewer::Seat(Seat(0))), Pending::none());
    assert_eq!(visible(Viewer::Seat(Seat(1))), Pending::one(Seat(1)));
    assert_eq!(visible(Viewer::Spectator), Pending::none());
    assert_eq!(
        GoblinPoker::view(&s, Viewer::Seat(Seat(0))).placed,
        Some(r(5))
    );
    let other = GoblinPoker::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(other.placed, None);
    assert!(other.seats.iter().all(|seat| seat.stack.is_empty()));
    refused(&s, 0, place(6));
}

#[test]
fn the_turn_goes_left_from_the_lowest_stack() {
    let mut s = three(&[&[5], &[2], &[7]], &[]);
    act(&mut s, 0, pitch(8));
    assert_eq!(s.turn, Some(Seat(1)));
    act(&mut s, 1, pitch(8));
    assert_eq!(s.turn, Some(Seat(2)));
    act(&mut s, 2, pitch(8));
    // Round 2 starts with the 2.
    assert_eq!((s.round, s.turn), (2, Some(Seat(1))));
    act(&mut s, 1, pitch(9));
    assert_eq!(s.turn, Some(Seat(2)));
    act(&mut s, 2, pitch(9));
    assert_eq!(s.turn, Some(Seat(0)));
}

#[test]
fn folding_costs_the_lowest_card_in_play() {
    let mut s = three(&[&[5], &[3], &[7]], &[]);
    assert_eq!(s.fold_cost(), Some(3));
    act(&mut s, 0, Action::Fold);
    assert_eq!((s.chips[0], s.pot), (-4, 6));
    assert!(s.out[0] && s.stacks[0].is_empty() && s.hands[0].is_empty());
    assert_eq!(
        s.events,
        vec![Event::Folded {
            seat: Seat(0),
            cost: 3
        }]
    );
    assert_eq!(s.supply.seen(), ranks(&[5]).as_slice());
    assert_eq!(s.supply.discard_len(), 3);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(balance(&s), 0);
    // The folder's own card counts.
    act(&mut s, 1, Action::Fold);
    assert_eq!(
        s.events[0],
        Event::Folded {
            seat: Seat(1),
            cost: 3
        }
    );
}

#[test]
fn the_last_player_in_takes_the_pot_at_once() {
    let mut s = three(&[&[5], &[3], &[7]], &[]);
    act(&mut s, 0, Action::Fold);
    act(&mut s, 1, Action::Fold);
    // Seat 2 never had to act.
    assert_eq!(s.chips, vec![-5, -5, 7]);
    assert!(s.events.contains(&Event::HandEnded {
        winners: vec![Seat(2)],
        share: 9
    }));
    assert_eq!((s.hand_no, s.phase, s.pot), (2, Phase::Setup, 3));
    assert_eq!(balance(&s), 0);

    // A winner that had already pitched: its card goes to the discards.
    let mut s = three(&[&[5], &[3], &[7]], &[]);
    s.hands_total = 1;
    act(&mut s, 0, pitch(8));
    act(&mut s, 1, Action::Fold);
    act(&mut s, 2, Action::Fold);
    // Seat 2 pays the 5: the 3 left with seat 1.
    assert_eq!(s.chips, vec![10, -4, -6]);
    assert_eq!(s.phase, Phase::Over);
    assert!(s.pitched.iter().all(Option::is_none));
    assert_eq!(cards(&s), 9);
}

#[test]
fn pitched_cards_are_dealt_face_up_from_the_lowest_stack() {
    let mut s = three(&[&[5], &[2], &[7]], &[]);
    all_pitch(&mut s, 8);
    assert_eq!(
        s.stacks,
        vec![ranks(&[5, 8]), ranks(&[2, 8]), ranks(&[7, 8])]
    );
    assert!(s.hands.iter().all(|h| h == &ranks(&[9])));
    assert!(s.pitched.iter().all(Option::is_none));
    assert_eq!(
        s.events,
        vec![
            Event::Pitched { seat: Seat(2) },
            caught(1, 8, false),
            caught(2, 8, false),
            caught(0, 8, false),
            Event::Starts { seat: Seat(1) },
        ]
    );
}

#[test]
fn a_pair_costs_its_rank_and_puts_the_player_out() {
    let mut s = three(&[&[8], &[2], &[7]], &[]);
    all_pitch(&mut s, 8);
    assert!(s.events.contains(&caught(0, 8, true)));
    assert!(s.out[0] && s.stacks[0].is_empty() && s.hands[0].is_empty());
    assert_eq!((s.chips[0], s.pot), (-9, 11));
    assert_eq!((s.round, s.turn), (2, Some(Seat(1))));
    assert_eq!(balance(&s), 0);
    assert_eq!(cards(&s), 9);
}

#[test]
fn the_last_survivor_wins_before_catching_a_pair() {
    // Dealt in the order 2, 0, 1: seats 2 and 0 pair, seat 1 is alone.
    let mut s = three(&[&[8], &[8], &[8, 1]], &[]);
    all_pitch(&mut s, 8);
    assert_eq!(s.chips, vec![-9 - 1, 18 - 1, -9 - 1]);
    assert!(s.events.contains(&Event::HandEnded {
        winners: vec![Seat(1)],
        share: 19
    }));
    assert!(!s
        .events
        .iter()
        .any(|e| matches!(e, Event::Caught { seat: Seat(1), .. })));
    assert_eq!(s.hand_no, 2);
    assert_eq!(balance(&s), 0);
}

#[test]
fn an_extra_card_makes_a_stack_lower() {
    let mut s = three(&[&[2, 3], &[2, 3, 6], &[4]], &[]);
    all_pitch(&mut s, 9);
    assert_eq!(s.events[1], caught(1, 9, false));
    assert_eq!((s.round, s.turn), (2, Some(Seat(1))));
}

#[test]
fn a_perfect_tie_is_broken_from_the_deck() {
    let mut s = three(&[&[2], &[2], &[7]], &[2, 5, 3]);
    all_pitch(&mut s, 9);
    let tie = |seat, card, kept| Event::TieBreak {
        seat: Seat(seat),
        card: r(card),
        kept,
    };
    // Seat 0 draws a 2: a pair, discarded and replaced by the 5.
    assert_eq!(
        s.events[1..5],
        [
            tie(0, 2, false),
            tie(0, 5, true),
            tie(1, 3, true),
            caught(1, 9, false)
        ]
    );
    assert_eq!(
        s.stacks,
        vec![ranks(&[2, 5, 9]), ranks(&[2, 3, 9]), ranks(&[7, 9])]
    );
    assert_eq!(s.supply.seen(), ranks(&[2]).as_slice());
    assert_eq!((s.round, s.turn), (2, Some(Seat(1))));
}

#[test]
fn an_empty_deck_leaves_the_first_tied_seat() {
    let mut s = three(&[&[7], &[2], &[2]], &[]);
    all_pitch(&mut s, 9);
    assert_eq!(s.events[1], caught(1, 9, false));
    assert_eq!((s.round, s.turn), (2, Some(Seat(1))));
    assert!(!s.events.iter().any(|e| matches!(e, Event::TieBreak { .. })));

    // One card left: seat 1 takes it, and an extra card is lower.
    let mut s = three(&[&[7], &[2], &[2]], &[5]);
    all_pitch(&mut s, 9);
    assert_eq!(s.events[2], caught(1, 9, false));
    assert_eq!(s.supply.deck_len(), 0);
}

#[test]
fn survivors_split_the_pot_and_odd_chips_stay() {
    let last_round = || {
        let mut s = table(&[&[9], &[9], &[9]], &[&[5], &[2], &[7]], &[]);
        s.round = 5;
        s.pot = 10;
        s.chips = vec![-4, -3, -3];
        s
    };
    let mut s = last_round();
    all_pitch(&mut s, 9);
    assert!(s.events.contains(&Event::HandEnded {
        winners: vec![Seat(0), Seat(1), Seat(2)],
        share: 3
    }));
    // The odd chip stays for the next hand, with the new antes.
    assert_eq!((s.hand_no, s.pot), (2, 4));
    assert_eq!(s.chips, vec![-2, -1, -1]);
    assert_eq!(balance(&s), 0);

    let mut s = last_round();
    s.hands_total = 1;
    all_pitch(&mut s, 9);
    assert_eq!((s.phase, s.turn, s.pot), (Phase::Over, None, 1));
    assert_eq!(s.chips, vec![-1, 0, 0]);
    assert_eq!(
        GoblinPoker::outcome(&s),
        Some(Outcome::Ranked(vec![2, 0, 0]))
    );
    assert!(GoblinPoker::pending(&s).is_empty());
    for seat in 0..3 {
        assert!(GoblinPoker::legal_actions(&s, Seat(seat)).is_empty());
        refused(&s, seat, Action::Fold);
    }
}

#[test]
fn a_hand_lasts_five_rounds_at_most() {
    let mut s = setup(
        &[
            &[1, 4, 5, 6, 7, 8],
            &[2, 4, 5, 6, 7, 8],
            &[3, 4, 5, 6, 7, 8],
        ],
        &[],
    );
    for (seat, card) in [(0, 1), (1, 2), (2, 3)] {
        act(&mut s, seat, place(card));
    }
    for (round, card) in (1..=5).zip([4, 5, 6, 7, 8]) {
        assert_eq!((s.hand_no, s.round), (1, round));
        assert_eq!(GoblinPoker::outcome(&s), None);
        all_pitch(&mut s, card);
    }
    assert_eq!(s.hand_no, 2);
    assert_eq!(s.chips, vec![-1, -1, -1]);
}

#[test]
fn absurd_actions_are_refused() {
    let s = three(&[&[5], &[3], &[7]], &[]);
    refused(&s, 0, pitch(4));
    refused(&s, 0, place(8));
    refused(&s, 1, pitch(8));
    refused(&s, 9, Action::Fold);
    let mut pitched = s.clone();
    act(&mut pitched, 0, pitch(8));
    refused(&pitched, 0, pitch(9));
    refused(&pitched, 0, Action::Fold);

    let o = opening();
    refused(&o, 0, pitch(5));
    refused(&o, 0, Action::Fold);
    refused(&o, 0, place(1));
    refused(&o, 9, place(5));
}

#[test]
fn a_view_hides_other_hands_and_pitched_cards() {
    let mut s = three(&[&[5], &[3], &[7]], &[1, 1]);
    s.seed = 123_456_789;
    s.hands[1] = ranks(&[10, 10]);
    act(&mut s, 0, pitch(8));
    let mine = GoblinPoker::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!((mine.pitched, &mine.hand), (Some(r(8)), &ranks(&[9])));
    let theirs = GoblinPoker::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!((theirs.pitched, &theirs.hand), (None, &ranks(&[10, 10])));
    assert!(theirs.seats[0].pitched && !theirs.seats[1].pitched);
    assert_eq!(theirs.seats[0].hand_size, 1);
    assert_eq!(theirs.fold_cost, Some(3));
    let watching = GoblinPoker::view(&s, Viewer::Spectator);
    assert!(watching.hand.is_empty() && watching.pitched.is_none());
    for view in [&theirs, &watching] {
        let text = serde_json::to_string(view).unwrap();
        assert!(!text.contains("seed") && !text.contains("supply"), "{text}");
        assert!(!text.contains("123456789"));
        // Seat 0's 8 and 9 are nowhere to be read.
        assert!(!text.contains('8') && !text.contains('9'), "{text}");
    }
    let text = serde_json::to_string(&watching).unwrap();
    assert!(text.contains("\"foldCost\":3") && text.contains("\"handSize\":1"));
}

#[test]
fn actions_have_the_json_the_board_sends() {
    let json = |a: &Action| serde_json::to_string(a).unwrap();
    assert_eq!(json(&Action::Fold), "\"Fold\"");
    assert_eq!(json(&place(4)), "{\"Place\":{\"card\":4}}");
    assert_eq!(json(&pitch(4)), "{\"Pitch\":{\"card\":4}}");
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let s = GoblinPoker::new(&Config::default(), 3, 7);
    let hand: Vec<u8> = s.hands[0].iter().map(|c| c.value()).collect();
    assert_eq!(hand, vec![8, 9, 7, 6, 5, 7]);
}

#[test]
fn chips_and_cards_are_conserved() {
    let mut rng = SmallRng::seed_from_u64(11);
    for players in 3..=8u8 {
        for _ in 0..25 {
            let mut s = GoblinPoker::new(&Config::default(), players, rng.gen());
            let mut steps = 0;
            loop {
                let pending = GoblinPoker::pending(&s);
                let Some(&seat) = pending.0.choose(&mut rng) else {
                    break;
                };
                let actions = GoblinPoker::legal_actions(&s, seat);
                let action = actions.choose(&mut rng).expect("a pending seat can act");
                GoblinPoker::apply(&mut s, seat, action).unwrap();
                assert_eq!(balance(&s), 0);
                assert_eq!(cards(&s), DECK_SIZE);
                for stack in &s.stacks {
                    let mut sorted = stack.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    assert_eq!(sorted.len(), stack.len(), "a pair in a stack");
                }
                steps += 1;
                assert!(steps < 10_000, "the game does not end");
            }
            assert_eq!(s.hand_no, 5);
            assert!(GoblinPoker::outcome(&s).is_some());
        }
    }
}
