use cards::deck::{Face, Rank};
use cards::president::testing::{c, cards, next_round, table};
use cards::president::{
    standings, title, Action, Config, Event, Jokers, Phase, President, State, Title,
};
use ludo_core::{Game, Match, Outcome, Pending, Seat, Viewer};
use Title::{President as Pres, Scum, Secretary, VicePresident as Vice};

fn play(rank: Rank, naturals: u8) -> Action {
    Action::Play {
        rank: Some(rank),
        naturals,
        jokers: 0,
    }
}

fn wild(rank: Rank, naturals: u8, jokers: u8) -> Action {
    Action::Play {
        rank: Some(rank),
        naturals,
        jokers,
    }
}

fn jokers(n: u8) -> Action {
    Action::Play {
        rank: None,
        naturals: 0,
        jokers: n,
    }
}

fn nat(rank: Rank) -> Face {
    Face::Natural { rank }
}

fn one_round() -> Config {
    Config {
        rounds: 1,
        ..Config::default()
    }
}

fn act(s: &mut State, seat: u8, a: Action) {
    President::apply(s, Seat(seat), &a).unwrap_or_else(|e| panic!("{a:?} par {seat} : {e}"));
}

#[test]
fn everyone_gets_the_same_number_of_cards_and_the_rest_is_shown() {
    let s = President::new(&Config::default(), 4, 1);
    assert!(s.hands.iter().all(|h| h.len() == 13));
    assert!(s.set_aside.is_empty());
    let s = President::new(&Config::default(), 5, 1);
    assert!(s.hands.iter().all(|h| h.len() == 10));
    assert_eq!(s.set_aside.len(), 2);
    assert_eq!(
        President::view(&s, Viewer::Seat(Seat(3))).set_aside,
        s.set_aside
    );
    // [W] Seat 0 deals the first round; the player on its left leads.
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
    let s = President::new(
        &Config {
            jokers: Jokers::Wild,
            ..Config::default()
        },
        4,
        1,
    );
    assert!(s.hands.iter().all(|h| h.len() == 13));
    assert_eq!(s.set_aside.len(), 2);
}

