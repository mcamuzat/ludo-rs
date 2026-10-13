use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::auberge::testing::{setup, table};
use pairs::auberge::{Action, Auberge, Config, Event, Phase, State};
use pairs::deck::{Supply, DECK_SIZE};
use pairs::testing::{r, ranks};
use serde_json::json;

fn act(s: &mut State, seat: u8, action: Action) {
    Auberge::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn place(card: u8) -> Action {
    Action::Place { card: r(card) }
}

fn attack(target: u8, card: u8) -> Action {
    Action::Attack {
        target: Seat(target),
        card: r(card),
    }
}

fn dare(target: u8) -> Action {
    Action::Dare {
        target: Seat(target),
    }
}

fn accuse(target: u8) -> Action {
    Action::Accuse {
        target: Seat(target),
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        Auberge::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!Auberge::legal_actions(s, Seat(seat)).contains(&action));
}

#[test]
fn a_new_game_deals_three_cards_and_waits_for_everyone() {
    for players in 2..=8u8 {
        for seed in 0..10 {
            let s = Auberge::new(&Config::default(), players, seed);
            assert_eq!(s.phase, Phase::Setup);
            assert!(s.hands.iter().all(|h| h.len() == 3));
            assert_eq!(s.supply.deck_len(), DECK_SIZE - 3 * usize::from(players));
            assert_eq!(Auberge::pending(&s).0.len(), usize::from(players));
            assert_eq!((s.round, s.rounds), (1, 5));
        }
    }
}

#[test]
fn rounds_come_from_the_config_within_bounds() {
    let rounds = |r| Auberge::new(&Config { rounds: r }, 2, 1).rounds;
    assert_eq!(rounds(None), 5);
    assert_eq!(rounds(Some(0)), 5);
    assert_eq!(rounds(Some(3)), 3);
    assert_eq!(rounds(Some(255)), 15);
}

#[test]
fn a_placed_card_stays_secret_until_everyone_has_placed() {
    let mut s = setup(&[&[5, 6, 7], &[2, 8, 9], &[4, 4, 10]], &[1]);
    act(&mut s, 0, place(5));
    assert_eq!(Auberge::pending(&s), Pending(vec![Seat(1), Seat(2)]));
    let other = Auberge::view(&s, Viewer::Seat(Seat(1)));
    assert!(other.seats[0].front.is_empty());
    assert_eq!(other.seats[0].hand_size, 3);
    assert_eq!(other.placed, None);
    assert_eq!(Auberge::view(&s, Viewer::Seat(Seat(0))).placed, Some(r(5)));
    // Nobody learns who has already chosen.
    assert_eq!(
        Auberge::visible_pending(&s, Viewer::Seat(Seat(0))),
        Pending::none()
    );
    assert_eq!(
        Auberge::visible_pending(&s, Viewer::Seat(Seat(1))),
        Pending::one(Seat(1))
    );
    assert_eq!(
        Auberge::visible_pending(&s, Viewer::Spectator),
        Pending::none()
    );
    refused(&s, 0, place(6));
}

#[test]
fn the_lowest_placed_card_starts() {
    let mut s = setup(&[&[5, 6, 7], &[2, 8, 9], &[4, 4, 10]], &[1]);
    act(&mut s, 0, place(5));
    act(&mut s, 2, place(4));
    act(&mut s, 1, place(2));
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.fronts, vec![ranks(&[5]), ranks(&[2]), ranks(&[4])]);
    assert_eq!(
        s.hands,
        vec![ranks(&[6, 7]), ranks(&[8, 9]), ranks(&[4, 10])]
    );
    assert!(s.events.contains(&Event::Starts { seat: Seat(1) }));
    assert_eq!(Auberge::pending(&s), Pending::one(Seat(1)));
}

#[test]
fn a_tie_for_lowest_is_drawn_by_lot() {
    let mut starters = Vec::new();
    for seed in 0..40 {
        let mut s = setup(&[&[3, 6, 7], &[3, 8, 9], &[5, 5, 10]], &[1]);
        s.seed = seed;
        act(&mut s, 0, place(3));
        act(&mut s, 1, place(3));
        act(&mut s, 2, place(5));
        starters.push(s.turn.unwrap());
    }
    assert!(starters.contains(&Seat(0)) && starters.contains(&Seat(1)));
    assert!(!starters.contains(&Seat(2)));
}

