use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::testing::{r, ranks};
use pairs::the_cave::testing::table;
use pairs::the_cave::{
    capture, hand_size, round_points, Action, Config, Event, Phase, State, TheCave,
};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

fn play(s: &mut State, seat: u8, card: u8) {
    let action = Action::Play { card: r(card) };
    TheCave::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn total(s: &State) -> usize {
    s.deck.len()
        + s.cave.len()
        + s.hands.iter().map(Vec::len).sum::<usize>()
        + s.captured.iter().map(Vec::len).sum::<usize>()
}

#[test]
fn capture_takes_the_closest_lower_rank() {
    assert_eq!(capture(&ranks(&[4, 7, 7, 8, 9]), r(8)), ranks(&[7, 7]));
    assert_eq!(capture(&ranks(&[2, 9, 3]), r(10)), ranks(&[9]));
}

#[test]
fn capture_takes_the_highest_when_nothing_is_lower() {
    assert_eq!(capture(&ranks(&[4, 5, 7, 9, 10]), r(4)), ranks(&[10]));
    assert_eq!(capture(&ranks(&[4, 5, 7, 9, 10]), r(1)), ranks(&[10]));
    assert_eq!(capture(&ranks(&[5, 5, 6]), r(5)), ranks(&[6]));
    assert_eq!(capture(&ranks(&[5, 5]), r(5)), ranks(&[5, 5]));
}

#[test]
fn an_empty_cave_captures_nothing() {
    assert!(capture(&[], r(5)).is_empty());
    let mut s = table(&[&[5, 3], &[2, 4]], &[], &[]);
    play(&mut s, 0, 5);
    assert!(s.captured[0].is_empty());
    assert_eq!(s.cave, ranks(&[5]));
}

#[test]
fn a_play_captures_enters_the_cave_and_refills_it() {
    let mut s = table(&[&[8, 1], &[2, 2]], &[4, 7, 7, 8, 9], &[6, 6, 3]);
    play(&mut s, 0, 8);
    assert_eq!(s.captured[0], ranks(&[7, 7]));
    let mut cave = s.cave.clone();
    cave.sort_unstable();
    assert_eq!(cave, ranks(&[4, 6, 8, 8, 9]));
    assert_eq!(s.deck.len(), 2);
    assert_eq!(s.turn, Seat(1));
    assert!(s.events.contains(&Event::Played {
        seat: Seat(0),
        card: r(8),
        captured: ranks(&[7, 7])
    }));
}

#[test]
fn the_cave_is_not_refilled_once_the_deck_is_empty() {
    let mut s = table(&[&[8, 1], &[9, 2]], &[4, 7, 7, 8, 9], &[6]);
    play(&mut s, 0, 8);
    assert_eq!(s.cave.len(), 5);
    assert!(s.deck.is_empty());
    // The 9 takes both 8s: four cards left, nothing to refill with.
    play(&mut s, 1, 9);
    assert_eq!(s.captured[1], ranks(&[8, 8]));
    assert_eq!(s.cave.len(), 4);
}

#[test]
fn majority_scores_one_point_per_card_and_ties_share() {
    let captured = vec![ranks(&[10, 10, 10, 2]), ranks(&[10, 2, 5]), ranks(&[5])];
    assert_eq!(round_points(&captured), [3 + 1, 1 + 1, 1]);
}

#[test]
fn the_round_ends_when_hands_are_empty() {
    let mut s = table(&[&[8], &[2]], &[4, 7, 7, 8, 9], &[]);
    play(&mut s, 0, 8);
    play(&mut s, 1, 2);
    let over = s.events.iter().find_map(|e| match e {
        Event::RoundOver { points, scores } => Some((points.clone(), scores.clone())),
        _ => None,
    });
    // Seat 0 took the two 7s, seat 1 the 9.
    assert_eq!(over, Some((vec![2, 1], vec![2, 1])));
}

#[test]
fn deal_sizes() {
    for players in 2..=6u8 {
        let s = TheCave::new(&Config::default(), players, u64::from(players));
        assert!(s.hands.iter().all(|h| h.len() == hand_size(players)));
        assert_eq!(s.cave.len(), 5);
        assert_eq!(total(&s), 55);
        assert_eq!(s.turn, Seat((s.dealer.0 + 1) % players));
    }
    assert_eq!((2..=6).map(hand_size).collect::<Vec<_>>(), [7, 6, 5, 5, 4]);
}

#[test]
fn one_round_per_player_and_the_deal_rotates() {
    for players in 2..=6u8 {
        let mut rng = SmallRng::seed_from_u64(u64::from(players));
        let mut s = TheCave::new(&Config::default(), players, 9 + u64::from(players));
        let mut dealers = vec![s.dealer.0];
        while let Some(&seat) = TheCave::pending(&s).0.first() {
            let a = TheCave::legal_actions(&s, seat)
                .choose(&mut rng)
                .cloned()
                .unwrap();
            TheCave::apply(&mut s, seat, &a).unwrap();
            assert_eq!(total(&s), 55);
            for e in &s.events {
                if let Event::Dealt { dealer, .. } = e {
                    dealers.push(dealer.0);
                }
            }
        }
        assert_eq!(dealers.len(), usize::from(players));
        for w in dealers.windows(2) {
            assert_eq!(w[1], (w[0] + 1) % players);
        }
        assert_eq!(s.phase, Phase::Over);
        assert!(matches!(TheCave::outcome(&s), Some(Outcome::Ranked(_))));
    }
}

#[test]
fn illegal_moves_are_refused() {
    let s = table(&[&[8, 1], &[2, 2]], &[4, 7], &[6]);
    for (seat, card) in [(0, 5), (1, 2)] {
        let mut copy = s.clone();
        assert!(TheCave::apply(&mut copy, Seat(seat), &Action::Play { card: r(card) }).is_err());
        assert_eq!(copy, s);
    }
    assert_eq!(
        TheCave::legal_actions(&s, Seat(0)),
        [Action::Play { card: r(1) }, Action::Play { card: r(8) }]
    );
}

#[test]
fn the_view_hides_hands_and_deck() {
    let s = TheCave::new(&Config::default(), 3, 4);
    let v = TheCave::view(&s, Viewer::Seat(Seat(2)));
    assert_eq!(v.hand, s.hands[2]);
    assert_eq!(v.cave, s.cave);
    assert_eq!(v.deck_size, s.deck.len());
    assert_eq!(v.seats[0].hand_size, s.hands[0].len());
    let json = serde_json::to_string(&v).unwrap();
    assert!(
        !json.contains("\"deck\"") && !json.contains("seed"),
        "{json}"
    );
}
