use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::{Supply, DECK_SIZE};
use pairs::ghost_town::testing::{setup, table};
use pairs::ghost_town::{Action, Config, Event, GhostTown, Phase, State};
use pairs::testing::{r, ranks};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde_json::json;

fn act(s: &mut State, seat: u8, action: Action) {
    GhostTown::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn place(card: u8) -> Action {
    Action::Place { card: r(card) }
}

fn play(card: u8, to: u8) -> Action {
    Action::Play {
        card: r(card),
        to: Seat(to),
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        GhostTown::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!GhostTown::legal_actions(s, Seat(seat)).contains(&action));
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot)
}

fn cards(s: &State) -> usize {
    s.hands.iter().chain(&s.houses).map(Vec::len).sum::<usize>()
        + s.street.len()
        + s.supply.deck_len()
        + s.supply.discard_len()
}

#[test]
fn a_new_game_antes_and_deals_five_cards() {
    for players in 2..=6u8 {
        let s = GhostTown::new(&Config::default(), players, 3);
        assert_eq!(s.phase, Phase::Setup);
        assert!(s.hands.iter().all(|h| h.len() == 5));
        assert_eq!(s.pot, 10 * u32::from(players));
        assert!(s.chips.iter().all(|&c| c == -10));
        assert_eq!(GhostTown::pending(&s).0.len(), usize::from(players));
        assert_eq!((s.hand_no, s.hands_total), (1, 5));
        assert_eq!(cards(&s), DECK_SIZE);
    }
}

#[test]
fn the_number_of_hands_comes_from_the_config_within_bounds() {
    let hands = |h| GhostTown::new(&Config { hands: h }, 2, 1).hands_total;
    assert_eq!(hands(None), 5);
    assert_eq!(hands(Some(0)), 5);
    assert_eq!(hands(Some(2)), 2);
    assert_eq!(hands(Some(255)), 15);
}

#[test]
fn a_placed_card_stays_secret_until_everyone_has_placed() {
    let mut s = setup(&[&[5, 6, 7, 8, 9], &[2, 3, 4, 8, 9]], &[1]);
    act(&mut s, 0, place(5));
    let other = GhostTown::view(&s, Viewer::Seat(Seat(1)));
    assert!(other.street.is_empty());
    assert_eq!(other.seats[0].hand_size, 5);
    assert_eq!(
        GhostTown::view(&s, Viewer::Seat(Seat(0))).placed,
        Some(r(5))
    );
    assert_eq!(
        GhostTown::visible_pending(&s, Viewer::Seat(Seat(0))),
        Pending::none()
    );
    assert_eq!(
        GhostTown::visible_pending(&s, Viewer::Seat(Seat(1))),
        Pending::one(Seat(1))
    );
    refused(&s, 0, place(6));
    refused(&s, 1, Action::Draw);
}

#[test]
fn the_placed_cards_make_the_street_and_the_lowest_starts() {
    let mut s = setup(
        &[&[5, 6, 7, 8, 9], &[2, 3, 4, 8, 9], &[4, 6, 7, 10, 10]],
        &[1],
    );
    act(&mut s, 0, place(5));
    act(&mut s, 1, place(2));
    act(&mut s, 2, place(4));
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.street, ranks(&[5, 2, 4]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert!(s.hands.iter().all(|h| h.len() == 4));
    assert!(s.houses.iter().all(Vec::is_empty));
    assert!(s.events.contains(&Event::Starts { seat: Seat(1) }));
}

#[test]
fn duplicates_in_the_street_are_discarded() {
    let mut s = setup(
        &[&[5, 6, 7, 8, 9], &[2, 3, 4, 8, 9], &[5, 6, 7, 10, 10]],
        &[1],
    );
    act(&mut s, 0, place(5));
    act(&mut s, 1, place(2));
    act(&mut s, 2, place(5));
    assert_eq!(s.street, ranks(&[5, 2]));
    assert_eq!(s.supply.seen(), ranks(&[5]).as_slice());
}

#[test]
fn a_tie_for_lowest_is_broken_with_cards_from_the_deck() {
    let mut s = setup(
        &[&[3, 6, 7, 8, 9], &[3, 4, 5, 8, 9], &[9, 6, 7, 10, 10]],
        &[8, 4, 1],
    );
    act(&mut s, 0, place(3));
    act(&mut s, 1, place(3));
    act(&mut s, 2, place(9));
    // Seat 0 drew the 8, seat 1 the 4: seat 1 starts, both cards are discarded.
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.street, ranks(&[3, 9]));
    assert_eq!(s.supply.seen(), ranks(&[3, 8, 4]).as_slice());
    assert_eq!(s.supply.deck_len(), 1);
    assert!(s.events.contains(&Event::TieBreak {
        seat: Seat(1),
        card: r(4)
    }));
}