#[test]
fn placing_a_card_you_do_not_hold_is_refused() {
    let s = setup(&[&[5, 6, 7], &[2, 8, 9]], &[1]);
    refused(&s, 0, place(9));
    refused(&s, 0, Action::Draw);
}

#[test]
fn drawing_is_secret_and_passes_left() {
    let mut s = table(&[&[5, 6], &[8]], &[&[2], &[3]], &[9]);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.hands[0], ranks(&[5, 6, 9]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.events, vec![Event::Drew { seat: Seat(0) }]);
}

#[test]
fn an_attack_gives_a_hand_card_and_the_turn_to_the_target() {
    let mut s = table(&[&[5, 6], &[8], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, attack(2, 5));
    assert_eq!(s.hands[0], ranks(&[6]));
    assert_eq!(s.fronts[2], ranks(&[4, 5]));
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn an_attack_may_not_make_a_visible_pair() {
    let s = table(&[&[5, 6], &[8]], &[&[2], &[5]], &[9]);
    refused(&s, 0, attack(1, 5));
    refused(&s, 0, attack(1, 9));
    assert!(Auberge::legal_actions(&s, Seat(0)).contains(&attack(1, 6)));
}

#[test]
fn a_dare_that_does_not_pair_gives_the_turn_to_the_target() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, dare(2));
    assert_eq!(s.fronts[2], ranks(&[4, 9]));
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(
        s.events,
        vec![Event::Dared {
            seat: Seat(0),
            target: Seat(2),
            card: r(9),
            paired: false
        }]
    );
}

#[test]
fn a_dare_that_pairs_eliminates_the_darer() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[2], &[3], &[4]], &[3]);
    act(&mut s, 0, dare(1));
    assert!(s.out[0]);
    assert_eq!(s.fronts[1], ranks(&[3]));
    assert_eq!(s.turn, Some(Seat(1)));
    // The dared card and the darer's face-up cards are seen; the hand is not.
    assert_eq!(s.supply.seen(), ranks(&[3, 2]).as_slice());
    assert_eq!(s.supply.discard_len(), 3);
    assert!(s.hands[0].is_empty() && s.fronts[0].is_empty());
    assert!(s.events.contains(&Event::Out { seat: Seat(0) }));
}

