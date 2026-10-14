use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deadfall::testing::{door, table};
use pairs::deadfall::{hand_size, Action, Config, Deadfall, Event, Phase, State};
use pairs::deck::DECK_SIZE;
use pairs::testing::{r, ranks};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn act(s: &mut State, seat: u8, action: Action) {
    Deadfall::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn doors(s: &mut State, cards: &[u8]) {
    for (seat, &card) in cards.iter().enumerate() {
        act(s, seat as u8, Action::Door { card: r(card) });
    }
}

fn call(seat: u8) -> Action {
    Action::Call { seat: Seat(seat) }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        Deadfall::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!Deadfall::legal_actions(s, Seat(seat)).contains(&action));
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot)
}

fn cards(s: &State) -> usize {
    s.hands.iter().chain(&s.stacks).map(Vec::len).sum::<usize>()
        + s.supply.deck_len()
        + s.supply.discard_len()
}

#[test]
fn a_new_game_antes_and_deals_by_table_size() {
    for players in 2..=7u8 {
        let size = hand_size(players);
        assert_eq!(size, [7, 6, 6, 6, 6, 5][usize::from(players - 2)]);
        let s = Deadfall::new(&Config::default(), players, 3);
        assert_eq!(s.phase, Phase::Door);
        assert!(s.hands.iter().all(|h| h.len() == size));
        assert_eq!(s.pot, u32::from(players));
        assert!(s.chips.iter().all(|&c| c == -1));
        assert_eq!(Deadfall::pending(&s).0.len(), usize::from(players));
        assert_eq!(cards(&s), DECK_SIZE);
    }
}

#[test]
fn door_cards_are_chosen_in_secret() {
    let mut s = door(&[&[5, 6], &[2, 3], &[4, 9]], &[]);
    act(&mut s, 0, Action::Door { card: r(5) });
    refused(&s, 0, Action::Door { card: r(6) });
    assert_eq!(
        Deadfall::visible_pending(&s, Viewer::Seat(Seat(0))),
        Pending::none()
    );
    assert_eq!(
        Deadfall::visible_pending(&s, Viewer::Seat(Seat(1))),
        Pending::one(Seat(1))
    );
    assert_eq!(Deadfall::view(&s, Viewer::Seat(Seat(1))).placed, None);
    assert_eq!(Deadfall::view(&s, Viewer::Seat(Seat(0))).placed, Some(r(5)));
}

#[test]
fn the_lowest_door_card_goes_first() {
    let mut s = door(&[&[5, 6], &[2, 3], &[4, 9]], &[]);
    doors(&mut s, &[5, 2, 9]);
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.stacks, vec![ranks(&[5]), ranks(&[2]), ranks(&[9])]);
    assert_eq!(s.hands, vec![ranks(&[6]), ranks(&[3]), ranks(&[4])]);
    assert_eq!(
        s.events,
        vec![Event::DoorsRevealed {
            cards: ranks(&[5, 2, 9]),
            leader: Some(Seat(1)),
        }]
    );
}

