use std::collections::HashSet;

use ludo_core::{Game, Outcome, Seat, Viewer};
use pennywise::{Action, Character, Config, Pennywise, Purse, State};
use serde_json::{json, Value};

fn purse(coins: &[u8]) -> Purse {
    let mut p = Purse::default();
    for &c in coins {
        p.add(c);
    }
    p
}

/// A table where seat i holds `purses[i]`, with `pot` in the middle.
fn state(purses: &[&[u8]], pot: &[u8], current: u8) -> State {
    let players = u8::try_from(purses.len()).unwrap();
    let mut s = Pennywise::new(&Config::default(), players, 0);
    s.purses = purses.iter().map(|p| purse(p)).collect();
    s.pot = purse(pot);
    s.current = Seat(current);
    s
}

fn play(coin: u8, take: &[u8]) -> Action {
    Action::Play {
        coin,
        take: take.to_vec(),
    }
}

fn ranks(s: &State) -> Option<Vec<u8>> {
    Pennywise::outcome(s).map(|Outcome::Ranked(r)| r)
}

#[test]
fn every_seat_starts_with_the_character_purse() {
    for (character, total) in [
        (Character::Cody, 64),
        (Character::Darlene, 52),
        (Character::Sugar, 26),
        (Character::Taylor, 23),
    ] {
        let s = Pennywise::new(&Config { character }, 3, 1);
        assert_eq!(s.purses.len(), 3);
        for p in &s.purses {
            assert_eq!(p.total(), total, "{character:?}");
            assert_eq!(*p, character.purse());
        }
        assert!(s.pot.is_empty());
        assert_eq!(s.last, None);
        assert_eq!(s.ruined, None);
    }
    let cody = Character::Cody.purse();
    assert_eq!(
        [1, 5, 10, 25].map(|c| cody.count(c)),
        [4, 3, 2, 1],
        "Cody est la main d'exemple de la fiche"
    );
}

#[test]
fn first_player_comes_from_the_seed() {
    let cfg = Config::default();
    assert_eq!(
        Pennywise::new(&cfg, 4, 42).current,
        Pennywise::new(&cfg, 4, 42).current
    );
    let firsts: HashSet<Seat> = (0..200)
        .map(|seed| Pennywise::new(&cfg, 4, seed).current)
        .collect();
    assert_eq!(firsts.len(), 4);
}

#[test]
fn ten_takes_back_nine_at_most() {
    let mut s = state(&[&[10], &[1]], &[5, 3, 1, 1], 0);
    assert_eq!(
        Pennywise::legal_actions(&s, Seat(0)),
        vec![play(10, &[5, 3, 1])]
    );
    Pennywise::apply(&mut s, Seat(0), &play(10, &[5, 3, 1])).unwrap();
    assert_eq!(s.pot, purse(&[10, 1]));
    assert_eq!(s.purses[0], purse(&[5, 3, 1]));
}

#[test]
fn equal_maximal_takes_are_offered_as_choices() {
    let s = state(&[&[5], &[1]], &[3, 2, 2, 1], 0);
    let legal: HashSet<Action> = Pennywise::legal_actions(&s, Seat(0)).into_iter().collect();
    assert_eq!(legal, HashSet::from([play(5, &[3, 1]), play(5, &[2, 2])]));
}

#[test]
fn nothing_to_take_is_a_legal_empty_take() {
    let s = state(&[&[10], &[1]], &[], 0);
    assert_eq!(Pennywise::legal_actions(&s, Seat(0)), vec![play(10, &[])]);
    let s = state(&[&[5], &[1]], &[10, 5], 0);
    assert_eq!(Pennywise::legal_actions(&s, Seat(0)), vec![play(5, &[])]);
    let s = state(&[&[1], &[1]], &[3, 2], 0);
    assert_eq!(Pennywise::legal_actions(&s, Seat(0)), vec![play(1, &[])]);
}

#[test]
fn turn_passes_to_the_left() {
    let mut s = state(&[&[1, 1], &[1, 1], &[1, 1]], &[], 2);
    Pennywise::apply(&mut s, Seat(2), &play(1, &[])).unwrap();
    assert_eq!(s.current, Seat(0));
    assert_eq!(Pennywise::pending(&s).0, vec![Seat(0)]);
}

#[test]
fn last_coin_with_change_back_does_not_ruin() {
    let mut s = state(&[&[10], &[1]], &[5], 0);
    Pennywise::apply(&mut s, Seat(0), &play(10, &[5])).unwrap();
    assert_eq!(s.ruined, None);
    assert!(!Pennywise::pending(&s).is_empty());
}

