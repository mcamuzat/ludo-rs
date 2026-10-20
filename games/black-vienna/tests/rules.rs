use black_vienna::{deal, hand_sizes, suspects, CardId, Suspect, INVESTIGATIONS};
use ludo_core::Seat;

#[test]
fn every_suspect_is_on_four_investigations() {
    assert_eq!(INVESTIGATIONS.len(), 36);
    let mut seen = [0u8; 27];
    for id in 0..36u8 {
        let cards = suspects(CardId::new(id).unwrap());
        assert!(cards[0] != cards[1] && cards[1] != cards[2] && cards[0] != cards[2]);
        let letters: String = cards.iter().map(|s| s.letter()).collect();
        assert_eq!(letters, INVESTIGATIONS[usize::from(id)]);
        for s in cards {
            seen[s.index()] += 1;
        }
    }
    assert_eq!(seen, [4; 27]);
    assert!(CardId::new(36).is_none());
}

#[test]
fn hands_by_table_size() {
    assert_eq!(hand_sizes(3, Seat(0)), vec![8, 8, 8]);
    assert_eq!(hand_sizes(4, Seat(1)), vec![6; 4]);
    assert_eq!(hand_sizes(6, Seat(5)), vec![4; 6]);
    // [R] At five, the dealer gets four.
    assert_eq!(hand_sizes(5, Seat(2)), vec![5, 5, 4, 5, 5]);
}

#[test]
fn the_deal_splits_everything() {
    for players in 3..=6u8 {
        for seed in 0..10 {
            let dealer = Seat((seed % u64::from(players)) as u8);
            let d = deal(players, dealer, seed);
            let mut all: Vec<Suspect> = d.trio.to_vec();
            for (hand, size) in d.hands.iter().zip(hand_sizes(players, dealer)) {
                assert_eq!(hand.len(), size);
                all.extend(hand);
            }
            all.sort();
            all.dedup();
            assert_eq!(all.len(), 27);
            assert!(d.trio.windows(2).all(|w| w[0] < w[1]), "trio sorted");
            let mut cards: Vec<CardId> = d.piles.iter().flatten().copied().collect();
            assert!(d.piles.iter().all(|p| p.len() == 12));
            cards.sort();
            cards.dedup();
            assert_eq!(cards.len(), 36);
        }
    }
}

#[test]
fn suspects_serialize_as_letters() {
    let o = Suspect::new(26).unwrap();
    assert_eq!(o.letter(), 'Ö');
    assert_eq!(serde_json::to_string(&o).unwrap(), "\"Ö\"");
    assert_eq!(
        serde_json::from_str::<Suspect>("\"B\"").unwrap(),
        Suspect::new(1).unwrap()
    );
    assert!(serde_json::from_str::<Suspect>("\"0\"").is_err());
    assert!(serde_json::from_str::<Suspect>("\"ß\"").is_err());
    assert!(serde_json::from_str::<Suspect>("\"AB\"").is_err());
    assert!(Suspect::new(27).is_none());
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let d = deal(4, Seat(0), 7);
    let trio: String = d.trio.iter().map(|s| s.letter()).collect();
    let hand: String = d.hands[0].iter().map(|s| s.letter()).collect();
    assert_eq!((trio.as_str(), hand.as_str()), ("CPÖ", "FGMSYZ"));
}

// ---- Turns, investigations, the reserve (task 3) ----

use black_vienna::testing::{card, table, trio};
use black_vienna::{
    reusable, special_allowed, Action, Answer, BlackVienna, Config, Phase, State, TOKENS,
};
use ludo_core::{Game, Outcome};

