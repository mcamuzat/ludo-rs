use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::testing::{r, ranks};
use pairs::to_the_moon::testing::table;
use pairs::to_the_moon::{hand_size, Action, Config, Event, Phase, State, ToTheMoon};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn act(s: &mut State, seat: u8, action: Action) {
    ToTheMoon::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn build(card: u8) -> Action {
    Action::Build { card: r(card) }
}

fn withdraw(card: u8) -> Action {
    Action::Withdraw {
        take: Some(r(card)),
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        ToTheMoon::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!ToTheMoon::legal_actions(s, Seat(seat)).contains(&action));
}

fn cards(s: &State) -> usize {
    s.hands.iter().map(Vec::len).sum::<usize>()
        + s.trick.len()
        + s.supply.deck_len()
        + s.supply.discard_len()
}

#[test]
fn a_new_round_deals_by_table_size_and_the_dealer_leads() {
    for (players, size) in [(2u8, 8usize), (3, 7), (4, 6), (5, 5), (6, 4)] {
        assert_eq!(hand_size(players), size);
        let s = ToTheMoon::new(&Config::default(), players, 9);
        assert!(s.hands.iter().all(|h| h.len() == size));
        assert_eq!(s.trick.len(), 1);
        assert_eq!(
            (s.dealer, s.leader, s.turn),
            (Seat(0), Seat(0), Some(Seat(1)))
        );
        assert_eq!((s.round_no, s.rounds_total), (1, 6));
        assert_eq!(cards(&s), DECK_SIZE);
        assert_eq!(
            s.supply.deck_len(),
            DECK_SIZE - size * usize::from(players) - 1
        );
    }
}

#[test]
fn a_build_needs_a_rank_not_in_the_countdown() {
    let mut s = table(&[&[1], &[3, 5, 5], &[2]], &[3], &[9]);
    let legal = ToTheMoon::legal_actions(&s, Seat(1));
    assert!(legal.contains(&build(5)));
    refused(&s, 1, build(3));
    refused(&s, 1, build(4));
    act(&mut s, 1, build(5));
    assert_eq!(s.trick, ranks(&[3, 5]));
    assert_eq!(s.hands[1], ranks(&[3, 5]));
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn only_the_seat_to_move_may_act() {
    let s = table(&[&[1], &[2], &[4]], &[3], &[9]);
    refused(&s, 0, build(1));
    refused(&s, 2, build(4));
    refused(&s, 7, Action::Test);
}

#[test]
fn a_perfect_countdown_wins_at_once() {
    let mut s = table(&[&[1], &[10], &[2]], &[1, 2, 3, 4, 5, 6, 7, 8, 9], &[4, 6]);
    act(&mut s, 1, build(10));
    assert_eq!(s.scores, vec![0, 10, 0]);
    assert!(s.events.contains(&Event::TrickWon {
        seat: Seat(1),
        cards: 10,
        perfect: true,
    }));
    // Seat 1 leads the next countdown, everyone back in.
    assert_eq!((s.leader, s.trick.clone()), (Seat(1), ranks(&[4])));
    assert_eq!(s.out, vec![false; 3]);
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn a_test_that_completes_the_countdown_wins_too() {
    let mut s = table(&[&[1], &[2], &[3]], &[1, 2, 3, 4, 5, 6, 7, 8, 9], &[10, 4]);
    act(&mut s, 1, Action::Test);
    assert_eq!(s.scores, vec![0, 10, 0]);
}

#[test]
fn a_good_test_stays_in() {
    let mut s = table(&[&[1], &[2], &[3]], &[5], &[7]);
    act(&mut s, 1, Action::Test);
    assert_eq!(s.trick, ranks(&[5, 7]));
    assert!(!s.out[1]);
    assert_eq!(s.hands[1], ranks(&[2]));
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn a_matching_test_knocks_out_with_the_card_in_hand() {
    let mut s = table(&[&[1], &[2], &[3]], &[5, 7], &[7]);
    act(&mut s, 1, Action::Test);
    assert!(s.out[1]);
    assert_eq!(s.hands[1], ranks(&[2, 7]));
    assert_eq!(s.known[1], ranks(&[7]));
    assert_eq!(s.trick, ranks(&[5, 7]));
    assert_eq!(s.turn, Some(Seat(2)));
    let view = ToTheMoon::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(view.seats[1].known, ranks(&[7]));
    assert_eq!(view.seats[1].hand_size, 2);
}

#[test]
fn no_test_from_an_empty_deck() {
    let s = table(&[&[1], &[2], &[3]], &[5], &[]);
    refused(&s, 1, Action::Test);
}

#[test]
fn withdrawing_takes_a_card_unless_it_is_the_last_one() {
    let mut s = table(&[&[1], &[2], &[3]], &[5, 8], &[9]);
    refused(&s, 1, Action::Withdraw { take: None });
    refused(&s, 1, withdraw(4));
    act(&mut s, 1, withdraw(8));
    assert!(s.out[1]);
    assert_eq!(s.hands[1], ranks(&[2, 8]));
    assert_eq!(s.known[1], ranks(&[8]));
    assert_eq!(s.trick, ranks(&[5]));
    assert_eq!(s.turn, Some(Seat(2)));
    // One card left on the table: seat 2 withdraws empty-handed.
    refused(&s, 2, withdraw(5));
    act(&mut s, 2, Action::Withdraw { take: None });
    assert_eq!(s.hands[2], ranks(&[3]));
}

#[test]
fn the_last_player_in_takes_the_trick_and_leads() {
    let mut s = table(&[&[1], &[2], &[3]], &[5, 8], &[9, 6]);
    act(&mut s, 1, withdraw(8));
    act(&mut s, 2, Action::Withdraw { take: None });
    // Only seat 0 is still in: one point for the 5 left.
    assert_eq!(s.scores, vec![1, 0, 0]);
    assert!(s.events.contains(&Event::TrickWon {
        seat: Seat(0),
        cards: 1,
        perfect: false,
    }));
    assert_eq!(s.leader, Seat(0));
    assert_eq!(s.trick, ranks(&[9]));
    assert_eq!(s.out, vec![false; 3]);
    assert_eq!(s.turn, Some(Seat(1)));
    // A card a player plays is no longer known to be in its hand.
    act(&mut s, 1, build(8));
    assert!(s.known[1].is_empty());
}

#[test]
fn players_who_are_out_are_skipped() {
    let mut s = table(&[&[1], &[2], &[3], &[4]], &[5, 8], &[9, 9, 9]);
    act(&mut s, 1, withdraw(8));
    act(&mut s, 2, Action::Test);
    assert_eq!(s.turn, Some(Seat(3)));
    act(&mut s, 3, build(4));
    assert_eq!(s.turn, Some(Seat(0)));
    act(&mut s, 0, build(1));
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn an_empty_deck_ends_the_round_and_the_deal_passes_left() {
    let mut s = table(&[&[1], &[2]], &[5], &[]);
    act(&mut s, 1, Action::Withdraw { take: None });
    assert_eq!(s.scores, vec![1, 0]);
    assert!(s.events.contains(&Event::RoundEnded { round: 1 }));
    assert_eq!((s.round_no, s.dealer, s.leader), (2, Seat(1), Seat(1)));
    assert_eq!(s.turn, Some(Seat(0)));
    assert!(s.hands.iter().all(|h| h.len() == 8));
    assert!(s.known.iter().all(Vec::is_empty));
    assert_eq!(cards(&s), DECK_SIZE);
}

#[test]
fn the_game_ends_after_the_last_round_ranked_by_score() {
    let mut s = table(&[&[1], &[2], &[3]], &[5], &[]);
    s.round_no = s.rounds_total;
    s.scores = vec![10, 13, 12];
    act(&mut s, 1, Action::Withdraw { take: None });
    act(&mut s, 2, Action::Withdraw { take: None });
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.scores, vec![11, 13, 12]);
    assert!(ToTheMoon::pending(&s).is_empty());
    assert_eq!(ToTheMoon::outcome(&s), Some(Outcome::Ranked(vec![2, 0, 1])));
    refused(&s, 0, build(1));
}

#[test]
fn equal_scores_stay_tied() {
    let mut s = table(&[&[1], &[2]], &[5], &[]);
    s.round_no = s.rounds_total;
    s.scores = vec![9, 10];
    act(&mut s, 1, Action::Withdraw { take: None });
    assert_eq!(ToTheMoon::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn a_view_shows_my_hand_only() {
    let s = ToTheMoon::new(&Config::default(), 3, 4);
    let mine = ToTheMoon::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(mine.hand, s.hands[1]);
    assert_eq!(mine.trick, s.trick);
    assert!(mine.seats.iter().all(|seat| seat.hand_size == 7));
    let json = serde_json::to_string(&mine).unwrap();
    assert!(!json.contains("seed"));
    assert!(ToTheMoon::view(&s, Viewer::Spectator).hand.is_empty());
}

#[test]
fn random_games_keep_every_card_and_end() {
    let mut rng = SmallRng::seed_from_u64(5);
    for players in 2..=6u8 {
        for _ in 0..10 {
            let mut s = ToTheMoon::new(&Config::default(), players, rng.gen());
            let mut steps = 0;
            while let Some(&seat) = ToTheMoon::pending(&s).0.first() {
                let action = ToTheMoon::legal_actions(&s, seat)
                    .choose(&mut rng)
                    .cloned()
                    .unwrap();
                act(&mut s, seat.0, action);
                assert_eq!(cards(&s), DECK_SIZE);
                assert!(s.trick.len() <= 10);
                steps += 1;
                assert!(steps < 5_000);
            }
            assert_eq!(s.round_no, 6);
            // Every card captured counts one point; a round deals all 55.
            assert!(s.scores.iter().sum::<u32>() <= 6 * DECK_SIZE as u32);
        }
    }
}
