use cards::david_goliath::testing::{c, cards, table};
use cards::david_goliath::{color_points, hand_points, Action, Color, Config, DavidGoliath, State};
use ludo_core::{Determinize, Game, Outcome, Seat, Viewer};
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn play(s: &mut State, seat: u8, card: &str) {
    DavidGoliath::apply(s, Seat(seat), &Action::Play { card: c(card) }).unwrap();
}

fn won(s: &State, seat: usize) -> Vec<String> {
    let mut v: Vec<String> = s.won[seat].iter().map(|k| k.to_string()).collect();
    v.sort();
    v
}

#[test]
fn the_pack_depends_on_the_table_size() {
    for players in 3..=6u8 {
        let s = DavidGoliath::new(&Config::default(), players, 7);
        assert!(s.hands.iter().all(|h| h.len() == 15));
        let mut all: Vec<_> = s.hands.concat();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 15 * usize::from(players));
        assert!(all.iter().all(|k| (1..=3 * players).contains(&k.value)));
        // [R] The player at the dealer's left leads.
        assert_eq!(s.turn, Some(Seat(1)));
    }
}

#[test]
fn the_led_colour_must_be_followed() {
    let mut s = table(&["g4 r1", "g9 r3", "r2 b5", "g7 v1"], 0);
    play(&mut s, 0, "g4");
    assert_eq!(
        DavidGoliath::legal_actions(&s, Seat(1)),
        vec![Action::Play { card: c("g9") }]
    );
    let before = s.clone();
    assert!(DavidGoliath::apply(&mut s, Seat(1), &Action::Play { card: c("r3") }).is_err());
    assert_eq!(s, before, "an illegal move changes nothing");
    play(&mut s, 1, "g9");
    // No green: any card.
    assert_eq!(DavidGoliath::legal_actions(&s, Seat(2)).len(), 2);
}

#[test]
fn the_example_of_the_rules() {
    // [R] Green 4, green 9, red 2 (no green), green 7.
    let mut s = table(&["g4 r1", "g9 r3", "r2 b5", "g7 v1"], 0);
    play(&mut s, 0, "g4");
    play(&mut s, 1, "g9");
    play(&mut s, 2, "r2");
    play(&mut s, 3, "g7");
    assert_eq!(won(&s, 2), vec!["g9"]);
    assert_eq!(won(&s, 1), vec!["g4", "g7", "r2"]);
    assert!(won(&s, 0).is_empty() && won(&s, 3).is_empty());
    assert_eq!(s.turn, Some(Seat(1)), "the highest card leads");
    let last = s.last_trick.as_ref().unwrap();
    assert_eq!((last.highest, last.lowest), (Seat(1), Seat(2)));
    // The player out of green is known to be out of it.
    assert!(s.void[2][Color::Green.index()]);
}

#[test]
fn among_equal_values_the_last_played_counts() {
    // Highest 9 twice, lowest 2 twice: the last of each.
    let mut s = table(&["y9 r1", "y2 r3", "y9 b5", "y2 v1"], 0);
    play(&mut s, 0, "y9");
    play(&mut s, 1, "y2");
    play(&mut s, 2, "y9");
    play(&mut s, 3, "y2");
    let last = s.last_trick.as_ref().unwrap();
    assert_eq!((last.highest, last.lowest), (Seat(2), Seat(3)));
    assert_eq!(won(&s, 3), vec!["y9"]);
    assert_eq!(won(&s, 2), vec!["y2", "y2", "y9"]);
}

