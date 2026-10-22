use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::bread_basket::cards::{full_deck, hand_size, Card};
use pairs::bread_basket::testing::{c, card, table};
use pairs::bread_basket::{Action, BreadBasket, Config, Event, HandEnd, Phase, Rounds, State};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

fn act(s: &mut State, seat: u8, action: Action) {
    BreadBasket::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn play(s: &mut State, seat: u8, value: i8) {
    act(s, seat, Action::Play { card: card(value) });
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        BreadBasket::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
}

/// The `HandOver` event of the last action.
fn hand_over(s: &State) -> (HandEnd, Vec<i32>, Vec<i32>, Vec<i32>) {
    s.events
        .iter()
        .find_map(|e| match e {
            Event::HandOver {
                end,
                gained,
                penalties,
                scores,
            } => Some((*end, gained.clone(), penalties.clone(), scores.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no HandOver in {:?}", s.events))
}

fn total_cards(s: &State) -> usize {
    s.deck.len()
        + s.discard.len()
        + s.line.len()
        + s.hands.iter().map(Vec::len).sum::<usize>()
        + s.taken.iter().map(Vec::len).sum::<usize>()
}

#[test]
fn deal_sizes_and_first_line_card() {
    for players in 2..=6u8 {
        let s = BreadBasket::new(&Config::default(), players, u64::from(players));
        let n = usize::from(players);
        assert!(s.hands.iter().all(|h| h.len() == hand_size(players)));
        assert_eq!(s.line.len(), 1);
        assert_eq!(s.deck.len(), 55 - n * hand_size(players) - 1);
        assert_eq!(total_cards(&s), full_deck().len());
        assert_eq!(s.turn, Seat((s.dealer.0 + 1) % players));
        assert!(matches!(s.events[0], Event::Dealt { round: 1, .. }));
    }
}

#[test]
fn a_sandwich_scores_its_filling_and_discards_the_bread() {
    let mut s = table(&[&[4, 9], &[1]], &[4, 3], &[6, 2]);
    play(&mut s, 0, 4);
    assert_eq!(s.taken[0], c(&[3]));
    assert_eq!(s.discard, c(&[4, 4]));
    assert_eq!(s.line, c(&[6]));
    assert_eq!(s.deck.len(), 1);
    assert_eq!(s.turn, Seat(1));
    assert!(s.events.contains(&Event::Sandwich {
        seat: Seat(0),
        bread: card(4),
        filling: c(&[3]),
        points: 3,
    }));
    assert!(s.events.contains(&Event::LineStarted { card: card(6) }));
}

#[test]
fn a_sandwich_can_take_only_the_end_of_the_line() {
    let mut s = table(&[&[9, 1], &[1]], &[8, 2, 9, 5], &[3]);
    play(&mut s, 0, 9);
    assert_eq!(s.line, c(&[8, 2]));
    assert_eq!(s.taken[0], c(&[5]));
}

#[test]
fn a_higher_card_between_blocks_the_sandwich() {
    let mut s = table(&[&[5, 1], &[1]], &[5, 3, 7], &[3]);
    play(&mut s, 0, 5);
    assert_eq!(s.line, c(&[5, 3, 7, 5]));
    assert!(s.taken[0].is_empty());
}

#[test]
fn an_air_sandwich_discards_and_forces_a_draw() {
    let mut s = table(&[&[7, 2], &[1]], &[3, 7], &[5, 6]);
    play(&mut s, 0, 7);
    assert_eq!(s.discard, c(&[7, 7]));
    assert_eq!(s.hands[0], c(&[2, 5]));
    assert_eq!(s.line, c(&[3]));
    assert_eq!(s.deck, c(&[6]));
    assert!(s.events.contains(&Event::Air {
        seat: Seat(0),
        card: card(7)
    }));
    assert!(s.events.contains(&Event::Drew { seat: Seat(0) }));
}

#[test]
fn an_air_sandwich_on_the_last_card_does_not_go_out() {
    let mut s = table(&[&[7], &[1]], &[3, 7], &[5, 6]);
    play(&mut s, 0, 7);
    assert_eq!(s.hands[0], c(&[5]));
    assert_eq!(s.out, None);
    assert!(!s.events.iter().any(|e| matches!(e, Event::WentOut { .. })));
}

#[test]
fn deck_runs_out_on_the_air_draw() {
    let mut s = table(&[&[7, 2], &[1]], &[7], &[]);
    play(&mut s, 0, 7);
    let (end, gained, penalties, scores) = hand_over(&s);
    assert_eq!(end, HandEnd::DeckEmpty);
    assert_eq!(gained, [0, 0]);
    assert_eq!(penalties, [2, 1]);
    assert_eq!(scores, [-2, -1]);
    assert_eq!(s.round_no, 2);
}

#[test]
fn deck_runs_out_when_restarting_the_line() {
    let mut s = table(&[&[4, 9], &[1]], &[4, 3], &[]);
    play(&mut s, 0, 4);
    let (end, gained, penalties, scores) = hand_over(&s);
    assert_eq!(end, HandEnd::DeckEmpty);
    assert_eq!(gained, [3, 0]);
    assert_eq!(penalties, [9, 1]);
    assert_eq!(scores, [-6, -1]);
}

#[test]
fn drawing_the_last_card_ends_the_hand() {
    let mut s = table(&[&[2], &[1]], &[8], &[5]);
    act(&mut s, 0, Action::Draw);
    let (end, _, penalties, _) = hand_over(&s);
    assert_eq!(end, HandEnd::DeckEmpty);
    assert_eq!(penalties, [7, 1]);
}

#[test]
fn going_out_gives_each_other_player_one_turn() {
    let mut s = table(&[&[2], &[3, 4], &[5, 6]], &[9], &[10, 10, 10, 10]);
    play(&mut s, 0, 2);
    assert_eq!(s.out, Some(Seat(0)));
    assert!(s.events.contains(&Event::WentOut { seat: Seat(0) }));
    act(&mut s, 1, Action::Draw);
    assert_eq!(s.round_no, 1);
    act(&mut s, 2, Action::Draw);
    let (end, _, penalties, _) = hand_over(&s);
    assert_eq!(end, HandEnd::WentOut);
    assert_eq!(penalties, [0, 17, 21]);
    assert_eq!(s.round_no, 2);
}

#[test]
fn going_out_with_a_sandwich_restarts_the_line_then_last_turns() {
    let mut s = table(&[&[9], &[1], &[1]], &[9, 3], &[8, 10, 10]);
    play(&mut s, 0, 9);
    let kinds: Vec<&str> = s
        .events
        .iter()
        .map(|e| match e {
            Event::Played { .. } => "played",
            Event::Sandwich { .. } => "sandwich",
            Event::LineStarted { .. } => "line",
            Event::WentOut { .. } => "out",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, ["played", "sandwich", "line", "out"]);
    assert_eq!(s.line, c(&[8]));
    // Seat 1 empties its hand too: nothing changes.
    play(&mut s, 1, 1);
    assert_eq!(s.turn, Seat(2));
    // Seat 2 makes an air sandwich and must draw.
    play(&mut s, 2, 1);
    let (end, gained, penalties, _) = hand_over(&s);
    assert_eq!(end, HandEnd::WentOut);
    assert_eq!(gained, [3, 0, 0]);
    assert_eq!(penalties, [0, 0, 10]);
}

#[test]
fn onions_make_scores_negative_and_rank_correctly() {
    let mut s = table(&[&[6, -5], &[1]], &[6, -5], &[]);
    s.round_no = 6;
    play(&mut s, 0, 6);
    let (_, gained, penalties, scores) = hand_over(&s);
    assert_eq!(gained, [-5, 0]);
    assert_eq!(penalties, [5, 1]);
    assert_eq!(scores, [-10, -1]);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(BreadBasket::outcome(&s), Some(Outcome::Ranked(vec![1, 0])));
    let json = serde_json::to_string(&BreadBasket::view(&s, Viewer::Spectator)).unwrap();
    assert!(json.contains("-10"), "{json}");
}

#[test]
fn hand_penalty_subtracts_ranks_and_onions() {
    let mut s = table(&[&[10, -5, 4], &[1]], &[4, 2], &[]);
    play(&mut s, 0, 4);
    let (_, gained, penalties, scores) = hand_over(&s);
    assert_eq!(gained, [2, 0]);
    assert_eq!(penalties, [15, 1]);
    assert_eq!(scores, [-13, -1]);
}

/// Plays random legal actions to the end; returns every event seen.
fn random_game(config: &Config, players: u8, seed: u64) -> (State, Vec<Event>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut s = BreadBasket::new(config, players, seed);
    let mut events = s.events.clone();
    while let Some(&seat) = BreadBasket::pending(&s).0.first() {
        let action = BreadBasket::legal_actions(&s, seat)
            .choose(&mut rng)
            .cloned()
            .expect("a legal action");
        BreadBasket::apply(&mut s, seat, &action).unwrap();
        assert_eq!(total_cards(&s), 55);
        events.extend(s.events.iter().cloned());
    }
    (s, events)
}

fn dealers(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Dealt { dealer, .. } => Some(dealer.0),
            _ => None,
        })
        .collect()
}

#[test]
fn six_rounds_by_default_and_the_deal_rotates() {
    for seed in 0..10 {
        let (s, events) = random_game(&Config::default(), 3, seed);
        let d = dealers(&events);
        assert_eq!(d.len(), 6);
        for pair in d.windows(2) {
            assert_eq!(pair[1], (pair[0] + 1) % 3);
        }
        assert_eq!(s.phase, Phase::Over);
        assert!(BreadBasket::outcome(&s).is_some());
    }
}

#[test]
fn per_player_plays_one_deal_each() {
    let config = Config {
        rounds: Rounds::PerPlayer,
    };
    let (s, events) = random_game(&config, 5, 7);
    assert_eq!(dealers(&events).len(), 5);
    assert_eq!(s.phase, Phase::Over);
}

#[test]
fn illegal_moves_are_refused() {
    let s = table(&[&[4, 9], &[1]], &[4, 3], &[6, 2]);
    refused(&s, 0, Action::Play { card: card(5) });
    refused(&s, 1, Action::Draw);
    let mut over = s.clone();
    over.phase = Phase::Over;
    refused(&over, 0, Action::Draw);
    assert!(BreadBasket::legal_actions(&over, Seat(0)).is_empty());
}

#[test]
fn legal_actions_are_one_play_per_value_and_a_draw() {
    let s = table(&[&[4, 9, 4, -5], &[1]], &[2], &[6]);
    let legal = BreadBasket::legal_actions(&s, Seat(0));
    assert_eq!(
        legal,
        [
            Action::Play { card: Card::ONION },
            Action::Play { card: card(4) },
            Action::Play { card: card(9) },
            Action::Draw,
        ]
    );
    assert!(BreadBasket::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn the_view_hides_other_hands_and_the_deck() {
    let s = BreadBasket::new(&Config::default(), 3, 5);
    let v = BreadBasket::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(v.hand, s.hands[1]);
    assert_eq!(v.seats[0].hand_size, s.hands[0].len());
    assert_eq!(v.deck_size, s.deck.len());
    assert_eq!(v.line, s.line);
    let spectator = BreadBasket::view(&s, Viewer::Spectator);
    assert!(spectator.hand.is_empty());
    let json = serde_json::to_string(&v).unwrap();
    assert!(!json.contains("\"deck\""), "{json}");
    assert!(!json.contains("seed"), "{json}");
}
