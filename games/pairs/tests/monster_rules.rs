use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::monster::testing::table;
use pairs::monster::{monster_strength, strength, Action, Config, Event, Monster, Phase, State};
use pairs::testing::ranks;

fn act(s: &mut State, seat: u8, action: Action) {
    Monster::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn discard(cards: &[u8]) -> Action {
    Action::Discard {
        cards: ranks(cards),
    }
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot)
}

fn showdown(s: &State) -> Option<(Vec<Seat>, bool)> {
    s.events.iter().find_map(|e| match e {
        Event::Showdown { champions, won, .. } => Some((champions.clone(), *won)),
        _ => None,
    })
}

#[test]
fn hands_compare_by_high_card_and_a_card_beats_nothing() {
    // [R] 10-8-7-4 beats 10-8-7; the Monster counts each rank once.
    assert!(strength(&ranks(&[10, 8, 7, 4])) > strength(&ranks(&[10, 8, 7])));
    assert!(strength(&ranks(&[9, 8, 7, 6, 5, 4])) < strength(&ranks(&[10])));
    assert_eq!(
        monster_strength(&ranks(&[9, 9, 4, 9, 2])),
        ranks(&[9, 4, 2])
    );
}

#[test]
fn a_new_game_antes_by_table_size_and_deals_six() {
    for players in 4..=8u8 {
        let s = Monster::new(&Config::default(), players, 3);
        let ante = if players <= 5 { 2 } else { 1 };
        assert_eq!(s.pot, ante * u32::from(players));
        assert!(s.hands.iter().all(|h| h.len() == 6));
        // Left of the button acts first.
        assert_eq!(s.turn, Some(Seat(1)));
        assert_eq!(balance(&s), 0);
    }
}

#[test]
fn a_kept_hand_never_holds_a_pair() {
    let s = table(&[&[1, 2, 3, 4, 5, 6], &[9, 9, 9, 9, 8, 7], &[1], &[1]], 3);
    // Four 9s need three discards to leave one: only the 9-9-9 discard works.
    assert_eq!(
        Monster::legal_actions(&s, Seat(1)),
        vec![discard(&[9, 9, 9]), Action::Fold]
    );
    let mut copy = s.clone();
    assert!(Monster::apply(&mut copy, Seat(1), &discard(&[9, 9])).is_err());
    assert_eq!(copy, s);
}

#[test]
fn a_champion_who_beats_the_monster_takes_the_pot() {
    let mut s = table(
        &[
            &[10, 9, 8, 2, 2, 3],
            &[9, 9, 4, 5, 6, 7],
            &[10, 10, 1, 2, 3, 4],
            &[8, 7, 6, 5, 4, 1],
        ],
        3,
    );
    act(&mut s, 1, discard(&[9])); // keeps 9 7 6 5 4
    act(&mut s, 2, discard(&[10])); // keeps 10 4 3 2 1
    act(&mut s, 3, discard(&[])); // keeps 8 7 6 5 4 1
    act(&mut s, 0, discard(&[2])); // keeps 10 9 8 3 2
                                   // The Monster holds 9, 10, 2: 10-9-2. Seat 0's 10-9-8-3-2 beats it.
    assert_eq!(showdown(&s), Some((vec![Seat(0)], true)));
    assert!(s.events.contains(&Event::PotWon {
        seats: vec![Seat(0)],
        share: 8
    }));
    // Game 2: the button moves, everyone antes again.
    assert_eq!(s.game_no, 2);
    assert_eq!(s.button, Seat(1));
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(s.chips, vec![-2 + 8 - 2, -4, -4, -4]);
    assert_eq!(balance(&s), 0);
}

#[test]
fn a_beaten_champion_pays_and_leaves_and_folders_leave_too() {
    let mut s = table(
        &[
            &[1, 2, 3, 4, 5, 6],
            &[10, 9, 8, 7, 6, 5],
            &[2, 3, 4, 5, 6, 7],
            &[1, 2, 3, 4, 5, 7],
        ],
        3,
    );
    act(&mut s, 1, discard(&[5, 6]));
    act(&mut s, 2, discard(&[]));
    act(&mut s, 3, Action::Fold);
    act(&mut s, 0, discard(&[1]));
    // The Monster's 6-5-1 loses to seat 1's 10-9-8-7.
    assert_eq!(showdown(&s), Some((vec![Seat(1)], true)));
    let mut s = table(
        &[
            &[1, 2, 3, 4, 5, 6],
            &[9, 8, 7, 6, 5, 4],
            &[10, 3, 4, 5, 6, 7],
            &[1, 2, 3, 4, 5, 7],
        ],
        3,
    );
    act(&mut s, 1, discard(&[]));
    act(&mut s, 2, discard(&[10]));
    act(&mut s, 3, Action::Fold);
    act(&mut s, 0, discard(&[]));
    // Seat 1's 9-8-7-6-5-4 is the best hand but the Monster's 10 beats it.
    assert_eq!(showdown(&s), Some((vec![Seat(1)], false)));
    assert!(s.events.contains(&Event::Penalty {
        seats: vec![Seat(1)]
    }));
    assert_eq!(s.chips[1], -2 - 5);
    // Seats 0 and 2 play on; 1 (beaten) and 3 (folded) are out.
    assert_eq!(s.in_game, vec![true, false, true, false]);
    assert_eq!(s.round_no, 2);
    assert!(s.hands[1].is_empty() && s.hands[3].is_empty());
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(balance(&s), 0);
}

#[test]
fn the_last_player_standing_wins_and_nobody_left_replays() {
    let mut s = table(
        &[&[1, 2, 3, 4, 5, 6], &[9, 8, 7, 6, 5, 4], &[2, 3], &[1, 2]],
        1,
    );
    act(&mut s, 1, Action::Fold);
    act(&mut s, 2, Action::Fold);
    act(&mut s, 3, Action::Fold);
    act(&mut s, 0, Action::Fold);
    // Everyone folded: the same four play again.
    assert!(s.events.contains(&Event::Replay));
    assert_eq!(s.round_no, 2);
    assert_eq!(s.in_round, vec![true; 4]);
    let mut s = table(
        &[&[1, 2, 3, 4, 5, 6], &[9, 8, 7, 6, 5, 4], &[2, 3], &[1, 2]],
        1,
    );
    s.in_round = vec![true, true, false, false];
    s.in_game = vec![true, true, false, false];
    s.hands[2].clear();
    s.hands[3].clear();
    act(&mut s, 1, discard(&[]));
    act(&mut s, 0, Action::Fold);
    // Seat 1 alone: 9-8-7-6-5-4 against an empty Monster wins.
    assert_eq!(showdown(&s), Some((vec![Seat(1)], true)));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Monster::pending(&s), Pending::none());
    assert!(matches!(Monster::outcome(&s), Some(Outcome::Ranked(_))));
}

#[test]
fn nobody_sees_another_hand() {
    let s = Monster::new(&Config::default(), 5, 7);
    let v = Monster::view(&s, Viewer::Seat(Seat(2)));
    assert_eq!(v.hand, s.hands[2]);
    assert!(Monster::view(&s, Viewer::Spectator).hand.is_empty());
}
