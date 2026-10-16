use cards::collusion::testing::{c, table};
use cards::collusion::{bonuses, score_deal, Action, Collusion, Config, Phrase, State};
use ludo_core::{Game, Outcome, Seat};

fn play(text: &str) -> Action {
    Action::Play { card: c(text) }
}

fn plays(s: &State, seat: u8) -> Vec<Action> {
    Collusion::legal_actions(s, Seat(seat))
        .into_iter()
        .filter(|a| matches!(a, Action::Play { .. }))
        .collect()
}

fn says(s: &State, seat: u8) -> Vec<Phrase> {
    Collusion::legal_actions(s, Seat(seat))
        .into_iter()
        .filter_map(|a| match a {
            Action::Say { phrase } => Some(phrase),
            Action::Play { .. } => None,
        })
        .collect()
}

/// Plays `cards` in turn, starting with whoever is to move.
fn run(s: &mut State, cards: &[&str]) {
    for text in cards {
        let seat = s.turn.expect("someone to move");
        Collusion::apply(s, seat, &play(text)).unwrap_or_else(|e| panic!("{text}: {e}"));
    }
}

#[test]
fn must_follow_suit() {
    let mut s = table(["2h 9s", "Kh 3h 4s", "5c", "6d"], 0);
    run(&mut s, &["2h"]);
    assert_eq!(plays(&s, 1), vec![play("Kh"), play("3h")]);
}

#[test]
fn any_card_when_void() {
    let mut s = table(["2h 9s", "4s 5d", "5c", "6d"], 0);
    run(&mut s, &["2h"]);
    assert_eq!(plays(&s, 1), vec![play("4s"), play("5d")]);
    run(&mut s, &["4s"]);
    assert!(s.voids[1][1], "seat 1 is out of hearts");
    assert!(!s.voids[1][0]);
}

#[test]
fn highest_of_led_suit_wins_no_trumps() {
    let mut s = table(["5h 2c", "As 2d", "3h 3c", "4h 4c"], 0);
    run(&mut s, &["5h", "As", "3h", "4h"]);
    assert_eq!(s.taken, [1, 0, 0, 0]);
}

