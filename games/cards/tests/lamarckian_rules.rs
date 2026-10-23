use cards::deck::Rank;
use cards::lamarckian::testing::{c, cards, table};
use cards::lamarckian::{
    lowest_rank, pack, Action, Category, Config, Event, Lamarckian, Phase, POOL_SIZE, TURNS,
};
use ludo_core::{Game, Outcome, Seat, Viewer};

fn mark(s: &mut cards::lamarckian::State, seat: u8, card: &str) {
    Lamarckian::apply(s, Seat(seat), &Action::Mark { card: c(card) }).unwrap();
}

fn sorted(mut v: Vec<cards::deck::Card>) -> Vec<cards::deck::Card> {
    v.sort();
    v
}

#[test]
fn the_pack_lasts_exactly_seven_pools() {
    for players in 2..=6u8 {
        let size = pack(players).len();
        assert_eq!(
            size,
            4 * usize::from(players) + POOL_SIZE * usize::from(TURNS)
        );
        assert!(pack(players)
            .iter()
            .all(|c| c.rank() >= Some(lowest_rank(players))));
    }
    assert_eq!(lowest_rank(6), Rank::Two);
    assert_eq!(lowest_rank(2), Rank::Six);
    let s = Lamarckian::new(&Config::default(), 3, 5);
    assert_eq!(s.deck.len(), POOL_SIZE * usize::from(TURNS - 1));
    assert!(s.hands.iter().all(|h| h.len() == 4));
}

#[test]
fn marks_act_from_the_highest_and_capture_rank_or_suit() {
    // [L] example: pool Ah Qc 9s 5h, marks As, 10c, 9h.
    let mut s = table(
        &["As 2d 3d 4d", "Tc 2s 3s 4s", "9h 2c 3c 4c"],
        "Ah Qc 9s 5h",
        "",
    );
    mark(&mut s, 0, "As");
    mark(&mut s, 1, "Tc");
    mark(&mut s, 2, "9h");
    // The ace takes the Ah and the 9s; the ten takes the Qc; the nine of
    // hearts takes the 5h.
    assert!(s.hands[0].contains(&c("Ah")) && s.hands[0].contains(&c("9s")));
    assert!(s.hands[1].contains(&c("Qc")));
    assert!(s.hands[2].contains(&c("5h")));
    assert_eq!(s.hands.iter().map(Vec::len).collect::<Vec<_>>(), [5, 4, 4]);
    let captures: Vec<Seat> = s
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Captured { seat, .. } => Some(*seat),
            _ => None,
        })
        .collect();
    assert_eq!(captures, [Seat(0), Seat(1), Seat(2)]);
    // The turn is over: the pool (the three marks) is discarded.
    assert_eq!(sorted(s.discard.clone()), sorted(cards("As Tc 9h")));
}

#[test]
fn a_mark_that_matches_nothing_picks_any_pool_card() {
    let mut s = table(&["2d 3d 4d 5d", "8c 2s 3s 4s"], "Kh Qh Jh", "");
    mark(&mut s, 0, "2d");
    mark(&mut s, 1, "8c");
    // The eight of clubs acts first and matches nothing.
    assert_eq!(s.phase, Phase::Pick);
    assert_eq!(s.picking, Some((Seat(1), c("8c"))));
    assert_eq!(Lamarckian::legal_actions(&s, Seat(1)).len(), 3);
    Lamarckian::apply(&mut s, Seat(1), &Action::Pick { card: c("Qh") }).unwrap();
    assert!(s.hands[1].contains(&c("Qh")));
    // The eight fell in; the two of diamonds then picks too.
    assert_eq!(s.picking, Some((Seat(0), c("2d"))));
    Lamarckian::apply(&mut s, Seat(0), &Action::Pick { card: c("8c") }).unwrap();
    assert_eq!(s.hands[0].len(), 4);
    assert_eq!(s.hands[1].len(), 4);
}

#[test]
fn a_single_pool_card_is_taken_without_asking() {
    let mut s = table(&["2d 3d 4d 5d", "8c 2s 3s 4s"], "Kh", "");
    mark(&mut s, 0, "2d");
    mark(&mut s, 1, "8c");
    // 8c takes the Kh; then the 2d finds only the 8c, and takes it.
    assert!(s.hands[1].contains(&c("Kh")));
    assert!(s.hands[0].contains(&c("8c")));
    assert_eq!(s.phase, Phase::Choose);
}

#[test]
fn nobody_learns_who_has_chosen() {
    let mut s = table(&["2d 3d 4d 5d", "8c 2s 3s 4s"], "Kh", "");
    mark(&mut s, 0, "2d");
    assert!(Lamarckian::visible_pending(&s, Viewer::Seat(Seat(0))).is_empty());
    assert_eq!(
        Lamarckian::visible_pending(&s, Viewer::Seat(Seat(1))).0,
        vec![Seat(1)]
    );
    assert_eq!(
        Lamarckian::view(&s, Viewer::Seat(Seat(1))).seats[0].mark,
        None
    );
}

#[test]
fn a_showdown_scores_by_place_and_ties_share() {
    // Last turn of the only competition. The 8c acts first and falls in;
    // the 4d takes it, the 2h takes the 4d. Seat 0 keeps three kings, seats
    // 1 and 2 end with Q Q 8 6 5 each.
    let mut s = table(
        &["Ks Kh Kd 7c 8c", "Qs Qh 6s 5s 3s 4d", "Qd Qc 8d 6h 5h 2h"],
        "",
        "",
    );
    s.turn_no = TURNS;
    mark(&mut s, 0, "8c");
    mark(&mut s, 1, "4d");
    mark(&mut s, 2, "2h");
    assert_eq!(s.phase, Phase::Over);
    let lines = s.last_showdown.clone().unwrap();
    assert_eq!(lines[0].hand.category, Category::Trips);
    assert_eq!(lines.iter().map(|l| l.place).collect::<Vec<_>>(), [0, 1, 1]);
    assert_eq!(s.scores, [3, 2, 2]);
    assert_eq!(
        Lamarckian::outcome(&s),
        Some(Outcome::Ranked(vec![0, 1, 1]))
    );
}

#[test]
fn a_game_plays_every_competition() {
    let mut s = Lamarckian::new(&Config::default(), 4, 11);
    let mut showdowns = 0;
    let mut seen = Vec::new();
    while let Some(&seat) = Lamarckian::pending(&s).0.first() {
        let action = Lamarckian::legal_actions(&s, seat)[0].clone();
        Lamarckian::apply(&mut s, seat, &action).unwrap();
        if s.events == seen {
            continue;
        }
        seen = s.events.clone();
        showdowns += s
            .events
            .iter()
            .filter(|e| matches!(e, Event::Showdown { .. }))
            .count();
        if s.phase == Phase::Choose {
            assert!(s.hands.iter().all(|h| h.len() >= 4), "a hand never shrinks");
        }
    }
    assert_eq!(showdowns, 4);
    // Each showdown hands out 4 + 3 + 2 + 1 points, more with ties.
    assert!(s.scores.iter().sum::<u32>() >= 40);
}

#[test]
fn the_config_is_checked() {
    assert!(Lamarckian::check_config(&Config { competitions: 0 }, 3).is_err());
    assert!(Lamarckian::check_config(&Config { competitions: 5 }, 3).is_err());
    assert!(Lamarckian::check_config(&Config { competitions: 1 }, 3).is_ok());
}