#[test]
fn playing_a_card_costs_two_chips_and_passes_left() {
    let mut s = table(&[&[6, 7], &[8], &[9]], &[&[], &[], &[]], &[2], &[1]);
    act(&mut s, 0, play(6, 0));
    assert_eq!(s.houses[0], ranks(&[6]));
    assert_eq!(s.hands[0], ranks(&[7]));
    assert_eq!((s.chips[0], s.pot), (-12, 32));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![Event::Played {
            seat: Seat(0),
            to: Seat(0),
            card: r(6)
        }]
    );
}

#[test]
fn a_house_takes_no_pair_and_no_card_of_the_street() {
    let s = table(&[&[6, 7, 2], &[8]], &[&[6], &[]], &[2], &[1]);
    // A second 6 at home, a 2 like the street's.
    refused(&s, 0, play(6, 0));
    refused(&s, 0, play(2, 0));
    assert!(GhostTown::legal_actions(&s, Seat(0)).contains(&play(7, 0)));
}

#[test]
fn a_rank_you_hold_at_home_may_be_played_on_your_left_neighbour() {
    let mut s = table(&[&[6, 7], &[8], &[9]], &[&[6], &[], &[]], &[2], &[1]);
    // Only a rank already in my house, only to the left, only if they lack it.
    refused(&s, 0, play(7, 1));
    refused(&s, 0, play(6, 2));
    act(&mut s, 0, play(6, 1));
    assert_eq!(s.houses[1], ranks(&[6]));
    assert_eq!((s.chips[0], s.pot), (-12, 32));
    assert_eq!(s.turn, Some(Seat(1)));
    let s = table(&[&[6, 7], &[8]], &[&[6], &[6]], &[2], &[1]);
    refused(&s, 0, play(6, 1));
}

#[test]
fn the_neighbour_is_the_next_player_still_in() {
    let mut s = table(&[&[6, 7], &[8], &[9]], &[&[6], &[], &[]], &[2], &[1]);
    s.out[1] = true;
    refused(&s, 0, play(6, 1));
    assert!(GhostTown::legal_actions(&s, Seat(0)).contains(&play(6, 2)));
    assert_eq!(
        GhostTown::view(&s, Viewer::Seat(Seat(0))).neighbour,
        Some(Seat(2))
    );
}

#[test]
fn a_clean_draw_wins_the_card_from_the_pot() {
    let mut s = table(&[&[6], &[8], &[9]], &[&[3], &[], &[]], &[2], &[7]);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.houses[0], ranks(&[3, 7]));
    assert_eq!((s.chips[0], s.pot), (-3, 23));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![Event::Drew {
            seat: Seat(0),
            card: r(7),
            won: 7
        }]
    );
}

#[test]
fn a_draw_that_pairs_the_house_busts() {
    let mut s = table(&[&[6], &[8], &[9]], &[&[3], &[], &[]], &[2], &[3]);
    act(&mut s, 0, Action::Draw);
    assert!(s.out[0]);
    assert_eq!((s.chips[0], s.pot), (-13, 33));
    assert!(s.houses[0].is_empty() && s.hands[0].is_empty());
    assert_eq!(s.turn, Some(Seat(1)));
    assert!(s.events.contains(&Event::Busted {
        seat: Seat(0),
        card: r(3)
    }));
    // The house and the drawn card are seen, the hand is not.
    assert_eq!(s.supply.seen(), ranks(&[3, 3]).as_slice());
    assert_eq!(s.supply.discard_len(), 3);
}

#[test]
fn a_draw_that_matches_the_street_busts() {
    let mut s = table(&[&[6], &[8], &[9]], &[&[3], &[], &[]], &[2], &[2]);
    act(&mut s, 0, Action::Draw);
    assert!(s.out[0]);
    assert_eq!((s.chips[0], s.pot), (-12, 32));
}

