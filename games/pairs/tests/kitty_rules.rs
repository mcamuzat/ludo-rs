use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::kitty::testing::table;
use pairs::kitty::{hand_size, Action, Config, Event, Kitty, Phase, State};
use pairs::testing::{r, ranks};

fn act(s: &mut State, seat: u8, action: Action) {
    Kitty::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn passes(s: &mut State, cards: &[u8]) {
    for (seat, &card) in cards.iter().enumerate() {
        act(s, seat as u8, Action::Pass { card: r(card) });
    }
}

fn bids(s: &mut State, cards: &[u8]) {
    for (seat, &card) in cards.iter().enumerate() {
        act(s, seat as u8, Action::Bid { card: r(card) });
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        Kitty::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
}

fn cards(s: &State) -> usize {
    s.hands.iter().map(Vec::len).sum::<usize>()
        + s.kitty.len()
        + s.removed.len()
        + s.target.len()
        + s.discard.len()
        + s.bids.len()
}

fn sorted(mut cards: Vec<pairs::deck::Rank>) -> Vec<pairs::deck::Rank> {
    cards.sort_unstable();
    cards
}

#[test]
fn the_deal_depends_on_the_table_size() {
    for players in 3..=6u8 {
        let s = Kitty::new(&Config::default(), players, 7);
        let size = hand_size(players);
        assert_eq!(size, [16, 12, 9, 8][usize::from(players - 3)]);
        assert!(s.hands.iter().all(|h| h.len() == size));
        assert_eq!(s.soft, players != 4);
        assert_eq!(s.removed.len(), if players == 5 { 3 } else { 0 });
        // The first target is a single card, from the kitty of 7.
        assert_eq!(s.target.len(), 1);
        assert_eq!(s.kitty.len(), 6);
        assert_eq!(s.phase, Phase::Pass);
        assert_eq!(cards(&s), DECK_SIZE);
        assert_eq!(Kitty::pending(&s).0.len(), usize::from(players));
    }
}

#[test]
fn passes_are_secret_and_go_left_together() {
    let mut s = table(&[&[1, 5], &[2, 6], &[3, 7], &[4, 8]], 1, &[9], &[10]);
    act(&mut s, 0, Action::Pass { card: r(5) });
    // Chosen, not gone; nobody else learns that seat 0 has chosen.
    assert_eq!(s.hands[0], ranks(&[1, 5]));
    refused(&s, 0, Action::Pass { card: r(1) });
    assert_eq!(
        Kitty::visible_pending(&s, Viewer::Seat(Seat(1))),
        Pending::one(Seat(1))
    );
    assert_eq!(
        Kitty::visible_pending(&s, Viewer::Seat(Seat(0))),
        Pending::none()
    );
    assert_eq!(Kitty::view(&s, Viewer::Seat(Seat(0))).placed, Some(r(5)));
    assert_eq!(Kitty::view(&s, Viewer::Seat(Seat(1))).placed, None);
    act(&mut s, 1, Action::Pass { card: r(6) });
    act(&mut s, 2, Action::Pass { card: r(3) });
    act(&mut s, 3, Action::Pass { card: r(8) });
    assert_eq!(s.phase, Phase::Bid);
    assert_eq!(sorted(s.hands[0].clone()), ranks(&[1, 8]));
    assert_eq!(sorted(s.hands[1].clone()), ranks(&[2, 5]));
    assert_eq!(sorted(s.hands[2].clone()), ranks(&[6, 7]));
    assert_eq!(sorted(s.hands[3].clone()), ranks(&[3, 4]));
    let v = Kitty::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!((v.passed, v.received), (Some(r(6)), Some(r(5))));
}

#[test]
fn the_lowest_unique_bid_takes_the_target() {
    let mut s = table(&[&[2, 9], &[2, 9], &[5, 9], &[7, 9]], 1, &[10], &[6]);
    passes(&mut s, &[9, 9, 9, 9]);
    // The two 2s tie and cannot win: the 5 takes the 10.
    bids(&mut s, &[2, 2, 5, 7]);
    assert!(s.hands[2].contains(&r(10)));
    assert!(s.events.contains(&Event::BidsRevealed {
        bids: ranks(&[2, 2, 5, 7]),
        winners: vec![Seat(2)],
    }));
    assert!(s.events.contains(&Event::Took {
        seat: Seat(2),
        card: r(10)
    }));
    assert_eq!(sorted(s.discard.clone()), ranks(&[2, 2, 5, 7]));
    // Round 2: the 6 is the target.
    assert_eq!((s.round_no, s.target.clone()), (2, ranks(&[6])));
    assert_eq!(s.phase, Phase::Pass);
}

#[test]
fn without_a_unique_bid_the_target_is_discarded() {
    let mut s = table(&[&[3, 9], &[3, 9], &[6, 9], &[6, 9]], 1, &[10], &[8]);
    passes(&mut s, &[9, 9, 9, 9]);
    bids(&mut s, &[3, 3, 6, 6]);
    assert!(s.events.contains(&Event::Discarded {
        cards: ranks(&[10])
    }));
    assert!(s.discard.contains(&r(10)));
    assert!(s.hands.iter().all(|h| h == &ranks(&[9])));
}

#[test]
fn the_1_wins_but_gives_the_target_away() {
    let mut s = table(&[&[1, 9], &[2, 9], &[5, 9], &[7, 9]], 1, &[10], &[8]);
    passes(&mut s, &[9, 9, 9, 9]);
    bids(&mut s, &[1, 2, 5, 7]);
    assert_eq!(s.phase, Phase::Claim);
    assert_eq!(Kitty::pending(&s), Pending::one(Seat(0)));
    refused(&s, 0, Action::Take { card: r(10) });
    refused(
        &s,
        0,
        Action::Give {
            card: r(10),
            to: Seat(0),
        },
    );
    assert_eq!(Kitty::legal_actions(&s, Seat(0)).len(), 3);
    act(
        &mut s,
        0,
        Action::Give {
            card: r(10),
            to: Seat(3),
        },
    );
    assert!(s.hands[3].contains(&r(10)));
    assert_eq!(s.round_no, 2);
}

#[test]
fn soft_the_two_lowest_unique_bids_share_the_target() {
    // Three players, round 2: a 4 and a 10 to claim.
    let mut s = table(&[&[2, 9], &[3, 9], &[6, 9]], 2, &[4, 10], &[5, 5]);
    passes(&mut s, &[9, 9, 9]);
    bids(&mut s, &[2, 3, 6]);
    assert_eq!(Kitty::pending(&s), Pending::one(Seat(0)));
    assert_eq!(
        Kitty::legal_actions(&s, Seat(0)),
        vec![Action::Take { card: r(4) }, Action::Take { card: r(10) }]
    );
    act(&mut s, 0, Action::Take { card: r(10) });
    // The 3 takes what is left; the 6 gets nothing.
    assert!(s.hands[0].contains(&r(10)));
    assert!(s.hands[1].contains(&r(4)));
    assert_eq!(s.hands[2], ranks(&[9]));
    assert_eq!((s.round_no, s.target.clone()), (3, ranks(&[5, 5])));
}

#[test]
fn soft_a_lone_unique_bid_leaves_a_card_to_the_discard() {
    let mut s = table(&[&[2, 9], &[2, 9], &[6, 9]], 2, &[7, 7], &[5, 5]);
    passes(&mut s, &[9, 9, 9]);
    // Both cards are 7s: no choice to make.
    bids(&mut s, &[2, 2, 6]);
    assert!(s.hands[2].contains(&r(7)));
    assert!(s.events.contains(&Event::Discarded { cards: ranks(&[7]) }));
}

#[test]
fn the_most_cards_of_each_rank_score_it() {
    // Last round: the 1 is the target and nobody bids uniquely.
    let mut s = table(
        &[
            &[10, 10, 3, 4, 4],
            &[10, 10, 3, 9, 4],
            &[9, 9, 2, 2, 4],
            &[8, 7, 7, 6, 4],
        ],
        7,
        &[1],
        &[],
    );
    passes(&mut s, &[3, 3, 2, 6]);
    // Seat 0 now 10 10 4 4 6, seat 1 10 10 9 4 3, seat 2 9 9 2 4 3,
    // seat 3 8 7 7 4 2.
    bids(&mut s, &[4, 4, 4, 4]);
    assert_eq!(s.phase, Phase::Over);
    // 10: seats 0 and 1 (two each). 9: seat 2. 8, 7: seat 3. 6 and 4
    // (its last one): seat 0. 3: seats 1 and 2. 2: seats 2 and 3.
    assert_eq!(s.scores, vec![20, 13, 14, 17]);
    let Some(Event::Scored { ranks: by_rank, .. }) = s.events.last() else {
        panic!("{:?}", s.events);
    };
    assert_eq!(by_rank[9].winners, vec![Seat(0), Seat(1)]);
    assert!(by_rank[0].winners.is_empty());
    assert_eq!(Kitty::pending(&s), Pending::none());
    assert_eq!(Kitty::outcome(&s), Some(Outcome::Ranked(vec![0, 3, 2, 1])));
}

#[test]
fn a_whole_game_keeps_every_card() {
    for players in 3..=6u8 {
        let mut s = Kitty::new(&Config::default(), players, 11);
        while let Some(&seat) = Kitty::pending(&s).0.first() {
            let action = Kitty::legal_actions(&s, seat)[0].clone();
            act(&mut s, seat.0, action);
            assert_eq!(cards(&s), DECK_SIZE);
        }
        assert_eq!(s.round_no, if players == 4 { 7 } else { 4 });
        assert!(s.kitty.is_empty());
    }
}