#[test]
fn a_lead_is_one_to_four_cards_of_a_rank() {
    let s = table(one_round(), &["3s 3h 7d 2c", "4s", "5s", "6s"], 0);
    assert_eq!(
        President::legal_actions(&s, Seat(0)),
        vec![
            play(Rank::Two, 1),
            play(Rank::Three, 1),
            play(Rank::Three, 2),
            play(Rank::Seven, 1)
        ]
    );
    assert!(President::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn a_play_is_beaten_by_as_many_cards_of_a_higher_rank() {
    let mut s = table(
        one_round(),
        &["7s 7h 9d", "6s 6h 8s 8h Ks 2c 2d", "5s", "4s"],
        0,
    );
    act(&mut s, 0, play(Rank::Seven, 2));
    assert_eq!(
        President::legal_actions(&s, Seat(1)),
        vec![play(Rank::Two, 2), play(Rank::Eight, 2), Action::Pass]
    );
}

#[test]
fn passing_does_not_stop_you_playing_later_in_the_trick() {
    let mut s = table(one_round(), &["5s 9s", "6s Ks", "7s Qs", "8s Js"], 0);
    act(&mut s, 0, play(Rank::Five, 1));
    act(&mut s, 1, Action::Pass);
    act(&mut s, 2, play(Rank::Seven, 1));
    act(&mut s, 3, Action::Pass);
    act(&mut s, 0, Action::Pass);
    // [P] Seat 1 passed earlier and may play again.
    act(&mut s, 1, play(Rank::King, 1));
}

#[test]
fn when_everyone_else_passes_the_last_player_leads_anything() {
    let mut s = table(one_round(), &["5s 9s 9h", "6s", "7s", "8s"], 0);
    act(&mut s, 0, play(Rank::Five, 1));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    assert!(s.trick.is_none());
    assert!(s.events.contains(&Event::TrickClosed { leader: Seat(0) }));
    assert_eq!(President::pending(&s), Pending::one(Seat(0)));
    assert!(President::legal_actions(&s, Seat(0)).contains(&play(Rank::Nine, 2)));
}

#[test]
fn if_the_last_player_went_out_the_player_on_their_left_leads() {
    let mut s = table(one_round(), &["As", "6s 6h", "7s 7h", "8s 8h"], 0);
    act(&mut s, 0, play(Rank::Ace, 1));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
}

#[test]
fn players_who_are_out_are_skipped() {
    let mut s = table(one_round(), &["3s", "4s 4h", "5s 5h", "6s 6h"], 0);
    act(&mut s, 0, play(Rank::Three, 1));
    act(&mut s, 1, play(Rank::Four, 1));
    act(&mut s, 2, play(Rank::Five, 1));
    act(&mut s, 3, play(Rank::Six, 1));
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
    assert!(President::legal_actions(&s, Seat(0)).is_empty());
}

#[test]
fn the_order_of_going_out_gives_titles_and_points() {
    let mut s = table(one_round(), &["3s", "4s", "5s", "6s 7s"], 0);
    act(&mut s, 0, play(Rank::Three, 1));
    act(&mut s, 1, play(Rank::Four, 1));
    act(&mut s, 2, play(Rank::Five, 1));
    // Seat 3 still holds cards: the round is over, it is the Scum.
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.titles, vec![Pres, Vice, Secretary, Scum]);
    assert_eq!(s.scores, vec![3, 2, 1, 0]);
    assert_eq!(
        President::outcome(&s),
        Some(Outcome::Ranked(vec![0, 1, 2, 3]))
    );
    assert_eq!(
        (0..5).map(|place| title(place, 5)).collect::<Vec<_>>(),
        vec![Pres, Vice, Title::Neutral, Secretary, Scum]
    );
}

#[test]
fn equal_scores_share_a_rank() {
    assert_eq!(standings(&[3, 3, 1, 0]), vec![0, 0, 2, 3]);
}

#[test]
fn the_game_ends_after_the_configured_rounds() {
    let mut s = President::new(
        &Config {
            rounds: 2,
            ..Config::default()
        },
        4,
        5,
    );
    let mut steps = 0;
    while let Some(&seat) = President::pending(&s).0.first() {
        let a = President::timeout_action(&s, seat).expect("a pending seat has a move");
        act(&mut s, seat.0, a);
        steps += 1;
        assert!(steps < 10_000, "the game does not end");
    }
    assert_eq!((s.round_no, s.phase), (2, Phase::Over));
    assert_eq!(s.scores.iter().sum::<u32>(), 2 * (3 + 2 + 1));
}

#[test]
fn the_scum_and_the_secretary_hand_over_their_best_cards() {
    let s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    assert_eq!(s.round_no, 2);
    assert_eq!(s.phase, Phase::Exchange);
    assert_eq!(s.hands[0], cards("4s 9s"));
    assert!(s.hands[1].contains(&c("2h")) && s.hands[1].contains(&c("Ks")));
    assert_eq!(s.hands[2], cards("4h 8s Qs"));
    assert!(s.hands[3].contains(&c("Ah")));
    assert_eq!(President::pending(&s), Pending(vec![Seat(1), Seat(3)]));
    assert_eq!(
        President::view(&s, Viewer::Seat(Seat(0))).given,
        cards("2h Ks")
    );
    assert_eq!(
        President::view(&s, Viewer::Seat(Seat(3))).received,
        cards("Ah")
    );
}

#[test]
fn the_president_and_the_vice_hand_cards_back_then_play_starts() {
    let mut s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    assert!(President::apply(&mut s.clone(), Seat(0), &Action::Pass).is_err());
    act(
        &mut s,
        1,
        Action::Give {
            faces: vec![nat(Rank::Three), nat(Rank::Three)],
        },
    );
    assert_eq!(President::pending(&s), Pending::one(Seat(3)));
    act(
        &mut s,
        3,
        Action::Give {
            faces: vec![nat(Rank::Three)],
        },
    );
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.hands[0], cards("4s 9s 3s 3h"));
    assert_eq!(s.hands[2], cards("4h 8s Qs 3d"));
    // [W] The Scum deals; the player on its left leads.
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
}