#[test]
fn a_draw_takes_no_more_than_the_pot_holds() {
    let mut s = table(&[&[6], &[8]], &[&[3], &[]], &[2], &[9]);
    s.pot = 4;
    s.chips = vec![-2, -2];
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.chips[0], 2 - 10);
    assert!(s.events.contains(&Event::Drew {
        seat: Seat(0),
        card: r(9),
        won: 4
    }));
    // The pot was emptied: the hand is over, nobody collects, a new one starts.
    assert!(s.events.contains(&Event::HandEnded {
        winner: None,
        pot: 0
    }));
    assert_eq!(s.hand_no, 2);
    assert_eq!(s.phase, Phase::Setup);
    assert_eq!(balance(&s), 0);
}

#[test]
fn folding_leaves_the_hand_and_the_last_player_takes_the_pot() {
    let mut s = table(&[&[6], &[8], &[9]], &[&[3], &[4], &[]], &[2], &[7]);
    act(&mut s, 0, Action::Fold);
    assert!(s.out[0]);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.pot, 30);
    assert_eq!(s.supply.seen(), ranks(&[3]).as_slice());
    act(&mut s, 1, Action::Fold);
    // Seat 2 is alone: it collects the 30 chips and the next hand is dealt.
    assert!(s.events.contains(&Event::HandEnded {
        winner: Some(Seat(2)),
        pot: 30
    }));
    assert_eq!(s.chips, vec![-20, -20, 10]);
    assert_eq!((s.hand_no, s.pot), (2, 30));
    assert_eq!(balance(&s), 0);
    assert_eq!(cards(&s), DECK_SIZE);
}

#[test]
fn a_bust_can_leave_one_player_who_takes_everything() {
    let mut s = table(&[&[6], &[8]], &[&[3], &[]], &[2], &[3]);
    act(&mut s, 0, Action::Draw);
    // Seat 0 paid 3 more, then seat 1 collected 23.
    assert_eq!(s.chips, vec![-23, 3]);
    assert_eq!(balance(&s), 0);
}

#[test]
fn the_game_ends_after_the_last_hand_ranked_by_chips() {
    let mut s = table(&[&[6], &[8], &[9]], &[&[], &[], &[]], &[2], &[7]);
    s.hands_total = 1;
    act(&mut s, 0, Action::Fold);
    act(&mut s, 1, Action::Fold);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.chips, vec![-10, -10, 20]);
    assert_eq!(GhostTown::pending(&s), Pending::none());
    assert_eq!(GhostTown::outcome(&s), Some(Outcome::Ranked(vec![1, 1, 0])));
    refused(&s, 2, Action::Draw);
}

#[test]
fn absurd_plays_are_refused() {
    let s = table(&[&[6, 7], &[8], &[9]], &[&[6], &[], &[]], &[2], &[1]);
    refused(&s, 0, play(6, 9));
    refused(&s, 0, play(9, 0));
    refused(&s, 0, place(6));
    refused(&s, 1, Action::Draw);
    refused(&s, 9, Action::Fold);
}

#[test]
fn without_cards_to_draw_one_can_still_fold() {
    let s = table(&[&[], &[8]], &[&[3], &[]], &[2], &[]);
    assert_eq!(GhostTown::legal_actions(&s, Seat(0)), vec![Action::Fold]);
}

#[test]
fn chips_and_cards_are_conserved() {
    let mut rng = SmallRng::seed_from_u64(9);
    for players in 2..=6u8 {
        for _ in 0..20 {
            let mut s = GhostTown::new(&Config { hands: Some(2) }, players, rng.gen());
            let mut steps = 0;
            while let Some(&seat) = GhostTown::pending(&s).0.first() {
                let legal = GhostTown::legal_actions(&s, seat);
                let action = legal.choose(&mut rng).expect("a legal action").clone();
                assert!(!s.out[seat.index()], "an out seat is asked to move");
                act(&mut s, seat.0, action);
                assert_eq!(balance(&s), 0);
                assert_eq!(cards(&s), DECK_SIZE);
                for house in &s.houses {
                    let mut sorted = house.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    assert_eq!(sorted.len(), house.len(), "a pair in a house");
                    assert!(house.iter().all(|c| !s.street.contains(c)));
                }
                steps += 1;
                assert!(steps < 5000);
            }
            assert_eq!(s.pot, 0);
        }
    }
}

#[test]
fn a_seat_sees_its_own_hand_and_no_statistics() {
    let s = table(&[&[6, 7], &[8], &[9]], &[&[3], &[], &[]], &[2], &[1]);
    let mine = GhostTown::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(mine.hand, ranks(&[6, 7]));
    assert_eq!(mine.neighbour, Some(Seat(1)));
    assert_eq!(mine.seats[1].hand_size, 1);
    assert_eq!((mine.pot, mine.ante, mine.play_cost), (30, 10, 2));
    assert!(GhostTown::view(&s, Viewer::Spectator).hand.is_empty());
    let json = serde_json::to_value(&mine).unwrap();
    for hidden in ["supply", "hands", "seed", "unlocated"] {
        assert!(json.get(hidden).is_none(), "{hidden} in the view");
    }
}

