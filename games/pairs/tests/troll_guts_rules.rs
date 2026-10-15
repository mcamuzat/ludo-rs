use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::testing::{r, ranks};
use pairs::troll_guts::testing::table;
use pairs::troll_guts::{Action, Config, Event, Phase, Source, State, TrollGuts};

fn act(s: &mut State, seat: u8, action: Action) {
    TrollGuts::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot)
}

fn cards(s: &State) -> usize {
    s.stacks
        .iter()
        .chain(&s.faceup)
        .map(Vec::len)
        .sum::<usize>()
        + s.supply.deck_len()
        + s.supply.discard_len()
}

#[test]
fn a_hand_antes_deals_and_waits_for_the_stacks() {
    for players in 3..=6u8 {
        let s = TrollGuts::new(&Config::default(), players, 2);
        assert_eq!(s.phase, Phase::Arrange);
        assert!(s.stacks.iter().all(|st| st.len() == 5));
        assert!(s.faceup.iter().all(|f| !f.is_empty()));
        assert_eq!(s.pot, u32::from(players));
        assert_eq!(cards(&s), DECK_SIZE);
        assert_eq!(TrollGuts::pending(&s).0.len(), usize::from(players));
        let low = s.faceup.iter().map(|f| f[0]).min().unwrap();
        assert_eq!(s.faceup[s.first.index()][0], low);
    }
}

#[test]
fn stacks_are_ordered_in_secret_then_play_starts_with_the_low_card() {
    let mut s = TrollGuts::new(&Config::default(), 3, 4);
    let mine = s.stacks[0].clone();
    let mut order = mine.clone();
    order.sort_unstable();
    order.reverse();
    // An order that is not a permutation of the stack is refused.
    let stranger = (1..=10u8).map(r).find(|c| !mine.contains(c)).unwrap();
    let mut bad = order.clone();
    bad[0] = stranger;
    let copy = s.clone();
    assert!(TrollGuts::apply(&mut s, Seat(0), &Action::Arrange { order: bad }).is_err());
    assert_eq!(s, copy);
    act(
        &mut s,
        0,
        Action::Arrange {
            order: order.clone(),
        },
    );
    assert_eq!(
        TrollGuts::visible_pending(&s, Viewer::Seat(Seat(0))),
        Pending::none()
    );
    assert_eq!(
        TrollGuts::view(&s, Viewer::Seat(Seat(0))).placed,
        Some(order.clone())
    );
    assert_eq!(TrollGuts::view(&s, Viewer::Seat(Seat(1))).placed, None);
    for seat in 1..3u8 {
        let order = s.stacks[usize::from(seat)].clone();
        act(&mut s, seat, Action::Arrange { order });
    }
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.turn, Some(s.first));
    // My stack shows top first, in my order.
    assert_eq!(TrollGuts::view(&s, Viewer::Seat(Seat(0))).stack, order);
}

#[test]
fn draw_steal_and_buy_each_give_one_face_up_card() {
    let mut s = table(&[&[2, 9], &[3, 6], &[4]], &[&[10], &[1], &[5]], &[7, 8]);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.faceup[0], ranks(&[10, 8]));
    act(&mut s, 1, Action::Steal { from: Seat(0) });
    assert!(s.events.contains(&Event::Took {
        seat: Seat(1),
        card: r(2),
        from: Source::Stack { seat: Seat(0) },
    }));
    assert_eq!(s.stacks[0], ranks(&[9]));
    act(&mut s, 2, Action::Buy);
    assert_eq!(s.faceup[2], ranks(&[5, 4]));
    assert_eq!(s.chips[2], -2);
    assert_eq!(s.pot, 4);
    assert_eq!(s.turn, Some(Seat(0)));
    assert_eq!(balance(&s), 0);
}

#[test]
fn a_pair_costs_its_value_and_the_survivor_takes_the_pot() {
    let mut s = table(&[&[2], &[3], &[4]], &[&[10], &[6], &[5]], &[1, 6, 10]);
    act(&mut s, 0, Action::Draw); // 10: pair
    assert!(s.events.contains(&Event::Busted {
        seat: Seat(0),
        card: r(10),
        paid: 10
    }));
    assert!(s.out[0]);
    assert_eq!(s.turn, Some(Seat(1)));
    // A knocked-out player's stack cannot be stolen.
    assert!(!TrollGuts::legal_actions(&s, Seat(1)).contains(&Action::Steal { from: Seat(0) }));
    act(&mut s, 1, Action::Draw); // 6: pair
                                  // Seat 2 alone survives: it takes 3 antes + 10 + 6.
    assert!(s.events.contains(&Event::PotWon {
        seats: vec![Seat(2)],
        share: 19
    }));
    assert_eq!(s.hand_no, 2);
    assert_eq!(balance(&s), 0);
}

#[test]
fn the_game_ends_after_its_hands() {
    let mut s = table(&[&[2], &[3], &[4]], &[&[10], &[6], &[5]], &[6, 10]);
    s.hands_total = 1;
    act(&mut s, 0, Action::Draw);
    act(&mut s, 1, Action::Draw);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(TrollGuts::pending(&s), Pending::none());
    assert_eq!(TrollGuts::outcome(&s), Some(Outcome::Ranked(vec![2, 1, 0])));
}