#[test]
fn the_president_may_hand_back_what_they_received() {
    let s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    let gifts = President::legal_actions(&s, Seat(1));
    assert!(gifts.contains(&Action::Give {
        faces: vec![nat(Rank::Two), nat(Rank::King)]
    }));
    assert!(gifts
        .iter()
        .all(|g| matches!(g, Action::Give { faces } if faces.len() == 2)));
}

#[test]
fn a_joker_is_the_best_card_to_hand_over() {
    let s = next_round(
        Config {
            jokers: Jokers::High,
            ..Config::default()
        },
        &[Scum, Pres, Secretary, Vice],
        &["Jo 2s 5s", "3s", "4s Ah", "6s"],
    );
    assert_eq!(s.hands[0], cards("5s"));
    assert!(
        President::legal_actions(&s, Seat(1)).contains(&Action::Give {
            faces: vec![nat(Rank::Two), Face::Joker]
        })
    );
}

#[test]
fn the_exchange_is_private() {
    let s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    let third = President::view(&s, Viewer::Seat(Seat(2)));
    assert!(third.received.is_empty());
    assert_eq!(third.given, cards("Ah"));
    let watcher = President::view(&s, Viewer::Spectator);
    assert!(watcher.received.is_empty() && watcher.given.is_empty() && watcher.hand.is_empty());
    for e in &third.events {
        if let Event::Exchanged { count, .. } = e {
            assert!(*count <= 2);
        }
    }
    let json = serde_json::to_string(&third).unwrap();
    assert!(!json.contains("seed"));
}

#[test]
fn a_hand_is_shown_weakest_first() {
    let s = table(one_round(), &["2s As 3s", "4s", "5s", "6s"], 0);
    assert_eq!(
        President::view(&s, Viewer::Seat(Seat(0))).hand,
        cards("3s As 2s")
    );
}

#[test]
fn strong_jokers_beat_twos_alone_or_as_a_pair() {
    let cfg = Config {
        jokers: Jokers::High,
        ..one_round()
    };
    let mut s = table(cfg, &["2s 2h 3s", "Jo Jo 7s", "5s", "6s"], 0);
    act(&mut s, 0, play(Rank::Two, 2));
    assert_eq!(
        President::legal_actions(&s, Seat(1)),
        vec![jokers(2), Action::Pass]
    );
}

#[test]
fn a_strong_joker_never_joins_other_cards() {
    let cfg = Config {
        jokers: Jokers::High,
        ..one_round()
    };
    let s = table(cfg, &["3s", "Jo 9s 9h", "5s", "6s"], 1);
    let legal = President::legal_actions(&s, Seat(1));
    assert!(legal.contains(&jokers(1)));
    assert!(!legal.iter().any(|a| matches!(
        a,
        Action::Play {
            rank: Some(_),
            jokers: 1..,
            ..
        }
    )));
}

#[test]
fn a_wild_joker_completes_a_set() {
    let cfg = Config {
        jokers: Jokers::Wild,
        ..one_round()
    };
    let mut s = table(cfg, &["6s 6h 4s", "8s Jo 3s", "5s", "9s"], 0);
    act(&mut s, 0, play(Rank::Six, 2));
    assert_eq!(
        President::legal_actions(&s, Seat(1)),
        vec![wild(Rank::Eight, 1, 1), Action::Pass]
    );
}

#[test]
fn a_wild_set_does_not_beat_an_equal_natural_set() {
    let cfg = Config {
        jokers: Jokers::Wild,
        ..one_round()
    };
    let mut s = table(cfg, &["7s 7h 4s", "7d Jo 3s", "5s", "9s"], 0);
    act(&mut s, 0, play(Rank::Seven, 2));
    assert_eq!(President::legal_actions(&s, Seat(1)), vec![Action::Pass]);
}

#[test]
fn wild_jokers_alone_are_the_strongest_set_of_their_size() {
    let cfg = Config {
        jokers: Jokers::Wild,
        ..one_round()
    };
    let mut s = table(cfg, &["2s 2h 4s", "Jo Jo 5s", "6s", "9s"], 0);
    act(&mut s, 0, play(Rank::Two, 2));
    assert_eq!(
        President::legal_actions(&s, Seat(1)),
        vec![jokers(2), Action::Pass]
    );
}