#[test]
fn all_equal_values_go_to_the_last_player() {
    let mut s = table(&["y5 r1", "r5 r3", "b5 b2"], 0);
    play(&mut s, 0, "y5");
    play(&mut s, 1, "r5");
    play(&mut s, 2, "b5");
    assert_eq!(won(&s, 2), vec!["b5", "r5", "y5"]);
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn the_scoring_example_of_the_rules() {
    let pile = cards("r4 r10 r11 y8 g5 g9 b2 b3 b6 b10 b12 v10 v12");
    assert_eq!(color_points(&cards("r4 r10 r11")), 3);
    assert_eq!(color_points(&cards("y8")), 8);
    assert_eq!(color_points(&cards("g5 g9")), 14);
    assert_eq!(color_points(&cards("b2 b3 b6 b10 b12")), 5);
    assert_eq!(color_points(&cards("v10 v12")), 22);
    assert_eq!(hand_points(&pile), 52);
    assert_eq!(color_points(&[]), 0);
}

#[test]
fn a_game_lasts_one_round_per_player() {
    for players in 3..=6u8 {
        let mut s = DavidGoliath::new(&Config::default(), players, 11);
        let mut rounds_seen = vec![(s.round, s.dealer)];
        while let Some(seat) = s.turn {
            let a = DavidGoliath::timeout_action(&s, seat).unwrap();
            DavidGoliath::apply(&mut s, seat, &a).unwrap();
            if rounds_seen.last().unwrap().0 != s.round {
                rounds_seen.push((s.round, s.dealer));
            }
        }
        assert_eq!(rounds_seen.len(), usize::from(players));
        for (k, &(round, dealer)) in rounds_seen.iter().enumerate() {
            assert_eq!(round, k as u32);
            assert_eq!(
                dealer,
                Seat(k as u8 % players),
                "the deal passes to the left"
            );
        }
        let lines = s.last_round.as_ref().unwrap();
        assert_eq!(s.scores, lines.iter().map(|l| l.total).collect::<Vec<_>>());
        assert!(matches!(
            DavidGoliath::outcome(&s),
            Some(Outcome::Ranked(_))
        ));
        assert!(DavidGoliath::apply(&mut s, Seat(0), &Action::Play { card: c("r1") }).is_err());
    }
}

#[test]
fn the_round_scores_the_won_cards() {
    // Last trick of a round: one card each.
    let mut s = table(&["g4", "g9", "r2"], 0);
    s.won[0] = cards("v10 v12");
    play(&mut s, 0, "g4");
    play(&mut s, 1, "g9");
    play(&mut s, 2, "r2");
    // Seat 0: 22; seat 1: g4 + r2 = 6; seat 2: g9.
    let lines = s.last_round.as_ref().unwrap();
    assert_eq!(
        lines.iter().map(|l| l.points).collect::<Vec<_>>(),
        vec![22, 6, 9]
    );
    assert_eq!(s.scores, vec![22, 6, 9]);
    assert_eq!(s.round, 1);
    assert!(s.won.iter().all(|w| w.is_empty()));
    // The new round is led from the dealer's left, not by the last taker.
    assert_eq!(s.last_trick, None);
}

#[test]
fn determinization_respects_known_voids() {
    let mut s = DavidGoliath::new(&Config::default(), 4, 5);
    // Play a few tricks with the lowest legal card.
    for _ in 0..24 {
        let seat = s.turn.unwrap();
        let a = DavidGoliath::timeout_action(&s, seat).unwrap();
        DavidGoliath::apply(&mut s, seat, &a).unwrap();
    }
    let observer = Seat(0);
    let view = DavidGoliath::view(&s, Viewer::Seat(observer));
    let mut changed = false;
    for seed in 0..30 {
        let d = DavidGoliath::determinize(&s, observer, &mut SmallRng::seed_from_u64(seed));
        assert_eq!(d.hands[0], s.hands[0]);
        assert_eq!(DavidGoliath::view(&d, Viewer::Seat(observer)), view);
        for (i, hand) in d.hands.iter().enumerate() {
            assert_eq!(hand.len(), s.hands[i].len());
            for card in hand {
                assert!(
                    !d.void[i][card.color.index()],
                    "seat {i} holds a void colour"
                );
            }
        }
        changed |= d.hands[1..] != s.hands[1..];
    }
    assert!(changed, "the other hands are actually redealt");
}
