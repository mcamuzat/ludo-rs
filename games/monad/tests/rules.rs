use std::collections::{HashSet, VecDeque};

use ludo_core::{Game, Outcome, Seat, Viewer};
use monad::{Action, Card, Color, Config, Monad, Rank, State, Target};
use serde_json::json;

use Color::*;

fn state(players: u8) -> State {
    Monad::new(&Config::default(), players, 7)
}

fn card(rank: Rank, color: Color) -> Card {
    Card { rank, color }
}

fn units(colors: &[Color]) -> Vec<Card> {
    colors.iter().map(|&c| card(Rank::Unit, c)).collect()
}

/// A two-player table where seat 0 is to play with `hand`, seat 1 holds
/// `other`; the deck and discard are empty, columns full.
fn table(hand: Vec<Card>, other: Vec<Card>) -> State {
    let mut s = state(2);
    let mut hand = hand;
    hand.sort();
    let mut other = other;
    other.sort();
    s.hands = vec![hand, other];
    s.current = Seat(0);
    s.identities = vec![Yellow, Sky];
    s.columns = std::array::from_fn(|_| VecDeque::from(Color::ALL.to_vec()));
    s.deck.clear();
    s.discard.clear();
    s
}

fn snapshot(s: &State) -> serde_json::Value {
    serde_json::to_value(s).unwrap()
}

#[test]
fn setup_follows_the_player_count() {
    for (players, per_color, identities) in [
        (2u8, 2usize, None),
        (3, 3, Some(vec![Navy, Sky, Green])),
        (4, 4, Some(vec![Red, Orange, Navy, Sky])),
    ] {
        let s = state(players);
        assert_eq!(s.hands.len(), usize::from(players));
        for hand in &s.hands {
            assert_eq!(hand.len(), 6);
            assert!(hand.iter().all(|c| c.rank == Rank::Unit));
        }
        for color in Color::ALL {
            let n = s
                .hands
                .iter()
                .flatten()
                .filter(|c| c.color == color)
                .count();
            assert_eq!(n, per_color, "{players} joueurs, {color:?}");
        }
        let ids: HashSet<Color> = s.identities.iter().copied().collect();
        assert_eq!(ids.len(), usize::from(players));
        if let Some(expected) = identities {
            assert_eq!(ids, expected.into_iter().collect());
        }
        for column in &s.columns {
            let colors: HashSet<Color> = column.iter().copied().collect();
            assert_eq!(colors.len(), 6);
            assert_eq!(column.len(), 6);
        }
        assert_eq!(s.monad_stock, 10);
        assert_eq!(s.monads, vec![0; usize::from(players)]);
        assert!(s.deck.is_empty() && s.discard.is_empty());
    }
}

#[test]
fn setup_comes_from_the_seed() {
    let a = Monad::new(&Config::default(), 4, 99);
    let b = Monad::new(&Config::default(), 4, 99);
    assert_eq!(snapshot(&a), snapshot(&b));
    let firsts: HashSet<u8> = (0..200)
        .map(|seed| Monad::new(&Config::default(), 4, seed).current.0)
        .collect();
    assert_eq!(firsts.len(), 4);
}

#[test]
fn draw_flip_and_end_of_turn() {
    let mut s = table(units(&[Red, Navy]), units(&[Orange]));
    s.deck = vec![Red, Green];
    assert!(Monad::legal_actions(&s, Seat(0)).contains(&Action::Draw));
    assert!(!Monad::legal_actions(&s, Seat(0)).contains(&Action::Flip));
    Monad::apply(&mut s, Seat(0), &Action::Draw).unwrap();
    assert!(s.hands[0].contains(&card(Rank::Unit, Green)));
    assert_eq!(s.deck, vec![Red]);
    assert_eq!(s.current, Seat(1));

    let mut s = table(units(&[Red, Navy]), units(&[Orange]));
    s.discard = vec![Red, Sky];
    let legal = Monad::legal_actions(&s, Seat(0));
    assert!(legal.contains(&Action::Flip) && !legal.contains(&Action::Draw));
    Monad::apply(&mut s, Seat(0), &Action::Flip).unwrap();
    assert!(s.discard.is_empty());
    assert_eq!(s.current, Seat(1));
    // The first unit discarded is the first drawn.
    s.current = Seat(0);
    Monad::apply(&mut s, Seat(0), &Action::Draw).unwrap();
    assert!(s.hands[0].iter().filter(|c| c.color == Red).count() == 2);
}