#[test]
fn absurd_actions_are_refused_and_change_nothing() {
    let s = table(one_round(), &["5s 9s", "6s", "7s", "8s"], 0);
    for (seat, a) in [
        (1, play(Rank::Six, 1)),
        (0, play(Rank::Five, 2)),
        (0, play(Rank::King, 1)),
        (0, Action::Pass),
        (0, Action::Give { faces: vec![] }),
        (0, jokers(1)),
        (9, play(Rank::Five, 1)),
    ] {
        let mut copy = s.clone();
        assert!(
            President::apply(&mut copy, Seat(seat), &a).is_err(),
            "{a:?}"
        );
        assert_eq!(copy, s);
    }
    let mut over = table(one_round(), &["3s", "4s", "5s", "6s 7s"], 0);
    act(&mut over, 0, play(Rank::Three, 1));
    act(&mut over, 1, play(Rank::Four, 1));
    act(&mut over, 2, play(Rank::Five, 1));
    assert!(President::apply(&mut over, Seat(3), &play(Rank::Six, 1)).is_err());
}

#[test]
fn a_number_of_rounds_out_of_bounds_is_refused() {
    for rounds in [0, 11] {
        let e = Match::<President>::new(
            Config {
                rounds,
                ..Config::default()
            },
            4,
            1,
        )
        .err()
        .expect("refused");
        assert_eq!(e.0, "Le Président se joue en 1 à 10 manches.");
    }
}

#[test]
fn the_clock_passes_or_plays_the_weakest_cards() {
    let s = table(one_round(), &["9s 3h 3d", "4s", "5s", "6s"], 0);
    assert_eq!(
        President::timeout_action(&s, Seat(0)),
        Some(play(Rank::Three, 1))
    );
    let mut s = s;
    act(&mut s, 0, play(Rank::Three, 1));
    assert_eq!(President::timeout_action(&s, Seat(1)), Some(Action::Pass));
}

/// Plays the clock's choice until the current round is scored.
fn finish_round(s: &mut State) {
    let round = s.round_no;
    while s.round_no == round && s.phase != Phase::Over {
        let seat = President::pending(s).0[0];
        let a = President::timeout_action(s, seat).expect("a pending seat has a move");
        act(s, seat.0, a);
    }
}

fn revolution() -> Config {
    Config {
        revolution: true,
        ..one_round()
    }
}

#[test]
fn four_of_a_kind_starts_a_revolution_only_with_the_option() {
    let hands = ["7s 7h 7d 7c 9s", "3s 4s 2s", "5s", "6s"];
    let mut off = table(one_round(), &hands, 0);
    act(&mut off, 0, play(Rank::Seven, 4));
    assert!(!off.reversed);
    let mut on = table(revolution(), &hands, 0);
    act(&mut on, 0, play(Rank::Seven, 4));
    assert!(on.reversed);
    assert!(on.events.contains(&Event::Revolution { reversed: true }));
}

#[test]
fn a_revolution_reverses_the_ranks() {
    let mut s = table(revolution(), &["9s 9h 9d 9c 8s", "3s 2s Ks", "5s", "6s"], 0);
    act(&mut s, 0, play(Rank::Nine, 4));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    act(&mut s, 0, play(Rank::Eight, 1));
    // The 3 is now the best card, the 2 the worst.
    assert_eq!(
        President::legal_actions(&s, Seat(1)),
        vec![play(Rank::Three, 1), Action::Pass]
    );
    assert_eq!(
        President::view(&s, Viewer::Seat(Seat(1))).hand,
        cards("2s Ks 3s")
    );
}