#[test]
fn playing_the_last_coin_ruins_and_ends() {
    let mut s = state(&[&[1], &[5]], &[10], 1);
    Pennywise::apply(&mut s, Seat(1), &play(5, &[])).unwrap();
    assert_eq!(s.ruined, Some(Seat(1)));
    assert!(Pennywise::pending(&s).is_empty());
    assert!(Pennywise::outcome(&s).is_some());
}

#[test]
fn two_players_ruined_one_loses() {
    let mut s = state(&[&[5], &[10, 3]], &[], 0);
    Pennywise::apply(&mut s, Seat(0), &play(5, &[])).unwrap();
    assert_eq!(ranks(&s), Some(vec![1, 0]));
    let v = serde_json::to_value(Pennywise::view(&s, Viewer::Seat(Seat(1)))).unwrap();
    assert_eq!(v["totals"], json!([0, 13]));
    assert_eq!(v["ranks"], json!([1, 0]));
    assert_eq!(v["current"], Value::Null);
}

#[test]
fn survivors_with_equal_money_tie() {
    let mut s = state(&[&[1], &[10, 10], &[20]], &[], 0);
    Pennywise::apply(&mut s, Seat(0), &play(1, &[])).unwrap();
    assert_eq!(ranks(&s), Some(vec![2, 0, 0]));
}

#[test]
fn forged_moves_are_refused_without_change() {
    let s = state(&[&[10, 1], &[1]], &[5, 3, 1], 0);
    let forged = [
        (Seat(0), play(25, &[])),
        (Seat(0), play(10, &[3])),
        (Seat(0), play(10, &[1, 3, 5])),
        (Seat(1), play(1, &[])),
    ];
    for (seat, action) in forged {
        let mut t = s.clone();
        assert!(
            Pennywise::apply(&mut t, seat, &action).is_err(),
            "{action:?}"
        );
        assert_eq!(t, s);
    }
    let mut over = state(&[&[1], &[5]], &[], 0);
    Pennywise::apply(&mut over, Seat(0), &play(1, &[])).unwrap();
    let before = over.clone();
    assert!(Pennywise::apply(&mut over, Seat(1), &play(5, &[1])).is_err());
    assert_eq!(over, before);
    assert!(Pennywise::legal_actions(&over, Seat(1)).is_empty());
}

#[test]
fn takes_are_distinct_and_all_maximal() {
    let all = [1, 2, 3, 5, 10, 20, 25];
    let pot = [1, 1, 1, 1, 2, 2, 3, 3, 3, 5, 5, 5, 10, 10, 20];
    let s = state(&[&all, &all, &all, &all, &all, &all], &pot, 0);
    let legal = Pennywise::legal_actions(&s, Seat(0));
    let distinct: HashSet<&Action> = legal.iter().collect();
    assert_eq!(distinct.len(), legal.len());
    for coin in all {
        let sums: HashSet<u32> = legal
            .iter()
            .filter(|Action::Play { coin: c, .. }| *c == coin)
            .map(|Action::Play { take, .. }| take.iter().map(|&t| u32::from(t)).sum())
            .collect();
        assert_eq!(sums.len(), 1, "pièce {coin} : {sums:?}");
        assert!(sums.into_iter().all(|sum| sum < u32::from(coin)));
    }
    for Action::Play { take, .. } in &legal {
        assert!(take.windows(2).all(|w| w[0] >= w[1]), "{take:?}");
    }
}

#[test]
fn config_reads_the_lobby_json() {
    let darlene: Config = serde_json::from_value(json!({ "character": "Darlene" })).unwrap();
    assert_eq!(darlene.character, Character::Darlene);
    let empty: Config = serde_json::from_value(json!({})).unwrap();
    assert_eq!(empty.character, Character::Cody);
    assert_eq!(Config::default().character, Character::Cody);
}

#[test]
fn the_view_shows_everything() {
    let s = state(&[&[10, 1], &[5, 5]], &[3, 2], 1);
    let seat = serde_json::to_value(Pennywise::view(&s, Viewer::Seat(Seat(1)))).unwrap();
    let spectator = serde_json::to_value(Pennywise::view(&s, Viewer::Spectator)).unwrap();
    assert_eq!(seat, spectator);
    assert_eq!(seat["totals"], json!([11, 10]));
    assert_eq!(seat["potTotal"], json!(5));
    assert_eq!(
        seat["pot"],
        json!([{ "cents": 2, "count": 1 }, { "cents": 3, "count": 1 }])
    );
    assert_eq!(seat["purses"][1], json!([{ "cents": 5, "count": 2 }]));
    assert_eq!(seat["current"], json!(1));
    assert_eq!(seat["character"], json!("Cody"));
    assert_eq!(seat["ranks"], Value::Null);
}