#[test]
fn a_right_accusation_eliminates_the_accused() {
    // A pair between the hand and the face-up cards.
    let mut s = table(&[&[5], &[8, 3], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    assert!(s.out[1] && !s.out[0]);
    // Left of the accuser, skipping the eliminated player.
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(s.known[0], ranks(&[8, 3]));
    // A pair inside the hand counts too.
    let mut s = table(&[&[5], &[6, 6], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    assert!(s.out[1]);
}

#[test]
fn a_wrong_accusation_eliminates_the_accuser() {
    let mut s = table(&[&[5], &[8, 9], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    assert!(s.out[0] && !s.out[1]);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.hands[1], ranks(&[8, 9]));
}

#[test]
fn the_shown_hand_reaches_only_the_accuser_and_the_accused() {
    let mut s = table(&[&[5], &[8, 9], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    let shown = |viewer| {
        Auberge::view(&s, viewer)
            .events
            .into_iter()
            .find_map(|e| match e {
                Event::Accused { hand, guilty, .. } => Some((hand, guilty)),
                _ => None,
            })
            .expect("an accusation event")
    };
    assert_eq!(shown(Viewer::Seat(Seat(0))), (ranks(&[8, 9]), false));
    assert_eq!(shown(Viewer::Seat(Seat(1))), (ranks(&[8, 9]), false));
    assert_eq!(shown(Viewer::Seat(Seat(2))), (Vec::new(), false));
    assert_eq!(shown(Viewer::Spectator), (Vec::new(), false));
}

#[test]
fn a_player_without_hand_cannot_be_accused() {
    let s = table(&[&[5], &[], &[7]], &[&[2], &[3], &[4]], &[9]);
    refused(&s, 0, accuse(1));
    assert!(Auberge::legal_actions(&s, Seat(0)).contains(&accuse(2)));
}

#[test]
fn absurd_targets_are_refused() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[2], &[3], &[4]], &[9]);
    refused(&s, 0, dare(9));
    refused(&s, 0, dare(0));
    refused(&s, 0, accuse(0));
    refused(&s, 0, attack(9, 5));
    refused(&s, 1, Action::Draw);
    refused(&s, 9, Action::Draw);
    s.out[2] = true;
    refused(&s, 0, dare(2));
    refused(&s, 0, accuse(2));
}

#[test]
fn the_last_player_standing_wins_the_round_and_a_new_one_is_dealt() {
    let mut s = table(&[&[5], &[6, 6]], &[&[2], &[3]], &[9]);
    act(&mut s, 0, accuse(1));
    assert_eq!(s.wins, vec![1, 0]);
    assert_eq!(s.round, 2);
    assert_eq!(s.phase, Phase::Setup);
    assert!(s.hands.iter().all(|h| h.len() == 3));
    assert!(s.fronts.iter().all(Vec::is_empty));
    assert_eq!(s.out, vec![false, false]);
    assert!(s.known.iter().all(Vec::is_empty));
    assert_eq!(s.supply.deck_len(), DECK_SIZE - 6);
    assert!(s.events.contains(&Event::RoundWon {
        seat: Some(Seat(0))
    }));
    assert_eq!(Auberge::pending(&s).0.len(), 2);
}

#[test]
fn the_game_ends_after_the_last_round() {
    let mut s = table(&[&[5], &[6, 6]], &[&[2], &[3]], &[9]);
    s.rounds = 1;
    act(&mut s, 0, accuse(1));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Auberge::pending(&s), Pending::none());
    assert_eq!(Auberge::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
    refused(&s, 0, Action::Draw);
    refused(&s, 1, place(3));
}

#[test]
fn the_final_ranking_follows_the_rounds_won_and_keeps_ties() {
    let mut s = table(&[&[5], &[8], &[7], &[9]], &[&[], &[], &[], &[]], &[]);
    s.phase = Phase::Over;
    s.turn = None;
    s.wins = vec![2, 2, 0, 1];
    assert_eq!(
        Auberge::outcome(&s),
        Some(Outcome::Ranked(vec![0, 0, 3, 2]))
    );
    s.phase = Phase::Play;
    s.turn = Some(Seat(0));
    assert_eq!(Auberge::outcome(&s), None);
}

#[test]
fn a_stuck_round_is_won_by_nobody() {
    // No cards to draw; after this attack seat 0 can neither attack (its 5
    // would pair seat 1's) nor accuse (seat 1 has no hand).
    let mut s = table(&[&[5], &[9]], &[&[], &[5]], &[]);
    s.turn = Some(Seat(1));
    act(&mut s, 1, attack(0, 9));
    assert_eq!(s.wins, vec![0, 0]);
    assert_eq!(s.round, 2);
    assert_eq!(s.phase, Phase::Setup);
    assert!(s.events.contains(&Event::RoundWon { seat: None }));
}

#[test]
fn without_cards_to_draw_only_attacks_and_accusations_remain() {
    let s = table(&[&[5], &[9]], &[&[], &[]], &[]);
    let legal = Auberge::legal_actions(&s, Seat(0));
    assert_eq!(legal, vec![attack(1, 5), accuse(1)]);
}

#[test]
fn a_reshuffle_forgets_what_was_seen() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[], &[], &[]], &[]);
    s.supply = Supply::from_parts(Vec::new(), ranks(&[2]), ranks(&[4]), 0, 1);
    s.known[0] = ranks(&[4]);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.events[0], Event::Reshuffled);
    assert!(s.known[0].is_empty());
    assert_eq!(s.hands[0].len(), 2);
    assert_eq!(s.supply.deck_len(), 1);
}

#[test]
fn the_view_tells_a_player_about_their_own_pair_only() {
    let s = table(&[&[7, 3], &[8]], &[&[7], &[2]], &[9]);
    let mine = Auberge::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(mine.own_pair, Some(r(7)));
    assert_eq!(mine.hand, ranks(&[7, 3]));
    let theirs = Auberge::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(theirs.own_pair, None);
    assert_eq!(theirs.hand, ranks(&[8]));
    assert_eq!(theirs.seats[0].hand_size, 2);
    let watcher = Auberge::view(&s, Viewer::Spectator);
    assert!(watcher.hand.is_empty());
    assert_eq!(watcher.own_pair, None);
}