fn act(s: &mut State, seat: u8, action: Action) {
    BlackVienna::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn ask(pile: u8, target: u8) -> Action {
    Action::Ask {
        pile: Some(pile),
        target: Seat(target),
        extra: None,
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        BlackVienna::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!BlackVienna::legal_actions(s, Seat(seat)).contains(&action));
}

/// Three players: seat 1 holds B but neither L nor M.
fn three() -> State {
    table(
        &["ACDEFGHI", "BJKNOPQR", "STUVWXYZ"],
        "LMÖ",
        ["ACL AGM BLM", "AOS APQ", "BCY BHV"],
    )
}

#[test]
fn a_new_game_starts_with_the_dealer() {
    for players in 3..=6u8 {
        let s = BlackVienna::new(&Config::default(), players, 11);
        assert_eq!(s.phase, Phase::Turn);
        assert_eq!(s.turn, s.dealer);
        assert_eq!(s.reserve, TOKENS);
        assert!(s.piles.iter().all(|p| p.len() == 12));
    }
}

#[test]
fn the_dealer_starts_and_the_answerer_plays_next() {
    let mut s = three();
    act(&mut s, 0, ask(0, 2));
    assert_eq!(s.turn, Seat(2));
    assert_eq!(s.piles[0].len(), 2);
}

#[test]
fn an_answer_counts_the_suspects_in_hand() {
    let mut s = three();
    act(&mut s, 0, ask(0, 1));
    assert_eq!(
        s.answers,
        vec![Answer {
            by: Seat(0),
            target: Seat(1),
            card: card("BLM"),
            count: 1,
            placed: 1,
            special: false,
        }]
    );
    assert_eq!(s.reserve, TOKENS - 1);
}

#[test]
fn one_cannot_ask_oneself() {
    let s = three();
    refused(&s, 0, ask(0, 0));
    refused(&s, 1, ask(0, 2));
}

#[test]
fn special_needs_everyone_asked_and_six_cards() {
    let mut s = three();
    let zero = |target: u8, name: &str| Answer {
        by: Seat(0),
        target: Seat(target),
        card: card(name),
        count: 0,
        placed: 0,
        special: false,
    };
    s.answers = vec![
        zero(1, "DHR"),
        zero(2, "DJZ"),
        zero(1, "DLS"),
        zero(2, "DVY"),
        zero(1, "EGW"),
    ];
    s.asked = vec![false, true, true];
    assert!(!special_allowed(&s));
    s.answers.push(Answer {
        count: 1,
        placed: 1,
        ..zero(0, "ENQ")
    });
    assert!(!special_allowed(&s), "seat 0 never asked");
    s.asked[0] = true;
    assert!(special_allowed(&s));
    assert!(!reusable(&s).contains(&card("ENQ")));
    let special = Action::Ask {
        pile: Some(0),
        target: Seat(2),
        extra: Some(card("DHR")),
    };
    assert!(BlackVienna::legal_actions(&s, Seat(0)).contains(&special));
    refused(
        &s,
        0,
        Action::Ask {
            pile: Some(0),
            target: Seat(2),
            extra: Some(card("ENQ")),
        },
    );
    act(&mut s, 0, special);
    // [R] The old card now lies before seat 2, with its new answer (❓2).
    let last = s.answers.last().unwrap();
    assert_eq!(
        (last.card, last.target, last.special, last.count),
        (card("DHR"), Seat(2), true, 0)
    );
    assert_eq!(s.turn, Seat(2));
}

#[test]
fn a_short_reserve_gives_what_is_left_and_ends_the_game() {
    // Seat 2 holds S and X (CSX: 2) and U and W (OUW: 2); 5 tokens left.
    let mut s = table(
        &["ACDEFGHI", "BJKNOPQR", "STUVWXYZ"],
        "LMÖ",
        ["CSX", "AOS", "BCY"],
    );
    s.reserve = 5;
    s.asked = vec![true, true, true];
    s.answers = ["DHR", "DJZ", "DLS", "DVY", "EGW", "ENQ"]
        .iter()
        .map(|name| Answer {
            by: Seat(0),
            target: Seat(1),
            card: card(name),
            count: 0,
            placed: 0,
            special: false,
        })
        .collect();
    // OUW was answered with no token, so it can be laid again.
    s.answers.push(Answer {
        by: Seat(0),
        target: Seat(0),
        card: card("OUW"),
        count: 0,
        placed: 0,
        special: false,
    });
    act(
        &mut s,
        0,
        Action::Ask {
            pile: Some(0),
            target: Seat(2),
            extra: Some(card("OUW")),
        },
    );
    let placed: u8 = s.answers.iter().map(|a| a.placed).sum();
    let counts: Vec<u8> = s.answers.iter().rev().take(2).map(|a| a.count).collect();
    assert_eq!(counts, vec![2, 2], "true counts kept (❓6)");
    assert_eq!((placed, s.reserve), (4, 1));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(
        BlackVienna::outcome(&s),
        Some(Outcome::Ranked(vec![0, 0, 0]))
    );
    assert_eq!(s.scores, vec![0, 0, 0]);
}

#[test]
fn three_tokens_left_ends_without_winner() {
    let mut s = three();
    s.reserve = 4;
    act(&mut s, 0, ask(0, 1));
    assert_eq!((s.reserve, s.phase), (3, Phase::Over));
}

#[test]
fn empty_piles_leave_only_specials_or_the_end() {
    // ❓7: piles empty, no reusable card, and seat 1 already accused wrongly.
    let mut s = table(
        &["ACDEFGHI", "BJKNOPQR", "STUVWXYZ"],
        "LMÖ",
        ["BLM", "", ""],
    );
    s.wrong = vec![false, true, false];
    act(&mut s, 0, ask(0, 1));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.scores, vec![0, 0, 0]);
    let _ = trio("ABC");
}

// ---- Accusations (task 4) ----

