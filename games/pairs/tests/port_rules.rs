use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::port::testing::table;
use pairs::port::{Action, Config, Event, Phase, Port, State, Status};
use pairs::testing::{r, ranks};

fn act(s: &mut State, seat: u8, action: Action) {
    Port::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn cards(s: &State) -> usize {
    s.stacks.iter().map(Vec::len).sum::<usize>() + s.supply.deck_len() + s.supply.discard_len()
}

fn scored(s: &State) -> Option<(Vec<Option<u32>>, Vec<u32>)> {
    s.events.iter().find_map(|e| match e {
        Event::RoundScored { totals, points, .. } => Some((totals.clone(), points.clone())),
        _ => None,
    })
}

#[test]
fn a_round_opens_with_one_card_each_and_the_lowest_first() {
    for players in 2..=7u8 {
        let s = Port::new(&Config::default(), players, 5);
        assert!(s.stacks.iter().all(|st| !st.is_empty()));
        assert!(s.supply.discard_len() >= 5);
        assert_eq!(cards(&s), DECK_SIZE);
        let first = s.turn.unwrap();
        let low = s.stacks.iter().map(|st| st[0]).min().unwrap();
        assert_eq!(s.stacks[first.index()][0], low);
        assert_eq!(Port::pending(&s), Pending::one(first));
    }
}

#[test]
fn a_hit_that_pairs_busts_and_the_turn_moves_on() {
    let mut s = table(&[&[4, 7], &[2], &[5]], &[3, 7]);
    act(&mut s, 0, Action::Hit);
    assert_eq!(s.status[0], Status::Busted);
    assert!(s.events.contains(&Event::Busted {
        seat: Seat(0),
        card: r(7)
    }));
    assert_eq!(s.turn, Some(Seat(1)));
    act(&mut s, 1, Action::Hit);
    assert_eq!(s.stacks[1], ranks(&[2, 3]));
    assert_eq!(s.turn, Some(Seat(2)));
    // A busted player never plays again this round.
    act(&mut s, 2, Action::Lock);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn ranks_score_from_n_down_and_ties_take_the_higher() {
    // [R] 23, 22, 22, 17 and a bust, five players: 5, 4, 4, 2, 0.
    let totals = [Some(23), Some(22), Some(22), Some(17), None];
    assert_eq!(State::round_points(&totals, 5), vec![5, 4, 4, 2, 0]);
}

#[test]
fn the_round_ends_when_everyone_has_locked_or_busted() {
    let mut s = table(&[&[9, 8], &[10], &[2, 3]], &[1, 1, 10]);
    act(&mut s, 0, Action::Lock);
    act(&mut s, 1, Action::Hit); // 10 again: bust
    act(&mut s, 2, Action::Lock);
    let (totals, points) = scored(&s).expect("scored");
    assert_eq!(totals, vec![Some(17), None, Some(5)]);
    assert_eq!(points, vec![3, 0, 2]);
    assert_eq!(s.scores, vec![3, 0, 2]);
    assert_eq!(s.round_no, 2);
    assert!(s.status.iter().all(|&st| st == Status::Playing));
}

#[test]
fn the_first_alone_past_the_target_wins() {
    let mut s = table(&[&[9], &[2]], &[5]);
    s.scores = vec![19, 15];
    act(&mut s, 0, Action::Lock);
    act(&mut s, 1, Action::Lock);
    assert_eq!(s.scores, vec![21, 16]);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Port::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
    assert!(Port::legal_actions(&s, Seat(0)).is_empty());
}

#[test]
fn leaders_tied_past_the_target_play_on_alone() {
    let mut s = table(&[&[9], &[2], &[8]], &[5, 6, 7, 1, 3, 4]);
    s.scores = vec![18, 19, 10];
    act(&mut s, 0, Action::Lock);
    act(&mut s, 1, Action::Lock);
    act(&mut s, 2, Action::Lock);
    // 9 → 3 points (21), 8 → 2 (12), 2 → 1 (20): seat 0 alone on top.
    assert_eq!(s.scores, vec![21, 20, 12]);
    assert_eq!(s.phase, Phase::Over);
    let mut s = table(&[&[9], &[10], &[8]], &[5, 6, 7, 1, 3, 4]);
    s.scores = vec![20, 19, 10];
    for seat in 0..3 {
        act(&mut s, seat, Action::Lock);
    }
    // 9 → 2, 10 → 3: 22 and 22. Only seats 0 and 1 play round 2.
    assert_eq!(s.scores, vec![22, 22, 11]);
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.status[2], Status::Out);
    assert!(s.stacks[2].is_empty());
    assert!(!Port::pending(&s).contains(Seat(2)));
}

#[test]
fn everyone_sees_the_same_table() {
    let s = Port::new(&Config::default(), 4, 2);
    let v = Port::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(v, Port::view(&s, Viewer::Seat(Seat(3))));
    assert_eq!(v, Port::view(&s, Viewer::Spectator));
}
