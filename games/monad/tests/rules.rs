use std::collections::{HashSet, VecDeque};

use ludo_core::{Game, Seat, Viewer};
use monad::{Action, Card, Color, Config, Monad, Rank, State};
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