#[test]
fn a_second_four_of_a_kind_turns_the_ranks_back() {
    let mut s = table(revolution(), &["4s 4h 4d 4c 9s", "5s", "6s", "8s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Four, 4));
    assert!(!s.reversed);
    assert!(s.events.contains(&Event::Revolution { reversed: false }));
}

#[test]
fn a_four_of_a_kind_with_wild_jokers_is_a_revolution() {
    let cfg = Config {
        jokers: Jokers::Wild,
        ..revolution()
    };
    let mut s = table(cfg, &["9s 9h 9d Jo 4s", "5s", "6s", "8s"], 0);
    act(&mut s, 0, wild(Rank::Nine, 3, 1));
    assert!(s.reversed);
}

#[test]
fn strong_jokers_stay_on_top_during_a_revolution() {
    let cfg = Config {
        jokers: Jokers::High,
        ..revolution()
    };
    let mut s = table(cfg, &["3s 4s", "Jo 5s", "6s", "8s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Three, 1));
    assert_eq!(
        President::legal_actions(&s, Seat(1)),
        vec![jokers(1), Action::Pass]
    );
}

#[test]
fn a_new_round_restores_the_normal_order() {
    let cfg = Config {
        rounds: 2,
        ..revolution()
    };
    let mut s = table(cfg, &["4s 4h 4d 4c", "5s 6s", "7s 8s", "9s Ts"], 0);
    act(&mut s, 0, play(Rank::Four, 4));
    assert!(s.reversed);
    finish_round(&mut s);
    assert_eq!(s.round_no, 2);
    assert!(!s.reversed);
}

fn no_two() -> Config {
    Config {
        no_finish_on_two: true,
        ..one_round()
    }
}

#[test]
fn going_out_on_a_two_makes_you_the_scum() {
    let mut s = table(no_two(), &["2s", "3s 4s", "5s 6s", "7s 8s"], 0);
    act(&mut s, 0, play(Rank::Two, 1));
    assert_eq!(s.fouled, vec![Seat(0)]);
    assert!(s.events.contains(&Event::Finished {
        seat: Seat(0),
        fouled: true
    }));
    finish_round(&mut s);
    assert_eq!(s.titles[0], Scum);
    assert_eq!(s.scores[0], 0);
    let mut free = table(one_round(), &["2s", "3s 4s", "5s 6s", "7s 8s"], 0);
    act(&mut free, 0, play(Rank::Two, 1));
    assert_eq!(free.out, vec![Seat(0)]);
}

#[test]
fn during_a_revolution_the_forbidden_card_is_the_three() {
    let cfg = Config {
        revolution: true,
        ..no_two()
    };
    let mut s = table(cfg.clone(), &["3s", "4s 5s", "6s 7s", "8s 9s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Three, 1));
    assert_eq!(s.fouled, vec![Seat(0)]);
    let mut s = table(cfg, &["2s", "4s 5s", "6s 7s", "8s 9s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Two, 1));
    assert_eq!(s.out, vec![Seat(0)]);
}

#[test]
fn going_out_on_a_joker_is_forbidden_too() {
    let cfg = Config {
        jokers: Jokers::High,
        ..no_two()
    };
    let mut s = table(cfg, &["Jo", "4s 5s", "6s 7s", "8s 9s"], 0);
    act(&mut s, 0, jokers(1));
    assert_eq!(s.fouled, vec![Seat(0)]);
}

#[test]
fn the_forbidden_card_is_judged_before_its_own_revolution() {
    // ❓4: four 3s going out are judged in the normal order: allowed.
    let cfg = Config {
        revolution: true,
        ..no_two()
    };
    let mut s = table(cfg, &["3s 3h 3d 3c", "4s 5s", "6s 7s", "8s 9s"], 0);
    act(&mut s, 0, play(Rank::Three, 4));
    assert_eq!(s.out, vec![Seat(0)]);
    assert!(s.reversed);
}

#[test]
fn the_first_to_foul_finishes_last() {
    let mut s = table(no_two(), &["2s", "2h", "5s 6s", "7s 8s"], 0);
    act(&mut s, 0, play(Rank::Two, 1));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    act(&mut s, 1, play(Rank::Two, 1));
    assert_eq!(s.fouled, vec![Seat(0), Seat(1)]);
    finish_round(&mut s);
    assert_eq!((s.titles[0], s.titles[1]), (Scum, Secretary));
}