#[test]
fn the_json_the_web_relies_on() {
    assert_eq!(serde_json::to_value(Action::Draw).unwrap(), json!("Draw"));
    assert_eq!(serde_json::to_value(Action::Fold).unwrap(), json!("Fold"));
    assert_eq!(
        serde_json::to_value(place(3)).unwrap(),
        json!({"Place": {"card": 3}})
    );
    assert_eq!(
        serde_json::to_value(play(7, 1)).unwrap(),
        json!({"Play": {"card": 7, "to": 1}})
    );
    assert_eq!(
        serde_json::to_value(Event::HandEnded {
            winner: None,
            pot: 0
        })
        .unwrap(),
        json!({"type": "HandEnded", "winner": null, "pot": 0})
    );
    let s = table(&[&[6, 7], &[8]], &[&[3], &[]], &[2], &[1]);
    let view = serde_json::to_value(GhostTown::view(&s, Viewer::Seat(Seat(0)))).unwrap();
    assert_eq!(view["phase"], json!("Play"));
    assert_eq!(view["handNo"], json!(1));
    assert_eq!(view["handsTotal"], json!(5));
    assert_eq!(view["seats"][0]["house"], json!([3]));
    assert_eq!(view["seats"][1]["handSize"], json!(1));
    assert_eq!(view["seats"][1]["chips"], json!(-10));
    assert_eq!(view["playCost"], json!(2));
    assert_eq!(view["discardSeen"], json!([]));
}

#[test]
fn a_tie_break_stops_when_the_draw_pile_runs_out() {
    // Both draw a 4, then the pile is empty: the first tied player starts,
    // without reshuffling the tie-break cards back in.
    let mut s = setup(&[&[3, 6, 7, 8, 9], &[3, 4, 5, 8, 9]], &[4, 4]);
    act(&mut s, 0, place(3));
    act(&mut s, 1, place(3));
    assert_eq!(s.turn, Some(Seat(0)));
    assert!(!s.events.contains(&Event::Reshuffled));
    assert_eq!(s.supply.seen(), ranks(&[3, 4, 4]).as_slice());
}

#[test]
fn a_second_tie_deals_another_round_to_those_still_tied() {
    let mut s = setup(
        &[&[3, 6, 7, 8, 9], &[3, 4, 5, 8, 9], &[3, 6, 7, 10, 10]],
        &[5, 5, 9, 8, 2, 1],
    );
    for seat in 0..3 {
        act(&mut s, seat, place(3));
    }
    // 5, 5, 9: seats 0 and 1 tie again; then 8 and 2: seat 1 starts.
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.supply.deck_len(), 1);
}

#[test]
fn the_turn_skips_players_who_left_the_hand() {
    let mut s = table(&[&[6], &[8], &[9]], &[&[], &[], &[]], &[2], &[7, 4]);
    act(&mut s, 0, Action::Fold);
    act(&mut s, 1, Action::Draw);
    assert_eq!(s.turn, Some(Seat(2)));
    act(&mut s, 2, Action::Draw);
    assert_eq!(s.turn, Some(Seat(1)));
    assert!(GhostTown::legal_actions(&s, Seat(0)).is_empty());
}

#[test]
fn an_empty_draw_pile_is_rebuilt_from_the_discards() {
    let mut s = table(&[&[6], &[8]], &[&[3], &[]], &[2], &[]);
    s.supply = Supply::from_parts(Vec::new(), ranks(&[9]), Vec::new(), 0, 1);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.events[0], Event::Reshuffled);
    assert_eq!(s.houses[0], ranks(&[3, 9]));
    assert_eq!(s.chips[0], -1);
}

#[test]
fn the_draw_value_reads_the_discards_when_the_pile_is_empty() {
    // The only cards a draw can bring are the two 3s in the discards, and
    // seat 0 has a 3 at home: a certain bust.
    let mut s = table(&[&[], &[8]], &[&[3], &[]], &[2], &[]);
    s.supply = Supply::from_parts(Vec::new(), ranks(&[3, 3]), Vec::new(), 0, 1);
    assert_eq!(s.draw_value(Seat(0)), -3.0);
}