#[test]
fn no_draw_after_a_transaction() {
    let mut s = table(units(&[Red, Navy, Orange]), units(&[Orange]));
    s.deck = vec![Green];
    let before = Monad::legal_actions(&s, Seat(0));
    assert!(!before.contains(&Action::EndTurn));
    let trade = Action::Trade {
        cards: [card(Rank::Unit, Red), card(Rank::Unit, Navy)],
    };
    assert!(before.contains(&trade));
    Monad::apply(&mut s, Seat(0), &trade).unwrap();
    let after = Monad::legal_actions(&s, Seat(0));
    assert!(after.contains(&Action::EndTurn));
    for whole_turn in [Action::Draw, Action::Flip, Action::Pass] {
        assert!(!after.contains(&whole_turn));
    }
}

#[test]
fn the_view_hides_other_hands() {
    let s = state(2);
    let v0 = serde_json::to_value(Monad::view(&s, Viewer::Seat(Seat(0)))).unwrap();
    let v1 = serde_json::to_value(Monad::view(&s, Viewer::Seat(Seat(1)))).unwrap();
    let spectator = serde_json::to_value(Monad::view(&s, Viewer::Spectator)).unwrap();
    assert_eq!(v0["hand"], serde_json::to_value(&s.hands[0]).unwrap());
    assert_eq!(v1["hand"], serde_json::to_value(&s.hands[1]).unwrap());
    assert_eq!(spectator["hand"], json!(null));
    assert_eq!(v0["handCounts"][1], json!([6, 0, 0, 0, 0]));
    assert_eq!(v0["target"], json!(5));
}

#[test]
fn config_reads_the_lobby_json() {
    assert_eq!(
        serde_json::from_value::<Config>(json!({})).unwrap(),
        Config::default()
    );
}

fn trade(a: Card, b: Card) -> Action {
    let mut cards = [a, b];
    cards.sort();
    Action::Trade { cards }
}

fn buy(target: Target, cards: Vec<Card>) -> Action {
    let mut cards = cards;
    cards.sort();
    Action::Buy { target, cards }
}

fn legal(s: &State) -> Vec<Action> {
    Monad::legal_actions(s, s.current)
}

