use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::sweep::testing::table;
use pairs::sweep::{Action, Config, Event, Phase, State, Sweep};
use pairs::testing::{r, ranks};

fn act(s: &mut State, seat: u8, action: Action) {
    Sweep::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn shoot(s: &mut State, seat: u8, card: u8) {
    act(s, seat, Action::Shoot { card: r(card) });
}

fn sorted(mut cards: Vec<pairs::deck::Rank>) -> Vec<pairs::deck::Rank> {
    cards.sort_unstable();
    cards
}

fn cards(s: &State) -> usize {
    s.hands.iter().map(Vec::len).sum::<usize>()
        + s.supply.deck_len()
        + s.targets.len()
        + s.shooters.len()
        + s.discard.len()
}

#[test]
fn the_deal_and_the_number_of_rounds() {
    for players in 3..=6u8 {
        let s = Sweep::new(&Config::default(), players, 4);
        assert!(s.hands.iter().all(|h| h.len() == 5));
        assert_eq!(s.targets.len(), 5);
        assert_eq!(s.rounds_total, 11 - players);
        assert_eq!(s.turn, Some(s.leader));
        assert_eq!(cards(&s), DECK_SIZE);
    }
}

#[test]
fn the_booklet_example() {
    // [R] Targets 3, 4, 4, 6, 10; shooters 5, 6, 7, 7.
    let mut s = table(&[&[5], &[6], &[7], &[7]], &[3, 4, 4, 6, 10], 0, &[]);
    shoot(&mut s, 0, 5);
    assert_eq!(Sweep::pending(&s), Pending::one(Seat(1)));
    shoot(&mut s, 1, 6);
    shoot(&mut s, 2, 7);
    shoot(&mut s, 3, 7);
    // The 5 takes the 10; the 6 sweeps 3, 4, 4 and the fallen 5; the 7s
    // take one 6 each, the first played first. Nothing is left.
    assert_eq!(s.hands[0], ranks(&[10]));
    assert_eq!(sorted(s.hands[1].clone()), ranks(&[3, 4, 4, 5]));
    assert_eq!(s.hands[2], ranks(&[6]));
    assert_eq!(s.hands[3], ranks(&[6]));
    assert!(s.events.contains(&Event::Discarded {
        cards: ranks(&[7, 7])
    }));
    // One round: the score.
    assert_eq!(s.phase, Phase::Over);
}

#[test]
fn tied_lowest_shooters_share_the_highest_original_targets() {
    let mut s = table(&[&[2], &[2], &[9]], &[5, 8, 8, 8, 3], 1, &[]);
    shoot(&mut s, 1, 2);
    shoot(&mut s, 2, 9);
    shoot(&mut s, 0, 2);
    // Seat 1 played first: 8, then seat 0: 8, then seat 1 again: 8. The 9
    // sweeps the 5, the 3 and the two fallen 2s.
    assert_eq!(s.hands[1], ranks(&[8, 8]));
    assert_eq!(s.hands[0], ranks(&[8]));
    assert_eq!(sorted(s.hands[2].clone()), ranks(&[2, 2, 3, 5]));
}

#[test]
fn tied_shooters_choose_among_targets_of_different_ranks() {
    let mut s = table(&[&[1], &[9], &[9]], &[3, 6, 5, 4, 2], 0, &[]);
    s.rounds_total = 2;
    s.supply =
        pairs::deck::Supply::from_parts(ranks(&[7, 7, 7, 7, 7]), Vec::new(), Vec::new(), 0, 0);
    shoot(&mut s, 0, 1);
    shoot(&mut s, 1, 9);
    shoot(&mut s, 2, 9);
    // The 1 took the 6; the 9s take turns among 2, 3, 4, 5 and the 1.
    assert_eq!(s.hands[0], ranks(&[6]));
    assert_eq!(s.phase, Phase::Capture);
    assert_eq!(Sweep::pending(&s), Pending::one(Seat(1)));
    assert_eq!(Sweep::legal_actions(&s, Seat(1)).len(), 5);
    act(&mut s, 1, Action::Capture { card: r(5) });
    assert_eq!(Sweep::pending(&s), Pending::one(Seat(2)));
    act(&mut s, 2, Action::Capture { card: r(4) });
    act(&mut s, 1, Action::Capture { card: r(3) });
    act(&mut s, 2, Action::Capture { card: r(1) });
    // The last 2 is forced.
    assert_eq!(sorted(s.hands[1].clone()), ranks(&[2, 3, 5]));
    assert_eq!(sorted(s.hands[2].clone()), ranks(&[1, 4]));
    // Seat 1 took the last target: it leads round 2.
    assert_eq!(s.round_no, 2);
    assert_eq!(s.leader, Seat(1));
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn an_empty_hand_does_not_shoot() {
    let mut s = table(&[&[], &[4], &[6]], &[9, 1, 2, 3, 5], 0, &[]);
    s.turn = Some(Seat(1));
    assert!(Sweep::legal_actions(&s, Seat(0)).is_empty());
    shoot(&mut s, 1, 4);
    shoot(&mut s, 2, 6);
    assert_eq!(s.hands[1], ranks(&[9]));
    assert_eq!(sorted(s.hands[2].clone()), ranks(&[1, 2, 3, 4, 5]));
}

#[test]
fn the_most_cards_of_a_rank_score_it() {
    let mut s = table(
        &[&[10, 10, 3, 2], &[10, 10, 9, 1], &[9, 9, 3, 2]],
        &[8, 7, 6, 5, 4],
        0,
        &[],
    );
    shoot(&mut s, 0, 2);
    shoot(&mut s, 1, 1);
    shoot(&mut s, 2, 2);
    // The 1 takes the 8 and falls; the 2s take turns at the 1 (seat 0).
    // Seat 0: 10 10 3 1, seat 1: 10 10 9 8, seat 2: 9 9 3.
    assert_eq!(s.scores, vec![10 + 3 + 1, 10 + 8, 9 + 3]);
    let Some(Event::Scored { scores, .. }) = s.events.last().cloned() else {
        panic!("{:?}", s.events);
    };
    assert_eq!(scores, s.scores);
    assert_eq!(Sweep::view(&s, Viewer::Spectator).scores, scores);
    let ranks_out = Sweep::outcome(&s);
    assert!(matches!(ranks_out, Some(Outcome::Ranked(_))));
}

#[test]
fn a_whole_game_keeps_every_card() {
    for players in 3..=6u8 {
        let mut s = Sweep::new(&Config::default(), players, 12);
        while let Some(&seat) = Sweep::pending(&s).0.first() {
            let action = Sweep::legal_actions(&s, seat)[0].clone();
            act(&mut s, seat.0, action);
            assert_eq!(cards(&s), DECK_SIZE);
        }
        assert_eq!(s.round_no, 11 - players);
    }
}