#[test]
fn unlocated_counts_what_the_viewer_cannot_place() {
    let mut s = table(&[&[7, 3], &[8]], &[&[7], &[2]], &[9]);
    s.known[0] = ranks(&[10]);
    // Seat 0 places: both face-up cards, its hand, and the 10 it saw.
    assert_eq!(
        s.unlocated_for(Some(Seat(0))),
        [1, 1, 2, 4, 5, 6, 5, 8, 9, 9]
    );
    assert_eq!(s.unlocated_for(None), [1, 1, 3, 4, 5, 6, 6, 8, 9, 10]);
}

#[test]
fn the_json_the_web_relies_on() {
    assert_eq!(serde_json::to_value(Action::Draw).unwrap(), json!("Draw"));
    assert_eq!(
        serde_json::to_value(place(3)).unwrap(),
        json!({"Place": {"card": 3}})
    );
    assert_eq!(
        serde_json::to_value(attack(1, 7)).unwrap(),
        json!({"Attack": {"target": 1, "card": 7}})
    );
    assert_eq!(
        serde_json::to_value(dare(2)).unwrap(),
        json!({"Dare": {"target": 2}})
    );
    assert_eq!(
        serde_json::to_value(accuse(2)).unwrap(),
        json!({"Accuse": {"target": 2}})
    );
    assert_eq!(
        serde_json::to_value(Event::Accused {
            seat: Seat(0),
            target: Seat(1),
            guilty: true,
            hand: ranks(&[6, 6])
        })
        .unwrap(),
        json!({"type": "Accused", "seat": 0, "target": 1, "guilty": true, "hand": [6, 6]})
    );
    assert_eq!(
        serde_json::to_value(Event::RoundWon { seat: None }).unwrap(),
        json!({"type": "RoundWon", "seat": null})
    );
    let s = table(&[&[7, 3], &[8]], &[&[7], &[2]], &[9]);
    let view = serde_json::to_value(Auberge::view(&s, Viewer::Seat(Seat(0)))).unwrap();
    assert_eq!(view["phase"], json!("Play"));
    assert_eq!(view["ownPair"], json!(7));
    assert_eq!(view["seats"][1]["handSize"], json!(1));
    assert_eq!(view["seats"][1]["wins"], json!(0));
    // No statistics in the view: counting cards is the players' job.
    assert!(view["seats"][1].get("pairOdds").is_none());
    assert!(view.get("unlocated").is_none());
    assert_eq!(view["discardSeen"], json!([]));
    assert_eq!(view["rounds"], json!(5));
    for hidden in ["supply", "hands", "known", "seed"] {
        assert!(view.get(hidden).is_none(), "{hidden} leaked");
    }
}

#[test]
fn a_wrongly_accused_player_is_known_to_hold_no_pair() {
    let mut s = table(
        &[&[5], &[1, 2, 3, 4, 6, 8, 9], &[7]],
        &[&[], &[], &[]],
        &[10, 10],
    );
    act(&mut s, 0, accuse(1));
    assert!(s.out[0]);
    let odds = |s: &State| s.pair_chance(Some(Seat(2)), Seat(1));
    // Everyone saw the accusation fail: seat 1 has no pair for now.
    assert_eq!(odds(&s), 0.0);
    // Playing a hand card away cannot give it one.
    act(&mut s, 1, attack(2, 9));
    assert_eq!(odds(&s), 0.0);
    // Receiving a card can.
    act(&mut s, 2, dare(1));
    assert!(odds(&s) > 0.0);
}

#[test]
fn drawing_ends_the_proof_of_innocence() {
    let mut s = table(&[&[5], &[1, 2, 3], &[7]], &[&[], &[], &[]], &[10, 10]);
    act(&mut s, 0, accuse(1));
    act(&mut s, 1, Action::Draw);
    assert!(s.pair_chance(Some(Seat(2)), Seat(1)) > 0.0);
}

/// The lot uses a fixed-width sample: identical on wasm32 and natively. A
/// change here breaks the replay of recorded matches.
#[test]
fn the_tie_break_of_a_seed_is_pinned() {
    let starters: Vec<u8> = (0..12)
        .map(|seed| {
            let mut s = setup(&[&[3, 6, 7], &[3, 8, 9], &[3, 5, 10]], &[1]);
            s.seed = seed;
            for seat in 0..3 {
                act(&mut s, seat, place(3));
            }
            s.turn.unwrap().0
        })
        .collect();
    assert_eq!(starters, vec![1, 0, 1, 2, 1, 0, 1, 2, 2, 2, 1, 1]);
}
