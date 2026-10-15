use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::pendleton::testing::table;
use pairs::pendleton::{Action, Config, Event, Pendleton, Phase, State};
use pairs::testing::{r, ranks};

fn play(s: &mut State, seat: u8, column: u8) {
    let action = Action::Play { column };
    Pendleton::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn refused(s: &State, seat: u8, column: u8) {
    let mut copy = s.clone();
    let action = Action::Play { column };
    assert!(Pendleton::apply(&mut copy, Seat(seat), &action).is_err());
    assert_eq!(&copy, s);
    assert!(!Pendleton::legal_actions(s, Seat(seat)).contains(&action));
}

fn scored(s: &State) -> Option<&Event> {
    s.events
        .iter()
        .find(|e| matches!(e, Event::RoundScored { .. }))
}

#[test]
fn a_round_starts_with_five_distinct_sorted_targets() {
    for seed in 0..50 {
        let s = Pendleton::new(&Config::default(), 2, seed);
        assert_eq!(s.targets.len(), 5);
        assert!(s.targets.windows(2).all(|w| w[0] < w[1]), "{:?}", s.targets);
        // 55 cards: the targets, the drawn card, the rest in the pile.
        assert_eq!(s.supply.deck_len(), 49);
        assert!(s.drawn.is_some());
        assert_eq!(Pendleton::pending(&s), Pending::one(s.first));
        assert!(matches!(s.events[..], [Event::RoundStarted { .. }]));
    }
    let firsts: Vec<Seat> = (0..20)
        .map(|seed| Pendleton::new(&Config::default(), 2, seed).first)
        .collect();
    assert!(firsts.contains(&Seat(0)) && firsts.contains(&Seat(1)));
}

#[test]
fn a_card_never_joins_a_column_holding_its_rank() {
    // A 4 on the 4, beside an opposing 4, beside one's own 4: all refused.
    let s = table(
        &[2, 4, 6, 8, 9],
        [&[&[], &[], &[], &[4]], &[&[], &[], &[4]]],
        4,
        0,
        &[7],
    );
    refused(&s, 0, 1);
    refused(&s, 0, 2);
    refused(&s, 0, 3);
    refused(&s, 1, 0);
    assert_eq!(
        Pendleton::legal_actions(&s, Seat(0)),
        vec![Action::Play { column: 0 }, Action::Play { column: 4 }]
    );
}

#[test]
fn a_play_passes_the_turn_with_a_fresh_card() {
    let mut s = table(&[2, 4, 6, 8, 9], [&[], &[]], 5, 0, &[3, 7]);
    play(&mut s, 0, 2);
    assert_eq!(s.played[0][2], ranks(&[5]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.drawn, Some(r(7)));
    assert_eq!(
        s.events,
        vec![Event::Played {
            seat: Seat(0),
            card: r(5),
            column: 2,
            target: r(6)
        }]
    );
}

#[test]
fn an_unplayable_draw_scores_the_round() {
    // Seat 1 will draw a 7: every column holds one. Seat 0 wins the 2
    // (7 against nothing), loses the 4 and the 6; nobody played on the 7.
    let mut s = table(
        &[2, 4, 6, 7, 8],
        [
            &[&[7], &[3], &[1], &[], &[]],
            &[&[], &[3, 7], &[7, 2], &[], &[7]],
        ],
        5,
        0,
        &[7],
    );
    // Seat 0 plays its 5 on the 8, against the 7: no change.
    play(&mut s, 0, 4);
    let Some(Event::RoundScored {
        seat,
        card,
        columns,
        round,
        totals,
    }) = scored(&s).cloned()
    else {
        panic!("no score: {:?}", s.events);
    };
    assert_eq!((seat, card), (Seat(1), Some(r(7))));
    let won: Vec<Vec<u32>> = columns.iter().map(|c| c.scored.clone()).collect();
    assert_eq!(
        won,
        vec![vec![2, 0], vec![0, 4], vec![0, 6], vec![0, 0], vec![0, 8]]
    );
    assert_eq!(round, vec![2, 18]);
    assert_eq!(totals, vec![2, 18]);
    assert_eq!(s.scores, vec![2, 18]);
    // Seat 0 scored less: it goes first in round 2.
    assert_eq!(s.round_no, 2);
    assert_eq!(s.first, Seat(0));
    assert_eq!(s.turn, Some(Seat(0)));
    assert!(s.played.iter().flatten().all(Vec::is_empty));
}

#[test]
fn a_tie_halves_the_target_rounded_down() {
    let mut s = table(&[1, 3, 5, 7, 9], [&[&[4]], &[&[2]]], 2, 0, &[]);
    s.played[1][0] = ranks(&[4]);
    s.played[0][0] = ranks(&[4]);
    // 4 against 4 on the 1: half of 1 is 0. 0 against 0 elsewhere: nothing.
    assert_eq!(s.column_score(0).scored, vec![0, 0]);
    s.targets[0] = r(2);
    assert_eq!(s.column_score(0).scored, vec![1, 1]);
    assert_eq!(s.column_score(4).scored, vec![0, 0]);
}

#[test]
fn whoever_scored_less_leads_the_next_round() {
    // The pile is empty: the next draw ends the round (❓4).
    let mut s = table(&[1, 3, 5, 7, 9], [&[], &[]], 2, 0, &[]);
    play(&mut s, 0, 0);
    // 2 against nothing on the 1: seat 0 scores 1, seat 1 leads.
    assert_eq!(s.scores, vec![1, 0]);
    assert_eq!(s.first, Seat(1));
    let mut s = table(&[2, 3, 5, 7, 9], [&[], &[]], 4, 1, &[]);
    s.first = Seat(1);
    play(&mut s, 1, 1);
    assert_eq!(s.scores, vec![0, 3]);
    assert_eq!(s.first, Seat(0));
}

#[test]
fn after_an_even_round_the_second_player_leads() {
    // 4 against 1 + 3 on the 2: one point each.
    let mut s = table(&[2, 3, 5, 7, 9], [&[&[4]], &[&[1]]], 3, 1, &[]);
    s.first = Seat(1);
    play(&mut s, 1, 0);
    assert_eq!(s.scores, vec![1, 1]);
    assert_eq!(s.first, Seat(0));
}

#[test]
fn the_game_ends_when_someone_leads_past_the_target() {
    let mut s = table(&[1, 3, 5, 7, 9], [&[], &[]], 2, 0, &[]);
    s.scores = vec![98, 90];
    play(&mut s, 0, 4);
    assert_eq!(s.scores, vec![107, 90]);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Pendleton::pending(&s), Pending::none());
    assert_eq!(Pendleton::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
    refused(&s, 0, 0);
    // Even past the target: another round.
    let mut s = table(&[1, 3, 5, 7, 9], [&[], &[]], 2, 0, &[]);
    s.scores = vec![91, 100];
    play(&mut s, 0, 4);
    assert_eq!(s.scores, vec![100, 100]);
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.round_no, 2);
}

#[test]
fn both_players_see_the_same_table() {
    let s = Pendleton::new(&Config::default(), 2, 9);
    let a = Pendleton::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(a, Pendleton::view(&s, Viewer::Seat(Seat(1))));
    assert_eq!(a, Pendleton::view(&s, Viewer::Spectator));
    assert_eq!(a.drawn, s.drawn);
    assert_eq!(a.deck_size, 49);
}

#[test]
fn a_whole_game_reaches_the_target() {
    let mut s = Pendleton::new(&Config::default(), 2, 4);
    while let Some(&seat) = Pendleton::pending(&s).0.first() {
        let action = Pendleton::legal_actions(&s, seat)[0].clone();
        Pendleton::apply(&mut s, seat, &action).unwrap();
    }
    assert!(s.scores.iter().any(|&p| p >= 100));
    assert_ne!(s.scores[0], s.scores[1]);
}
