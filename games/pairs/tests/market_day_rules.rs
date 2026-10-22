use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::market_day::testing::table;
use pairs::market_day::{
    hand_size, hands_in_game, resolve, Action, Config, Event, Fate, MarketDay, Phase, State,
};
use pairs::testing::{r, ranks};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use Fate::{Killed, Scored, Spoiled};

fn act(s: &mut State, seat: u8, action: Action) {
    MarketDay::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn play(s: &mut State, seat: u8, card: u8) {
    act(s, seat, Action::Play { card: r(card) });
}

fn fates(cards: &[u8]) -> Vec<Fate> {
    resolve(&ranks(cards))
}

fn trick_scored(s: &State) -> (Vec<(Seat, pairs::deck::Rank, Fate)>, Vec<u16>) {
    s.events
        .iter()
        .find_map(|e| match e {
            Event::TrickScored { cards, gained, .. } => Some((cards.clone(), gained.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no TrickScored in {:?}", s.events))
}

#[test]
fn a_repeated_rank_scores_only_the_last() {
    assert_eq!(fates(&[8, 5, 8]), [Spoiled, Scored, Scored]);
}

#[test]
fn the_first_bandit_scores() {
    assert_eq!(fates(&[9, 4, 9, 9]), [Scored, Scored, Spoiled, Spoiled]);
}

#[test]
fn a_lone_rain_spoils_all_but_royals() {
    assert_eq!(
        fates(&[1, 2, 3, 7, 10]),
        [Spoiled, Spoiled, Scored, Scored, Spoiled]
    );
}

#[test]
fn two_rains_are_ordinary_sevens() {
    assert_eq!(fates(&[7, 10, 7]), [Spoiled, Scored, Scored]);
}

#[test]
fn royals_always_score() {
    assert_eq!(fates(&[3, 3, 3]), [Scored, Scored, Scored]);
    assert_eq!(fates(&[3, 2, 7]), [Scored, Spoiled, Scored]);
}

#[test]
fn knights_kill_the_highest_scoring_card_above_three() {
    assert_eq!(fates(&[10, 2, 8]), [Killed, Scored, Scored]);
    assert_eq!(fates(&[3, 2, 1]), [Scored, Scored, Scored]);
    assert_eq!(fates(&[10, 10, 2, 6]), [Spoiled, Killed, Scored, Scored]);
}

#[test]
fn two_knights_kill_two_cards() {
    assert_eq!(
        fates(&[10, 2, 8, 2, 5]),
        [Killed, Scored, Killed, Scored, Scored]
    );
}

#[test]
fn resolution_order_spoiled_royal_rain_knight_tinker() {
    // The lone Rain spoils the Knight and the Tinker before they act.
    assert_eq!(
        fates(&[1, 3, 7, 2, 10]),
        [Spoiled, Scored, Scored, Spoiled, Spoiled]
    );
    let mut s = table(&[&[1], &[3], &[7], &[10]]);
    for (seat, card) in [(0, 1), (1, 3), (2, 7), (3, 10)] {
        play(&mut s, seat, card);
    }
    assert_ne!(s.phase, Phase::Tinker);
    let (_, gained) = trick_scored(&s);
    assert_eq!(gained, [0, 3, 7, 0]);
}

#[test]
fn the_tinker_may_swap_with_a_scoring_card_but_not_a_royal() {
    let mut s = table(&[&[1, 4], &[8, 4], &[3, 4], &[5, 4]]);
    for (seat, card) in [(0, 1), (1, 8), (2, 3), (3, 5)] {
        play(&mut s, seat, card);
    }
    assert_eq!(s.phase, Phase::Tinker);
    assert_eq!(MarketDay::pending(&s).0, [Seat(0)]);
    assert_eq!(
        MarketDay::legal_actions(&s, Seat(0)),
        [
            Action::Swap { with: 1 },
            Action::Swap { with: 3 },
            Action::Keep
        ]
    );
    act(&mut s, 0, Action::Swap { with: 1 });
    let (cards, gained) = trick_scored(&s);
    assert_eq!(gained, [8, 1, 3, 5]);
    assert_eq!(cards[0], (Seat(1), r(1), Scored));
    assert_eq!(cards[1], (Seat(0), r(8), Scored));
    // Seat 0 took the most: it leads.
    assert_eq!(s.leader, Seat(0));
    assert_eq!(s.turn, Seat(0));
}

#[test]
fn two_players_play_twice_around() {
    let mut s = table(&[&[1, 4], &[8, 4]]);
    for (seat, card) in [(0, 1), (1, 8)] {
        play(&mut s, seat, card);
    }
    // Two players play twice around: nothing is scored yet.
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.turn, Seat(0));
}

#[test]
fn a_spoiled_tinker_does_not_act() {
    let mut s = table(&[&[1, 4], &[1, 4], &[6, 4], &[5, 4]]);
    for (seat, card) in [(0, 1), (1, 1), (2, 6), (3, 5)] {
        play(&mut s, seat, card);
    }
    // The second 1 is the Tinker that counts; seat 0's is spoiled.
    assert_eq!(s.phase, Phase::Tinker);
    assert_eq!(MarketDay::pending(&s).0, [Seat(1)]);
    act(&mut s, 1, Action::Keep);
    let (_, gained) = trick_scored(&s);
    assert_eq!(gained, [0, 1, 6, 5]);
}

#[test]
fn the_top_scorer_leads_the_next_trick_and_a_tie_keeps_the_leader() {
    let mut s = table(&[&[4, 4], &[9, 4], &[6, 4], &[5, 4]]);
    for (seat, card) in [(0, 4), (1, 9), (2, 6), (3, 5)] {
        play(&mut s, seat, card);
    }
    assert_eq!(s.leader, Seat(1));
    assert_eq!(s.turn, Seat(1));
    let mut s = table(&[&[6, 4], &[3, 4], &[3, 4], &[6, 4]]);
    for (seat, card) in [(0, 6), (1, 3), (2, 3), (3, 6)] {
        play(&mut s, seat, card);
    }
    // 6 spoiled for seat 0; seat 3 scores 6, seats 1 and 2 score 3: seat 3.
    assert_eq!(s.leader, Seat(3));
    let mut s = table(&[&[5, 4], &[5, 4], &[2, 4], &[2, 4]]);
    s.leader = Seat(2);
    s.turn = Seat(2);
    for (seat, card) in [(2, 2), (3, 2), (0, 5), (1, 5)] {
        play(&mut s, seat, card);
    }
    // Seat 0's 5 is spoiled, the first Knight kills seat 1's: seats 2 and 3
    // score 2 each, a tie, so the same leader.
    assert_eq!(s.leader, Seat(2));
}

#[test]
fn two_cards_each_at_three_players() {
    let mut s = table(&[&[8, 2, 1], &[9, 4, 1], &[6, 5, 1]]);
    let order = [(0, 8), (1, 9), (2, 6), (0, 2), (1, 4), (2, 5)];
    for (k, &(seat, card)) in order.iter().enumerate() {
        assert_eq!(s.turn, Seat(seat), "play {k}");
        play(&mut s, seat, card);
    }
    // Knight (seat 0) kills the 9: seat 0 scores 8 + 2, seat 1 4, seat 2 11.
    let (_, gained) = trick_scored(&s);
    assert_eq!(gained, [10, 4, 11]);
    assert_eq!(s.leader, Seat(2));
}

#[test]
fn deal_sizes_and_game_length() {
    for players in 2..=8u8 {
        let s = MarketDay::new(&Config::default(), players, u64::from(players));
        assert!(s.hands.iter().all(|h| h.len() == hand_size(players)));
        assert_eq!(s.turn, Seat((s.dealer.0 + 1) % players));
    }
    assert_eq!(hand_size(3), 10);
    assert_eq!(hand_size(4), 6);
    assert_eq!(hands_in_game(4), 6);
    assert_eq!(hands_in_game(8), 8);
}

#[test]
fn max_six_or_players_hands() {
    for players in [2u8, 4, 7] {
        let mut rng = SmallRng::seed_from_u64(u64::from(players));
        let mut s = MarketDay::new(&Config::default(), players, 70 + u64::from(players));
        let mut deals = 1;
        while let Some(&seat) = MarketDay::pending(&s).0.first() {
            let a = MarketDay::legal_actions(&s, seat)
                .choose(&mut rng)
                .cloned()
                .unwrap();
            MarketDay::apply(&mut s, seat, &a).unwrap();
            deals += s
                .events
                .iter()
                .filter(|e| matches!(e, Event::Dealt { .. }))
                .count();
            let cards = s.undealt.len()
                + s.discard.len()
                + s.trick.len()
                + s.hands.iter().map(Vec::len).sum::<usize>();
            assert_eq!(cards, 55);
        }
        assert_eq!(deals, usize::from(hands_in_game(players)));
        assert!(matches!(MarketDay::outcome(&s), Some(Outcome::Ranked(_))));
    }
}

#[test]
fn illegal_moves_are_refused() {
    let s = table(&[&[1, 4], &[8, 4], &[3, 4], &[5, 4]]);
    for (seat, a) in [
        (0, Action::Play { card: r(9) }),
        (1, Action::Play { card: r(8) }),
        (0, Action::Keep),
    ] {
        let mut copy = s.clone();
        assert!(
            MarketDay::apply(&mut copy, Seat(seat), &a).is_err(),
            "{a:?}"
        );
        assert_eq!(copy, s);
    }
}

#[test]
fn the_view_hides_hands() {
    let s = MarketDay::new(&Config::default(), 4, 2);
    let v = MarketDay::view(&s, Viewer::Seat(Seat(3)));
    assert_eq!(v.hand, s.hands[3]);
    assert_eq!(v.seats[0].hand_size, s.hands[0].len());
    let json = serde_json::to_string(&v).unwrap();
    assert!(
        !json.contains("undealt") && !json.contains("seed"),
        "{json}"
    );
}