use black_vienna::{End, Event};

fn accuse(text: &str) -> Action {
    Action::Accuse { trio: trio(text) }
}

fn guess(text: &str) -> Action {
    Action::Guess { trio: trio(text) }
}

fn accusations(s: &State, seat: u8) -> usize {
    BlackVienna::legal_actions(s, Seat(seat))
        .iter()
        .filter(|a| matches!(a, Action::Accuse { .. }))
        .count()
}

#[test]
fn accusing_is_offered_only_on_ones_turn() {
    let mut s = three();
    assert_eq!(accusations(&s, 0), 2925);
    assert_eq!(accusations(&s, 1), 0);
    act(&mut s, 0, ask(0, 1));
    assert_eq!(accusations(&s, 0), 0);
    assert_eq!(accusations(&s, 1), 2925);
}

#[test]
fn a_trio_is_sorted_and_distinct() {
    let s = three();
    let [a, b, c] = trio("ABC");
    refused(&s, 0, Action::Accuse { trio: [c, a, b] });
    refused(&s, 0, Action::Accuse { trio: [a, a, b] });
}

#[test]
fn a_wrong_accuser_keeps_playing_without_accusing() {
    let mut s = three();
    act(&mut s, 0, accuse("ABC"));
    assert_eq!(
        s.events,
        vec![Event::Accused {
            seat: Seat(0),
            right: false
        }]
    );
    assert!(s.wrong[0]);
    // Still seat 0's turn: it must ask, and may not accuse again.
    assert_eq!(
        (s.phase, s.turn, accusations(&s, 0)),
        (Phase::Turn, Seat(0), 0)
    );
    act(&mut s, 0, ask(0, 1));
    act(&mut s, 1, ask(0, 0));
    assert_eq!(s.turn, Seat(0));
    assert_eq!(accusations(&s, 0), 0);
}

#[test]
fn a_right_accusation_asks_the_others_for_a_trio() {
    let mut s = table(
        &["ACDEFGHI", "BJKNOPQR", "STUVWXYZ", ""],
        "LMÖ",
        ["ACL AGM BLM", "AOS APQ", "BCY BHV"],
    );
    s.players = 4;
    s.hands[3] = black_vienna::testing::letters("");
    s.asked = vec![false; 4];
    s.wrong = vec![false, false, true, false];
    s.guesses = vec![None; 4];
    s.scores = vec![0; 4];
    s.reserve = 30;
    act(&mut s, 0, accuse("LMÖ"));
    assert_eq!(s.phase, Phase::Guess);
    assert_eq!(BlackVienna::pending(&s).0, vec![Seat(1), Seat(3)]);
    act(&mut s, 1, guess("LMÖ"));
    assert_eq!(s.phase, Phase::Guess);
    act(&mut s, 3, guess("ABC"));
    assert_eq!(s.phase, Phase::Over);
    // [R] 3 per token for the accuser, 1 per token for a right guess.
    assert_eq!(s.scores, vec![90, 30, 0, 0]);
    assert_eq!(
        BlackVienna::outcome(&s),
        Some(Outcome::Ranked(vec![0, 1, 2, 2]))
    );
    assert!(matches!(
        s.events.last(),
        Some(Event::Revealed {
            end: End::Solved,
            accuser: Some(Seat(0)),
            ..
        })
    ));
}

#[test]
fn the_last_player_may_ask_once_then_must_accuse() {
    let mut s = three();
    s.wrong = vec![false, true, false];
    s.turn = Seat(2);
    act(&mut s, 2, accuse("ABC"));
    // ❓4: interrupted at once; seat 0 is the last player.
    assert_eq!((s.last, s.turn), (Some(Seat(0)), Seat(0)));
    s.reserve = 4;
    act(&mut s, 0, ask(0, 1));
    // ❓5: the final question escapes the reserve rule.
    assert_eq!((s.phase, s.turn, s.reserve), (Phase::Turn, Seat(0), 3));
    let legal = BlackVienna::legal_actions(&s, Seat(0));
    assert!(legal.iter().all(|a| matches!(a, Action::Accuse { .. })));
    act(&mut s, 0, accuse("LMÖ"));
    // [R] One point per token left.
    assert_eq!((s.phase, s.scores.clone()), (Phase::Over, vec![3, 0, 0]));
}

#[test]
fn the_last_player_may_accuse_at_once_and_fail() {
    let mut s = three();
    s.wrong = vec![false, true, false];
    s.turn = Seat(2);
    act(&mut s, 2, accuse("ABC"));
    act(&mut s, 0, accuse("ABD"));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.scores, vec![0, 0, 0]);
    assert!(matches!(
        s.events.last(),
        Some(Event::Revealed {
            end: End::Failed,
            ..
        })
    ));
}
