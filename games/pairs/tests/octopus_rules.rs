use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::octopus::testing::table;
use pairs::octopus::{strength, Action, Config, Event, Kind, Octopus, Phase, State};
use pairs::testing::{r, ranks};

fn act(s: &mut State, seat: u8, action: Action) {
    Octopus::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn plays(s: &mut State, cards: &[u8]) {
    for (seat, &card) in cards.iter().enumerate() {
        act(s, seat as u8, Action::Play { card: r(card) });
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        Octopus::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot)
}

fn cards(s: &State) -> usize {
    s.hands.iter().map(Vec::len).sum::<usize>() + s.undealt.len() + s.played.len()
}

fn showdown(s: &State) -> Option<&Event> {
    s.events
        .iter()
        .find(|e| matches!(e, Event::Showdown { .. }))
}

#[test]
fn hand_kinds() {
    assert_eq!(Kind::of(&ranks(&[9, 9, 8, 8, 8, 3, 3])), Kind::Set);
    assert_eq!(Kind::of(&ranks(&[3, 4, 5, 7, 9])), Kind::Clean);
    assert_eq!(Kind::of(&ranks(&[3, 3, 5])), Kind::Dirty);
    assert_eq!(Kind::of(&ranks(&[6])), Kind::Clean);
}

#[test]
fn comparing_hands() {
    // [R] 999-44 is worse than 99-888.
    assert!(
        strength(&ranks(&[9, 9, 9, 4, 4]), Kind::Set)
            > strength(&ranks(&[9, 9, 8, 8, 8]), Kind::Set)
    );
    // High cards are bad within a kind.
    assert!(strength(&ranks(&[2, 4, 9]), Kind::Clean) > strength(&ranks(&[6, 7, 8]), Kind::Clean));
    // The chosen kind beats the other one, and dirty is always last.
    let set = ranks(&[2, 2, 3, 3]);
    let clean = ranks(&[1, 2, 3, 4]);
    let dirty = ranks(&[1, 1, 2, 3]);
    assert!(strength(&set, Kind::Set) < strength(&clean, Kind::Set));
    assert!(strength(&clean, Kind::Clean) < strength(&set, Kind::Clean));
    assert!(strength(&clean, Kind::Set) < strength(&dirty, Kind::Set));
    assert!(strength(&set, Kind::Clean) < strength(&dirty, Kind::Clean));
}

#[test]
fn the_first_hand_antes_two_and_deals_eight() {
    for players in 4..=6u8 {
        let s = Octopus::new(&Config::default(), players, 3);
        assert!(s.hands.iter().all(|h| h.len() == 8));
        assert_eq!(s.pot, 2 * u32::from(players));
        assert_eq!(s.phase, Phase::Declare);
        assert_eq!(Octopus::pending(&s), Pending::one(Seat(0)));
        assert_eq!(cards(&s), DECK_SIZE);
        assert_eq!(balance(&s), 0);
    }
}

#[test]
fn declaring_goes_once_around_and_folding_costs_a_chip() {
    let mut s = Octopus::new(&Config::default(), 4, 3);
    act(&mut s, 0, Action::Stay);
    act(&mut s, 1, Action::Fold);
    assert_eq!(s.chips[1], -3);
    assert_eq!(s.pot, 9);
    act(&mut s, 2, Action::Stay);
    act(&mut s, 3, Action::Stay);
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(
        Octopus::pending(&s),
        Pending(vec![Seat(0), Seat(2), Seat(3)])
    );
    assert!(Octopus::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn the_last_one_in_takes_the_pot_and_everyone_antes_two_again() {
    let mut s = Octopus::new(&Config::default(), 4, 3);
    act(&mut s, 0, Action::Fold);
    act(&mut s, 1, Action::Stay);
    act(&mut s, 2, Action::Fold);
    act(&mut s, 3, Action::Fold);
    // Seat 1 takes 8 antes + 3 folds; it leads hand 2, where all ante 2.
    assert!(s.events.contains(&Event::FoldedOut {
        seat: Seat(1),
        pot: 11
    }));
    assert_eq!(s.hand_no, 2);
    assert_eq!(s.leader, Seat(1));
    assert_eq!(s.chips, vec![-5, 7, -5, -5]);
    assert_eq!(s.pot, 8);
    assert_eq!(Octopus::pending(&s), Pending::one(Seat(1)));
    assert_eq!(balance(&s), 0);
}

#[test]
fn played_cards_are_secret_until_all_are_in() {
    let mut s = table(&[&[1, 5, 6], &[2, 5, 7], &[3, 6, 8], &[4, 7, 9]], &[]);
    act(&mut s, 0, Action::Play { card: r(5) });
    refused(&s, 0, Action::Play { card: r(6) });
    assert_eq!(
        Octopus::visible_pending(&s, Viewer::Seat(Seat(0))),
        Pending::none()
    );
    assert_eq!(
        Octopus::visible_pending(&s, Viewer::Seat(Seat(1))),
        Pending::one(Seat(1))
    );
    assert_eq!(Octopus::view(&s, Viewer::Seat(Seat(1))).placed, None);
    // 5 and 5 tie; the 6 is the lowest unique card: seat 2 calls.
    act(&mut s, 1, Action::Play { card: r(5) });
    act(&mut s, 2, Action::Play { card: r(6) });
    act(&mut s, 3, Action::Play { card: r(7) });
    assert_eq!(s.phase, Phase::Call);
    assert_eq!(Octopus::pending(&s), Pending::one(Seat(2)));
    assert_eq!(s.played, ranks(&[5, 5, 6, 7]));
    assert!(s.hands.iter().all(|h| h.len() == 2));
}

#[test]
fn without_a_unique_card_another_round_is_played() {
    let mut s = table(&[&[5, 1, 9], &[5, 2, 8], &[6, 3, 7], &[6, 4, 10]], &[]);
    plays(&mut s, &[5, 5, 6, 6]);
    assert_eq!(s.phase, Phase::Play);
    assert!(s.events.contains(&Event::CardsPlayed {
        cards: vec![Some(r(5)), Some(r(5)), Some(r(6)), Some(r(6))],
        caller: None,
    }));
}

#[test]
fn a_pass_goes_to_the_next_player_still_in() {
    let mut s = table(&[&[1, 5, 6], &[2, 5, 7], &[3, 6, 8], &[4, 7, 9]], &[]);
    s.folded[1] = true;
    s.hands[1].clear();
    // The 1 calls for a pass.
    act(&mut s, 0, Action::Play { card: r(1) });
    act(&mut s, 2, Action::Play { card: r(3) });
    act(&mut s, 3, Action::Play { card: r(4) });
    act(&mut s, 0, Action::CallPass);
    assert_eq!(s.phase, Phase::Pass);
    act(&mut s, 0, Action::Pass { card: r(6) });
    act(&mut s, 2, Action::Pass { card: r(8) });
    act(&mut s, 3, Action::Pass { card: r(9) });
    // Seat 0 → seat 2 (seat 1 folded), 2 → 3, 3 → 0.
    assert!(s.hands[2].contains(&r(6)));
    assert!(s.hands[3].contains(&r(8)));
    assert!(s.hands[0].contains(&r(9)));
    let v = Octopus::view(&s, Viewer::Seat(Seat(2)));
    assert_eq!((v.passed, v.received), (Some(r(8)), Some(r(6))));
    assert_eq!(s.phase, Phase::Play);
}

#[test]
fn the_caller_picks_the_best_kind_and_the_worst_hand_pays_next_time() {
    // After this round: seat 0 holds 2-2-3-3 (set), seat 1 3-4-6-8
    // (clean), seat 2 2-4-5-6 (clean), seat 3 7-7-8-9 (dirty).
    let mut s = table(
        &[
            &[2, 2, 3, 3, 1],
            &[3, 4, 6, 8, 5],
            &[2, 4, 5, 6, 9],
            &[7, 7, 8, 9, 10],
        ],
        &[],
    );
    plays(&mut s, &[1, 5, 9, 10]);
    // The 1 is the lowest unique card: seat 0 calls a showdown of sets.
    assert_eq!(Octopus::pending(&s), Pending::one(Seat(0)));
    act(&mut s, 0, Action::Show { best: Kind::Set });
    let Some(Event::Showdown {
        winners,
        losers,
        share,
        ..
    }) = showdown(&s).cloned()
    else {
        panic!("{:?}", s.events);
    };
    assert_eq!((winners, losers, share), (vec![Seat(0)], vec![Seat(3)], 8));
    // Hand 2: seat 0 leads, seat 3 antes 5, the others 1.
    assert_eq!(s.hand_no, 2);
    assert_eq!(s.leader, Seat(0));
    assert_eq!(s.pot, 8);
    assert_eq!(s.chips, vec![-2 + 8 - 1, -3, -3, -7]);
    assert_eq!(balance(&s), 0);
}

#[test]
fn clean_hands_win_when_the_caller_says_so() {
    let mut s = table(
        &[
            &[2, 2, 3, 3, 1],
            &[3, 4, 6, 8, 5],
            &[2, 4, 5, 6, 9],
            &[7, 7, 8, 9, 10],
        ],
        &[],
    );
    plays(&mut s, &[1, 5, 9, 10]);
    act(&mut s, 0, Action::Show { best: Kind::Clean });
    let Some(Event::Showdown {
        winners, losers, ..
    }) = showdown(&s).cloned()
    else {
        panic!("{:?}", s.events);
    };
    // 2-4-5-6 beats 3-4-6-8; dirty 7-7-8-9 is still the worst.
    assert_eq!((winners, losers), (vec![Seat(2)], vec![Seat(3)]));
    assert_eq!(s.leader, Seat(2));
}

#[test]
fn tied_best_hands_split_the_pot() {
    let mut s = table(&[&[1, 2], &[3, 2], &[5, 9], &[6, 10]], &[]);
    // Hands down to one card: immediate showdown, the 2s tie for best.
    plays(&mut s, &[1, 3, 5, 6]);
    let Some(Event::Showdown {
        caller,
        best,
        winners,
        losers,
        share,
        ..
    }) = showdown(&s).cloned()
    else {
        panic!("{:?}", s.events);
    };
    assert_eq!((caller, best), (Some(Seat(0)), None));
    assert_eq!(
        (winners, losers, share),
        (vec![Seat(0), Seat(1)], vec![Seat(3)], 4)
    );
    assert_eq!(balance(&s), 0);
}

#[test]
fn on_the_last_hand_the_loser_pays_the_winner() {
    let mut s = table(&[&[1, 2], &[3, 4], &[5, 9], &[6, 10]], &[]);
    s.hands_total = 1;
    plays(&mut s, &[1, 3, 5, 6]);
    assert_eq!(s.phase, Phase::Over);
    // Seat 0 takes the 8 antes and seat 3's 5.
    assert_eq!(s.chips, vec![11, -2, -2, -7]);
    assert_eq!(s.pot, 0);
    assert_eq!(Octopus::pending(&s), Pending::none());
    assert_eq!(
        Octopus::outcome(&s),
        Some(Outcome::Ranked(vec![0, 1, 1, 3]))
    );
}

#[test]
fn a_whole_game_keeps_cards_and_chips() {
    for players in 4..=6u8 {
        let mut s = Octopus::new(&Config::default(), players, 13);
        while let Some(&seat) = Octopus::pending(&s).0.first() {
            let action = Octopus::legal_actions(&s, seat)[0].clone();
            act(&mut s, seat.0, action);
            assert_eq!(balance(&s), 0);
            if s.phase != Phase::Over {
                assert_eq!(cards(&s), DECK_SIZE);
            }
        }
        assert_eq!(s.hand_no, 5);
    }
}