fn play(s: &mut State, action: Action) {
    let seat = s.current;
    Monad::apply(s, seat, &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn column(s: &State, rank: Rank) -> Vec<Color> {
    s.columns[rank.column().unwrap()].iter().copied().collect()
}

#[test]
fn trade_needs_warm_and_cool() {
    let mut s = table(units(&[Red, Orange, Green]), units(&[Sky]));
    assert!(!legal(&s).contains(&trade(card(Rank::Unit, Red), card(Rank::Unit, Orange))));
    let top = s.columns[0][0];
    play(
        &mut s,
        trade(card(Rank::Unit, Red), card(Rank::Unit, Green)),
    );
    assert_eq!(
        s.hands[0],
        vec![card(Rank::Unit, Orange), card(Rank::Bi, top)]
    );
    assert_eq!(s.discard, vec![Red, Green]);
}

#[test]
fn traded_cards_go_under_their_column() {
    let mut s = table(
        vec![card(Rank::Bi, Navy), card(Rank::Bi, Yellow)],
        units(&[Sky]),
    );
    s.columns[0] = VecDeque::from(vec![Red, Orange, Sky, Green]);
    let top_tri = s.columns[1][0];
    play(&mut s, trade(card(Rank::Bi, Yellow), card(Rank::Bi, Navy)));
    assert_eq!(
        column(&s, Rank::Bi),
        vec![Red, Orange, Sky, Green, Yellow, Navy]
    );
    assert_eq!(s.hands[0], vec![card(Rank::Tri, top_tri)]);
}

#[test]
fn empty_column_blocks_the_trade() {
    let mut s = table(
        vec![card(Rank::Bi, Red), card(Rank::Bi, Green)],
        units(&[Sky]),
    );
    s.columns[1].clear();
    assert!(!legal(&s).iter().any(|a| matches!(a, Action::Trade { .. })));
}

#[test]
fn bonus_is_offered_then_taken() {
    let mut s = table(
        vec![card(Rank::Quint, Red), card(Rank::Quint, Navy)],
        units(&[Sky]),
    );
    s.deck = vec![Orange, Green];
    let (quad, tri, bi) = (s.columns[2][0], s.columns[1][0], s.columns[0][0]);
    play(
        &mut s,
        trade(card(Rank::Quint, Red), card(Rank::Quint, Navy)),
    );
    assert_eq!(s.monads[0], 1);
    assert_eq!(s.monad_stock, 9);
    assert_eq!(s.turn.bonus_offer, Some(Rank::Quint));
    assert_eq!(legal(&s), vec![Action::TakeBonus, Action::DeclineBonus]);
    play(&mut s, Action::TakeBonus);
    let mut expected = vec![
        card(Rank::Unit, Green),
        card(Rank::Bi, bi),
        card(Rank::Tri, tri),
        card(Rank::Quad, quad),
    ];
    expected.sort();
    assert_eq!(s.hands[0], expected);
    assert!(s.turn.bonus_used);
    assert_eq!(s.turn.bonus_offer, None);
    assert_eq!(column(&s, Rank::Quint)[6..], [Red, Navy]);
}

#[test]
fn bonus_skips_empty_parts() {
    let mut s = table(
        vec![card(Rank::Quint, Red), card(Rank::Quint, Navy)],
        units(&[Sky]),
    );
    s.columns[1].clear();
    let (quad, bi) = (s.columns[2][0], s.columns[0][0]);
    play(
        &mut s,
        trade(card(Rank::Quint, Red), card(Rank::Quint, Navy)),
    );
    play(&mut s, Action::TakeBonus);
    assert_eq!(s.hands[0], vec![card(Rank::Bi, bi), card(Rank::Quad, quad)]);

    let mut s = table(
        vec![card(Rank::Tri, Red), card(Rank::Tri, Navy)],
        units(&[Sky]),
    );
    s.columns[0].clear();
    play(&mut s, trade(card(Rank::Tri, Red), card(Rank::Tri, Navy)));
    assert_eq!(s.turn.bonus_offer, None);
    assert!(legal(&s).contains(&Action::EndTurn));
}

#[test]
fn declined_bonus_can_come_later() {
    let mut s = table(
        vec![
            card(Rank::Bi, Yellow),
            card(Rank::Bi, Green),
            card(Rank::Tri, Orange),
            card(Rank::Tri, Sky),
        ],
        units(&[Sky]),
    );
    s.deck = vec![Red, Red];
    // The Tri column is set so that the traded-for Tri cannot pair again.
    s.columns[1] = VecDeque::from(vec![Red, Yellow]);
    play(&mut s, trade(card(Rank::Bi, Yellow), card(Rank::Bi, Green)));
    assert_eq!(s.turn.bonus_offer, Some(Rank::Bi));
    play(&mut s, Action::DeclineBonus);
    play(&mut s, trade(card(Rank::Tri, Orange), card(Rank::Tri, Sky)));
    assert_eq!(s.turn.bonus_offer, Some(Rank::Tri));
    play(&mut s, Action::TakeBonus);
    // A second bonus trade in the same turn opens nothing.
    s.hands[0] = vec![card(Rank::Bi, Red), card(Rank::Bi, Navy)];
    s.columns[1] = VecDeque::from(vec![Green]);
    play(&mut s, trade(card(Rank::Bi, Red), card(Rank::Bi, Navy)));
    assert_eq!(s.turn.bonus_offer, None);
    play(&mut s, Action::EndTurn);
    assert!(!s.turn.bonus_used);
}

#[test]
fn no_bonus_for_units() {
    let mut s = table(units(&[Red, Navy]), units(&[Sky]));
    s.deck = vec![Green];
    play(&mut s, trade(card(Rank::Unit, Red), card(Rank::Unit, Navy)));
    assert_eq!(s.turn.bonus_offer, None);
}

#[test]
fn joker_trades_follow_the_other_card() {
    // Seat 0's Identity is yellow.
    let s = table(
        vec![
            card(Rank::Bi, Yellow),
            card(Rank::Quint, Navy),
            card(Rank::Quint, Green),
            card(Rank::Bi, Red),
        ],
        units(&[Sky]),
    );
    let actions = legal(&s);
    let joker = trade(card(Rank::Bi, Yellow), card(Rank::Quint, Navy));
    assert!(actions.contains(&joker));
    assert!(!actions.contains(&trade(card(Rank::Bi, Red), card(Rank::Quint, Navy))));
    let mut after = s.clone();
    play(&mut after, joker);
    assert_eq!(after.monads[0], 1);
    assert_eq!(after.turn.bonus_offer, None);
    assert_eq!(column(&after, Rank::Bi).last(), Some(&Yellow));
    assert_eq!(column(&after, Rank::Quint).last(), Some(&Navy));

    // Same rank with the Identity colour: an ordinary trade, bonus included.
    let mut s = table(
        vec![card(Rank::Quint, Yellow), card(Rank::Quint, Green)],
        units(&[Sky]),
    );
    play(
        &mut s,
        trade(card(Rank::Quint, Yellow), card(Rank::Quint, Green)),
    );
    assert_eq!(s.turn.bonus_offer, Some(Rank::Quint));

    // A Joker trading Units goes for a Bi.
    let mut s = table(
        vec![card(Rank::Quint, Yellow), card(Rank::Unit, Sky)],
        units(&[Sky]),
    );
    play(
        &mut s,
        trade(card(Rank::Quint, Yellow), card(Rank::Unit, Sky)),
    );
    assert_eq!(s.hands[0].len(), 1);
    assert_eq!(s.hands[0][0].rank, Rank::Bi);
    assert_eq!(s.discard, vec![Sky]);
}

#[test]
fn buy_takes_minimal_payments() {
    let s = table(
        vec![
            card(Rank::Bi, Red),
            card(Rank::Bi, Orange),
            card(Rank::Unit, Navy),
        ],
        units(&[Sky]),
    );
    let actions = legal(&s);
    let tri = Target::Card(Rank::Tri);
    assert!(actions.contains(&buy(
        tri,
        vec![
            card(Rank::Bi, Red),
            card(Rank::Bi, Orange),
            card(Rank::Unit, Navy)
        ]
    )));
    let buys: Vec<&Action> = actions
        .iter()
        .filter(|a| matches!(a, Action::Buy { .. }))
        .collect();
    let unique: HashSet<&&Action> = buys.iter().collect();
    assert_eq!(unique.len(), buys.len());
    for a in &buys {
        let Action::Buy { target, cards } = a else {
            unreachable!()
        };
        let total: u32 = cards.iter().map(|c| c.rank.value()).sum();
        let least = cards.iter().map(|c| c.rank.value()).min().unwrap();
        assert!(cards.len() >= 2);
        assert!(
            total >= target.value() && total - least < target.value(),
            "{a:?}"
        );
        assert!(cards.iter().all(|c| target.above(c.rank)));
    }
    // [M] example: two warm Bi and a Unit buy the top Tri.
    let mut s = s;
    play(
        &mut s,
        buy(
            tri,
            vec![
                card(Rank::Bi, Red),
                card(Rank::Bi, Orange),
                card(Rank::Unit, Navy),
            ],
        ),
    );
    assert_eq!(s.hands[0].len(), 1);
    assert_eq!(s.hands[0][0].rank, Rank::Tri);
    assert_eq!(s.discard, vec![Navy]);
    assert_eq!(column(&s, Rank::Bi)[6..], [Red, Orange]);
}

#[test]
fn buy_a_monad_with_eighty() {
    let s = table(
        vec![
            card(Rank::Quint, Red),
            card(Rank::Quint, Orange),
            card(Rank::Quad, Red),
        ],
        units(&[Sky]),
    );
    assert!(legal(&s).contains(&buy(
        Target::Monad,
        vec![
            card(Rank::Quint, Red),
            card(Rank::Quint, Orange),
            card(Rank::Quad, Red)
        ]
    )));
    let s = table(
        vec![
            card(Rank::Quint, Red),
            card(Rank::Quint, Orange),
            card(Rank::Tri, Red),
        ],
        units(&[Sky]),
    );
    assert!(!legal(&s).iter().any(|a| matches!(
        a,
        Action::Buy {
            target: Target::Monad,
            ..
        }
    )));
}

#[test]
fn leap_needs_a_prior_trade_or_buy() {
    let mut s = table(
        units(&[Red, Orange, Yellow, Navy, Sky, Green, Red, Green]),
        units(&[Sky]),
    );
    assert!(!legal(&s).iter().any(|a| matches!(a, Action::Leap { .. })));
    play(
        &mut s,
        trade(card(Rank::Unit, Red), card(Rank::Unit, Green)),
    );
    let leaps: Vec<Action> = legal(&s)
        .into_iter()
        .filter(|a| matches!(a, Action::Leap { .. }))
        .collect();
    // C(6,4) + C(6,5) + C(6,6) colour sets.
    assert_eq!(leaps.len(), 15 + 6 + 1);
    let quint = s.columns[3][0];
    play(
        &mut s,
        Action::Leap {
            colors: Color::ALL.to_vec(),
        },
    );
    assert!(s.hands[0].contains(&card(Rank::Quint, quint)));
    assert_eq!(
        s.hands[0].iter().filter(|c| c.rank == Rank::Unit).count(),
        0
    );
    // Colours in double never leap.
    let mut s = table(units(&[Red, Red, Orange, Yellow, Navy]), units(&[Sky]));
    play(&mut s, trade(card(Rank::Unit, Red), card(Rank::Unit, Navy)));
    assert!(!legal(&s).iter().any(|a| matches!(a, Action::Leap { .. })));
}

#[test]
fn pass_only_when_the_rule_allows() {
    // [M] example: a Bi and four Units, no Bi on the table but a Tri,
    // neither deck nor discard.
    let mut s = table(
        vec![
            card(Rank::Bi, Red),
            card(Rank::Unit, Orange),
            card(Rank::Unit, Orange),
            card(Rank::Unit, Yellow),
            card(Rank::Unit, Yellow),
        ],
        units(&[Sky]),
    );
    s.columns[0].clear();
    assert!(legal(&s).contains(&Action::Pass));
    // With two Bi and four Units, buying the Tri is forced.
    s.hands[0].push(card(Rank::Bi, Orange));
    s.hands[0].sort();
    let actions = legal(&s);
    assert!(!actions.contains(&Action::Pass));
    assert!(actions.iter().any(|a| matches!(
        a,
        Action::Buy {
            target: Target::Card(Rank::Tri),
            ..
        }
    )));
}

#[test]
fn reaching_the_target_ends_at_once() {
    let mut s = state(4);
    s.current = Seat(2);
    s.hands[2] = vec![card(Rank::Quint, Red), card(Rank::Quint, Green)];
    s.monads = vec![1, 0, 2, 0];
    s.monad_stock = 7;
    play(
        &mut s,
        trade(card(Rank::Quint, Red), card(Rank::Quint, Green)),
    );
    assert_eq!(s.winner, Some(Seat(2)));
    assert!(Monad::pending(&s).is_empty());
    assert_eq!(Monad::outcome(&s), Some(Outcome::Ranked(vec![1, 2, 0, 2])));
}

#[test]
fn a_full_round_of_passes_stalls() {
    let mut s = table(vec![card(Rank::Tri, Red)], vec![card(Rank::Tri, Orange)]);
    play(&mut s, Action::Pass);
    assert!(!s.stalled);
    play(&mut s, Action::Pass);
    assert!(s.stalled);
    assert_eq!(Monad::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn forty_rounds_without_a_monad_stall() {
    let mut s = table(units(&[Red]), units(&[Sky]));
    s.deck = vec![Green];
    s.turns_since_monad = 40 * 2 - 1;
    play(&mut s, Action::Draw);
    assert!(s.stalled);
    assert!(Monad::pending(&s).is_empty());
}

#[test]
fn forged_actions_are_refused_unchanged() {
    let s = table(
        vec![
            card(Rank::Bi, Red),
            card(Rank::Bi, Orange),
            card(Rank::Unit, Navy),
            card(Rank::Unit, Sky),
        ],
        units(&[Sky]),
    );
    let refused = |s: &State, seat: Seat, action: Action| {
        let mut copy = s.clone();
        assert!(
            Monad::apply(&mut copy, seat, &action).is_err(),
            "{action:?}"
        );
        assert_eq!(snapshot(&copy), snapshot(s));
    };
    refused(
        &s,
        Seat(0),
        trade(card(Rank::Unit, Red), card(Rank::Unit, Green)),
    );
    refused(
        &s,
        Seat(0),
        buy(
            Target::Card(Rank::Tri),
            vec![
                card(Rank::Bi, Red),
                card(Rank::Bi, Orange),
                card(Rank::Unit, Navy),
                card(Rank::Unit, Sky),
            ],
        ),
    );
    refused(
        &s,
        Seat(0),
        Action::Leap {
            colors: vec![Red, Orange, Navy, Sky],
        },
    );
    refused(&s, Seat(0), Action::EndTurn);
    refused(&s, Seat(1), Action::Pass);
    let mut offer = table(
        vec![
            card(Rank::Bi, Red),
            card(Rank::Bi, Navy),
            card(Rank::Unit, Red),
            card(Rank::Unit, Sky),
        ],
        units(&[Sky]),
    );
    offer.deck = vec![Green];
    play(&mut offer, trade(card(Rank::Bi, Red), card(Rank::Bi, Navy)));
    assert!(offer.turn.bonus_offer.is_some());
    refused(
        &offer,
        Seat(0),
        trade(card(Rank::Unit, Red), card(Rank::Unit, Sky)),
    );
    refused(&offer, Seat(0), Action::EndTurn);
    let mut over = s.clone();
    over.winner = Some(Seat(1));
    refused(&over, Seat(0), Action::EndTurn);
}

#[test]
fn timeout_declines_then_ends_the_turn() {
    let mut s = table(
        vec![card(Rank::Bi, Red), card(Rank::Bi, Navy)],
        units(&[Sky]),
    );
    assert_eq!(
        Monad::timeout_action(&s, Seat(0)),
        Some(trade(card(Rank::Bi, Red), card(Rank::Bi, Navy)))
    );
    s.deck = vec![Green];
    assert_eq!(Monad::timeout_action(&s, Seat(0)), Some(Action::Draw));
    play(&mut s, trade(card(Rank::Bi, Red), card(Rank::Bi, Navy)));
    assert_eq!(
        Monad::timeout_action(&s, Seat(0)),
        Some(Action::DeclineBonus)
    );
    play(&mut s, Action::DeclineBonus);
    assert_eq!(Monad::timeout_action(&s, Seat(0)), Some(Action::EndTurn));
}

#[test]
fn drawn_units_are_private() {
    let mut s = table(units(&[Red]), units(&[Sky]));
    s.deck = vec![Green];
    play(&mut s, Action::Draw);
    let mine = serde_json::to_value(Monad::view(&s, Viewer::Seat(Seat(0)))).unwrap();
    let theirs = serde_json::to_value(Monad::view(&s, Viewer::Seat(Seat(1)))).unwrap();
    assert_eq!(mine["lastTurn"][0]["drawn"], json!("Green"));
    assert_eq!(theirs["lastTurn"][0]["drawn"], json!(null));
    assert_eq!(theirs["lastTurn"][0]["drew"], json!(true));
}

#[test]
fn determinize_keeps_what_the_observer_sees() {
    use ludo_core::Determinize;
    use rand::SeedableRng;

    let mut s = state(3);
    // Play a few turns so that hands hold several ranks and the deck exists.
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(5);
    for _ in 0..60 {
        if Monad::pending(&s).is_empty() {
            break;
        }
        let seat = s.current;
        let actions = Monad::legal_actions(&s, seat);
        let pick =
            actions[rand::Rng::gen_range(&mut rng, 0..actions.len() as u32) as usize].clone();
        Monad::apply(&mut s, seat, &pick).unwrap();
    }
    let all = |s: &State| {
        let mut cards: Vec<Card> = s.hands.iter().flatten().copied().collect();
        cards.extend(s.deck.iter().map(|&c| card(Rank::Unit, c)));
        cards.sort();
        cards
    };
    for observer in (0..3).map(Seat) {
        let seen = serde_json::to_value(Monad::view(&s, Viewer::Seat(observer))).unwrap();
        for _ in 0..20 {
            let d = Monad::determinize(&s, observer, &mut rng);
            assert_eq!(
                serde_json::to_value(Monad::view(&d, Viewer::Seat(observer))).unwrap(),
                seen
            );
            assert_eq!(all(&d), all(&s));
            assert_eq!(d.hands[observer.index()], s.hands[observer.index()]);
        }
    }
}