#[test]
fn winner_leads_next_trick() {
    let mut s = table(["5h 2c", "Ah 2d", "3h 3c", "4h 4c"], 0);
    run(&mut s, &["5h", "Ah", "3h", "4h"]);
    assert_eq!(s.taken, [0, 1, 0, 0]);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn dealer_rotates_left() {
    let mut s = table(["5h", "Ah", "3h", "4h"], 1);
    assert_eq!(s.dealer, Seat(0));
    run(&mut s, &["Ah", "3h", "4h", "5h"]);
    assert_eq!(s.deal_no, 1);
    assert_eq!(s.dealer, Seat(1));
    assert_eq!(s.turn, Some(Seat(2)));
    assert!(s.hands.iter().all(|h| h.len() == 13));
}

#[test]
fn bonus_cases() {
    assert_eq!(bonuses([5, 5, 2, 1]), [10, 10, 0, 0]);
    assert_eq!(bonuses([6, 4, 2, 1]), [0, 0, 0, 20]);
    assert_eq!(bonuses([3, 3, 3, 4]), [0, 0, 0, 30]);
    assert_eq!(bonuses([0, 4, 4, 5]), [0, 10, 10, 0]);
}

#[test]
fn deducts_without_bonus_near_target() {
    let mut scores = [95, 0, 0, 0];
    let lines = score_deal(&mut scores, [6, 4, 2, 1], 100);
    assert_eq!(scores[0], 89);
    assert!(lines[0].deducted);
    assert_eq!(lines[0].bonus, 0);
    let mut scores = [45, 0, 0, 0];
    score_deal(&mut scores, [6, 4, 2, 1], 50);
    assert_eq!(scores[0], 39);
    // Below the target, tricks still count.
    let mut scores = [80, 0, 0, 0];
    score_deal(&mut scores, [6, 4, 2, 1], 100);
    assert_eq!(scores, [86, 4, 2, 21]);
}

#[test]
fn adds_with_bonus_past_target() {
    let mut scores = [95, 0, 0, 0];
    let lines = score_deal(&mut scores, [5, 5, 2, 1], 100);
    assert_eq!(scores[0], 110);
    assert!(!lines[0].deducted);
    assert_eq!(lines[0].total, 110);
}

#[test]
fn game_ends_on_target_reached_with_a_bonus() {
    // One card each: seat 0 takes the trick, the other three tie at 0 and
    // seat 0 earns the 30 bonus.
    let mut s = table(["Ah", "2h", "3h", "4h"], 0);
    s.scores = [70, 99, 10, 10];
    run(&mut s, &["Ah", "2h", "3h", "4h"]);
    assert_eq!(s.scores, [101, 99, 10, 10], "seat 1 adds nothing: 0 tricks");
    assert!(Collusion::pending(&s).is_empty());
    assert_eq!(s.turn, None);
    assert_eq!(
        Collusion::outcome(&s),
        Some(Outcome::Ranked(vec![0, 1, 2, 2]))
    );
}

#[test]
fn the_game_goes_on_below_target() {
    let mut s = table(["Ah", "2h", "3h", "4h"], 0);
    run(&mut s, &["Ah", "2h", "3h", "4h"]);
    assert_eq!(s.scores, [31, 0, 0, 0]);
    assert!(Collusion::outcome(&s).is_none());
    let last = s.last_deal.as_ref().expect("the deal is scored");
    assert_eq!(last[0].bonus, 30);
}

#[test]
fn say_once_then_play() {
    let mut s = table(["2h 9s", "Kh 3h", "5c 6c", "6d 7d"], 0);
    let phrase = Phrase::Aim { n: 1 };
    Collusion::apply(&mut s, Seat(0), &Action::Say { phrase }).unwrap();
    assert_eq!(s.turn, Some(Seat(0)));
    assert_eq!(s.talk, vec![(Seat(0), phrase)]);
    assert!(says(&s, 0).is_empty());
    let before = s.clone();
    assert!(Collusion::apply(
        &mut s,
        Seat(0),
        &Action::Say {
            phrase: Phrase::Unsure
        }
    )
    .is_err());
    assert_eq!(s, before);
    run(&mut s, &["2h"]);
    assert!(!says(&s, 1).is_empty(), "the next player may talk");
}

#[test]
fn aim_bounds() {
    let mut s = table(["2h 3h 4h 5h 6h 7h 8h 9h", "2s", "2c", "2d"], 0);
    s.taken = [2, 1, 1, 1];
    let aims: Vec<u8> = says(&s, 0)
        .into_iter()
        .filter_map(|p| match p {
            Phrase::Aim { n } => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(aims, (2..=10).collect::<Vec<_>>());
    let who: Vec<u8> = says(&s, 0)
        .into_iter()
        .filter_map(|p| match p {
            Phrase::WhoAims { n } => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(who, (0..=13).collect::<Vec<_>>());
    assert!(says(&s, 0).contains(&Phrase::Unsure));
}

#[test]
fn no_talk_with_a_single_playable_card() {
    let mut s = table(["2h 9s", "Kh 4s", "5c 6c", "6d 7d"], 0);
    run(&mut s, &["2h"]);
    assert_eq!(Collusion::legal_actions(&s, Seat(1)), vec![play("Kh")]);
}

#[test]
fn phrases_clear_on_new_deal() {
    let mut s = table(["Ah 2c", "2h 3c", "3h 4c", "4h 5c"], 0);
    Collusion::apply(
        &mut s,
        Seat(0),
        &Action::Say {
            phrase: Phrase::Unsure,
        },
    )
    .unwrap();
    run(&mut s, &["Ah", "2h", "3h", "4h"]);
    assert_eq!(s.talk.len(), 1, "phrases last the whole deal");
    run(&mut s, &["2c", "3c", "4c", "5c"]);
    assert_eq!(s.deal_no, 1);
    assert!(s.talk.is_empty());
}

#[test]
fn illegal_card_is_refused_unchanged() {
    let mut s = table(["2h 9s", "Kh 3h", "5c 6c", "6d 7d"], 0);
    let before = s.clone();
    assert!(Collusion::apply(&mut s, Seat(0), &play("Kh")).is_err());
    assert!(Collusion::apply(&mut s, Seat(1), &play("Kh")).is_err());
    assert_eq!(s, before);
}

#[test]
fn a_target_out_of_bounds_is_refused() {
    assert!(Collusion::check_config(&Config { target: 49 }, 4).is_err());
    assert!(Collusion::check_config(&Config { target: 101 }, 4).is_err());
    assert!(Collusion::check_config(&Config { target: 50 }, 4).is_ok());
    assert_eq!(Config::default().target, 100);
}

#[test]
fn a_new_game_deals_thirteen_each() {
    let s = Collusion::new(&Config::default(), 4, 7);
    assert!(s.hands.iter().all(|h| h.len() == 13));
    assert_eq!(s.dealer, Seat(0));
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn one_reaches_the_target_while_another_deducts() {
    let mut s = table(
        ["Ah 2s 3c 3d", "2h As 2c 4d", "3h 3s 4c 5d", "4h 4s Ac Ad"],
        0,
    );
    s.scores = [89, 0, 0, 98];
    run(
        &mut s,
        &[
            "Ah", "2h", "3h", "4h", "2s", "As", "3s", "4s", "2c", "4c", "Ac", "3c", "Ad", "3d",
            "4d", "5d",
        ],
    );
    // Tricks [1, 1, 0, 2]: seats 0 and 1 tie and earn 10; seat 3's two
    // tricks would reach 100 without a bonus, so they are taken off.
    assert_eq!(s.scores, [100, 11, 0, 96]);
    let last = s.last_deal.expect("the deal is scored");
    assert!(last[3].deducted);
    assert!(!last[0].deducted);
    assert_eq!(s.turn, None);
    assert_eq!(
        Collusion::outcome(&s),
        Some(Outcome::Ranked(vec![0, 2, 3, 1]))
    );
}