#[test]
fn a_tie_is_settled_by_the_next_door_card_alone() {
    let mut s = door(&[&[3, 7, 8], &[3, 4, 8], &[6, 1, 8]], &[]);
    doors(&mut s, &[3, 3, 6]);
    assert_eq!(s.phase, Phase::Door);
    assert!(matches!(
        s.events[..],
        [Event::DoorsRevealed { leader: None, .. }]
    ));
    // The second card decides, whatever the first: seat 2's 1.
    doors(&mut s, &[7, 4, 1]);
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn earlier_door_cards_break_a_tie_on_the_latest() {
    let mut s = door(&[&[2, 4, 8], &[2, 6, 8], &[5, 4, 8]], &[]);
    doors(&mut s, &[2, 2, 5]);
    doors(&mut s, &[4, 6, 4]);
    assert_eq!(s.turn, Some(Seat(0)));
}

/// The example of the book: A leads with the 2; B's and D's 5s are both
/// on top, and D's counts as played later.
#[test]
fn of_equal_top_cards_only_the_latest_may_be_called() {
    let hands: Vec<&[u8]> = vec![&[1], &[1], &[1], &[1], &[1], &[1]];
    let mut s = door(&hands, &[]);
    s.hands = vec![
        ranks(&[2, 3]),
        ranks(&[5, 3]),
        ranks(&[9, 3]),
        ranks(&[5, 3]),
        ranks(&[4, 3]),
        ranks(&[6, 3]),
    ];
    doors(&mut s, &[2, 5, 9, 5, 4, 6]);
    assert_eq!(s.turn, Some(Seat(0)));
    assert_eq!(
        s.callable(Seat(0)),
        vec![Seat(2), Seat(3), Seat(4), Seat(5)]
    );
    refused(&s, 0, call(1));
    refused(&s, 0, call(0));
    // Once B plays again, its 5 is the latest.
    act(&mut s, 0, Action::Play { card: r(3) });
    act(&mut s, 1, Action::Play { card: r(3) });
    assert!(s.callable(Seat(2)).contains(&Seat(3)));
    let mut later = s.clone();
    later.stacks[1] = ranks(&[5, 3, 5]);
    later.stamps[1] = later.clock + 1;
    assert!(!later.callable(Seat(2)).contains(&Seat(3)));
    assert!(later.callable(Seat(2)).contains(&Seat(1)));
}

#[test]
fn a_played_card_goes_on_top_and_the_turn_passes_left() {
    let mut s = table(&[&[4, 7], &[1], &[2]], &[&[6], &[10], &[9]], &[]);
    act(&mut s, 0, Action::Play { card: r(7) });
    assert_eq!(s.stacks[0], ranks(&[6, 7]));
    assert_eq!(s.hands[0], ranks(&[4]));
    assert_eq!(s.turn, Some(Seat(1)));
    refused(&s, 0, Action::Play { card: r(4) });
    refused(&s, 1, Action::Play { card: r(5) });
}

#[test]
fn an_empty_hand_can_only_call() {
    let s = table(&[&[], &[1], &[2]], &[&[6], &[10], &[9]], &[]);
    assert_eq!(Deadfall::legal_actions(&s, Seat(0)), vec![call(1), call(2)]);
}

#[test]
fn a_right_call_wins_the_rank_and_the_antes() {
    let mut s = table(&[&[4], &[1], &[2]], &[&[6], &[10], &[9]], &[3, 10]);
    act(&mut s, 0, call(1));
    // 10s in the undealt cards do not count: dead. Seat 1 pays 10.
    assert_eq!(s.chips[..2], [-1 + 10 + 3 - 1, -1 - 10 - 1]);
    let Some(Event::Showdown {
        dead,
        winner,
        paid,
        pot,
        hands,
        ..
    }) = s.events.first()
    else {
        panic!("{:?}", s.events)
    };
    assert_eq!((*dead, *winner, *paid, *pot), (true, Seat(0), 10, 3));
    assert_eq!(hands, &vec![ranks(&[4]), ranks(&[1]), ranks(&[2])]);
    // Hand 2 has begun: everyone anted again.
    assert_eq!((s.hand_no, s.phase, s.pot), (2, Phase::Door, 3));
    assert_eq!(balance(&s), 0);
}

#[test]
fn a_wrong_call_pays_the_called_player() {
    let mut s = table(&[&[4], &[1], &[9]], &[&[6], &[10], &[9]], &[]);
    act(&mut s, 0, call(2));
    assert!(matches!(
        s.events[0],
        Event::Showdown {
            dead: false,
            winner: Seat(2),
            paid: 9,
            ..
        }
    ));
    assert_eq!(s.chips, vec![-1 - 9 - 1, -1 - 1, -1 + 9 + 3 - 1]);
}

#[test]
fn the_callers_own_hand_counts() {
    let mut s = table(&[&[9], &[1], &[2]], &[&[6], &[10], &[9]], &[]);
    act(&mut s, 0, call(2));
    assert!(matches!(
        s.events[0],
        Event::Showdown {
            dead: false,
            winner: Seat(2),
            ..
        }
    ));
}

#[test]
fn the_game_ends_after_the_last_hand_ranked_by_chips() {
    let mut s = table(&[&[4], &[1], &[2]], &[&[6], &[10], &[9]], &[]);
    s.hand_no = s.hands_total;
    s.chips = vec![-1, 4, -6];
    s.pot = 3;
    act(&mut s, 0, call(1));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.chips, vec![12, -6, -6]);
    assert!(Deadfall::pending(&s).is_empty());
    assert_eq!(Deadfall::outcome(&s), Some(Outcome::Ranked(vec![0, 1, 1])));
    refused(&s, 1, call(0));
}

#[test]
fn a_view_shows_my_hand_only() {
    let s = Deadfall::new(&Config::default(), 3, 4);
    let mine = Deadfall::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(mine.hand, s.hands[1]);
    assert!(mine.seats.iter().all(|seat| seat.hand_size == 6));
    let json = serde_json::to_string(&mine).unwrap();
    assert!(!json.contains("seed"));
    assert!(Deadfall::view(&s, Viewer::Spectator).hand.is_empty());
}

#[test]
fn random_games_keep_every_card_and_every_chip() {
    let mut rng = SmallRng::seed_from_u64(5);
    for players in 2..=7u8 {
        for _ in 0..20 {
            let mut s = Deadfall::new(&Config::default(), players, rng.gen());
            let mut steps = 0;
            loop {
                let pending = Deadfall::pending(&s);
                let Some(&seat) = pending.0.choose(&mut rng) else {
                    break;
                };
                let action = Deadfall::legal_actions(&s, seat)
                    .choose(&mut rng)
                    .cloned()
                    .unwrap();
                act(&mut s, seat.0, action);
                assert_eq!(cards(&s), DECK_SIZE);
                assert_eq!(balance(&s), 0);
                steps += 1;
                assert!(steps < 1_000);
            }
            assert_eq!(s.hand_no, 5);
            assert_eq!(s.pot, 0);
        }
    }
}
