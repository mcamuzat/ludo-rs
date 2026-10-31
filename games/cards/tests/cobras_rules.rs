use cards::cobras::testing::{c, cards, table};
use cards::cobras::{
    basket_points, pack, Action, Cobras, Config, DummyPile, Phase, Played, Share, State, Who,
};
use ludo_core::{Game, Outcome, Seat, Viewer};

fn play(s: &mut State, seat: u8, card: &str) {
    Cobras::apply(s, Seat(seat), &Action::Play { card: c(card) }).unwrap();
}

fn king(s: &mut State, seat: u8, share: Share) {
    Cobras::apply(s, Seat(seat), &Action::King { share }).unwrap();
}

#[test]
fn the_pack_has_sixteen_triples() {
    let p = pack();
    assert_eq!(p.len(), 52);
    assert_eq!(p.iter().filter(|c| c.is_triple()).count(), 16);
    assert_eq!(c("y4").cobras(), 1);
    assert_eq!(c("y5").cobras(), 2);
    assert_eq!(c("y9").cobras(), 2);
    assert_eq!(c("y10").cobras(), 3);
}

#[test]
fn the_basket_scale_peaks_at_seven() {
    let scale: Vec<i32> = (0..=12).map(basket_points).collect();
    assert_eq!(scale, vec![0, 1, 3, 5, 8, 11, 15, 21, 15, 8, 3, 3, 3]);
}

#[test]
fn deals_follow_the_table_size() {
    for (players, hand, aside, dummy) in
        [(2, 17, 1, 17), (3, 17, 1, 0), (4, 13, 0, 0), (5, 10, 2, 0)]
    {
        let s = Cobras::new(&Config::default(), players, 11);
        assert!(s.hands.iter().all(|h| h.len() == hand), "{players}");
        assert_eq!(s.set_aside.len(), aside);
        assert_eq!(s.dummy.len(), dummy);
        assert_eq!(s.phase, Phase::Seed);
    }
}

#[test]
fn baskets_are_started_in_secret_then_together() {
    let mut s = Cobras::new(&Config::default(), 3, 4);
    let picks: Vec<_> = (0..3).map(|i| s.hands[i][0]).collect();
    Cobras::apply(&mut s, Seat(1), &Action::Seed { card: picks[1] }).unwrap();
    // Nobody else learns that seat 1 chose.
    assert_eq!(
        Cobras::visible_pending(&s, Viewer::Seat(Seat(0))).0,
        vec![Seat(0)]
    );
    assert!(Cobras::visible_pending(&s, Viewer::Spectator).is_empty());
    assert!(Cobras::apply(&mut s, Seat(1), &Action::Seed { card: picks[1] }).is_err());
    Cobras::apply(&mut s, Seat(0), &Action::Seed { card: picks[0] }).unwrap();
    Cobras::apply(&mut s, Seat(2), &Action::Seed { card: picks[2] }).unwrap();
    assert_eq!(s.phase, Phase::Trick);
    for (i, pick) in picks.iter().enumerate() {
        assert_eq!(s.baskets[i], pick.cobras());
        assert_eq!(s.hands[i].len(), 16);
        assert_eq!(s.revealed[i], Some(*pick));
    }
    let triples = picks.iter().filter(|c| c.is_triple()).count() as u32;
    assert_eq!(s.triples, triples);
    assert_eq!(s.turn, Some(Who::Seat(s.first_leader)));
}

#[test]
fn the_led_suit_must_be_followed_and_the_king_is_always_allowed() {
    let mut s = table(&["r3 g4", "r9 b2", "b5 b6"], "", 0);
    play(&mut s, 0, "r3");
    let legal = Cobras::legal_actions(&s, Seat(1));
    assert_eq!(
        legal,
        vec![
            Action::Play { card: c("r9") },
            Action::King { share: Share::Win },
            Action::King { share: Share::Loss },
        ]
    );
    assert!(Cobras::apply(&mut s, Seat(1), &Action::Play { card: c("b2") }).is_err());
}

#[test]
fn the_winner_scores_its_basket_and_the_losers_share_the_rest() {
    let mut s = table(&["r3 g4", "r12 b2", "b5 b6"], "", 0);
    s.baskets = vec![0, 4, 1];
    s.center = 1;
    play(&mut s, 0, "r3");
    play(&mut s, 1, "r12");
    play(&mut s, 2, "b5");
    // r3 (1) + r12 (3) + b5 (2) + 1 in the centre = 7, between two losers.
    assert_eq!(s.scores, vec![0, 8, 0]);
    assert_eq!(s.baskets, vec![3, 0, 4]);
    assert_eq!(s.center, 1);
    assert_eq!(s.triples, 1);
    assert_eq!(s.turn, Some(Who::Seat(Seat(1))));
    assert!(s.void[2][c("r1").suit.index()]);
    let last = s.last_trick.as_ref().unwrap();
    assert_eq!(last.winner, Some(Who::Seat(Seat(1))));
    assert_eq!(last.share, 3);
}

#[test]
fn the_rulebook_example() {
    // [K] p. 7: A leads red 5, B plays red 12, C shares the win with a King
    // Cobra. B turns in 4 cobras for 8 points, C 7 for 21; A receives the
    // 5 cobras of the cards plus 1 from the centre.
    let mut s = table(&["r5 g1", "r12 g2", "g3 g4"], "", 0);
    s.baskets = vec![0, 4, 7];
    s.center = 1;
    play(&mut s, 0, "r5");
    play(&mut s, 1, "r12");
    king(&mut s, 2, Share::Win);
    assert_eq!(s.scores, vec![0, 8, 21]);
    assert_eq!(s.baskets, vec![6, 0, 0]);
    assert_eq!(s.center, 0);
    assert!(!s.kings[2]);
    assert_eq!(s.turn, Some(Who::Seat(Seat(1))));
    // Once a round: seat 1 leads, seat 2 has no King left.
    play(&mut s, 1, "g2");
    assert!(Cobras::legal_actions(&s, Seat(2))
        .iter()
        .all(|a| matches!(a, Action::Play { .. })));
}

#[test]
fn sharing_the_loss_takes_a_share() {
    let mut s = table(&["r5 g1", "r12 g2", "g3 g4"], "", 0);
    play(&mut s, 0, "r5");
    play(&mut s, 1, "r12");
    king(&mut s, 2, Share::Loss);
    // 5 cobras between two losers: 2 each, 1 left.
    assert_eq!(s.baskets, vec![2, 0, 2]);
    assert_eq!(s.center, 1);
}

#[test]
fn a_king_lead_lets_the_next_card_set_the_suit() {
    let mut s = table(&["r5 g1", "g12 r2", "g3 r4"], "", 0);
    king(&mut s, 0, Share::Loss);
    // Seat 1 may play anything: no suit is led yet.
    assert_eq!(Cobras::legal_actions(&s, Seat(1)).len(), 4);
    play(&mut s, 1, "r2");
    play(&mut s, 2, "r4");
    assert_eq!(
        s.last_trick.as_ref().unwrap().winner,
        Some(Who::Seat(Seat(2)))
    );
    assert_eq!(s.turn, Some(Who::Seat(Seat(2))));
}

#[test]
fn a_trick_of_kings_has_no_winner_and_its_leader_leads_again() {
    let mut s = table(&["r5 g1", "g12 r2", "g3 r4"], "", 1);
    s.baskets = vec![2, 3, 5];
    s.center = 4;
    king(&mut s, 1, Share::Win);
    king(&mut s, 2, Share::Loss);
    king(&mut s, 0, Share::Loss);
    let last = s.last_trick.as_ref().unwrap();
    assert_eq!(last.winner, None);
    // ❓ A declared win still turns the basket in.
    assert_eq!(s.scores, vec![0, 5, 0]);
    assert_eq!(s.baskets, vec![4, 0, 7]);
    assert_eq!(s.center, 0);
    assert_eq!(s.turn, Some(Who::Seat(Seat(1))));
}

#[test]
fn with_no_loser_the_cobras_stay_in_the_centre() {
    let mut s = table(&["r5 g1", "r12 g2", "g3 g4"], "", 0);
    s.center = 2;
    play(&mut s, 0, "r5");
    king(&mut s, 1, Share::Win);
    king(&mut s, 2, Share::Win);
    assert_eq!(s.center, 2 + 2);
    assert_eq!(s.baskets, vec![0, 0, 0]);
}

#[test]
fn the_dummy_follows_on_suit_but_loses_ties() {
    // Seat 0 deals, so the order is 0, dummy, 1.
    let mut s = table(&["r7 g1 b1", "r9 g2 b2"], "b9 y13 g3 b3", 0);
    play(&mut s, 0, "r7");
    // The dummy played b9 at once, counted as red.
    assert_eq!(s.trick.len(), 2);
    assert_eq!(s.turn, Some(Who::Seat(Seat(1))));
    play(&mut s, 1, "r9");
    let last = s.last_trick.as_ref().unwrap();
    assert_eq!(last.winner, Some(Who::Seat(Seat(1))));
    // Next trick: seat 1 leads, seat 0 follows, then the dummy's y13 wins.
    play(&mut s, 1, "g2");
    play(&mut s, 0, "g1");
    let last = s.last_trick.as_ref().unwrap();
    assert_eq!(last.winner, Some(Who::Dummy));
    assert_eq!(last.losers, vec![Who::Seat(Seat(1)), Who::Seat(Seat(0))]);
    // The dummy leads the next trick at once.
    assert_eq!(s.leader, Who::Dummy);
    assert_eq!(s.trick.len(), 1);
    assert_eq!(s.turn, Some(Who::Seat(Seat(1))));
}

#[test]
fn the_dummy_leading_sets_the_suit() {
    let mut s = table(&["r7 g1 g8", "r9 g2 g6 b4"], "g5 y13 g3 b1", 0);
    play(&mut s, 0, "r7");
    play(&mut s, 1, "r9");
    // Seat 1 leads; then the dummy wins the next trick and leads.
    play(&mut s, 1, "g2");
    play(&mut s, 0, "g1");
    assert_eq!(s.leader, Who::Dummy);
    // The dummy led g3: seat 1 must follow green if it can.
    assert_eq!(s.trick[0].played, Played::Card(c("g3")));
    assert_eq!(
        Cobras::legal_actions(&s, Seat(1))
            .into_iter()
            .filter(|a| matches!(a, Action::Play { .. }))
            .count(),
        1
    );
}

#[test]
fn ten_triples_end_the_round_with_the_basket_penalty() {
    let mut s = table(&["r10 g1", "r11 g2", "r12 g3"], "", 0);
    s.triples = 7;
    s.baskets = vec![0, 0, 2];
    s.scores = vec![10, 20, 30];
    play(&mut s, 0, "r10");
    play(&mut s, 1, "r11");
    play(&mut s, 2, "r12");
    // Seat 2 won (basket 2 → 3 points); seats 0 and 1 got 4 cobras each.
    let lines = s.last_round.clone().unwrap();
    assert_eq!(lines[0].penalty, 4);
    assert_eq!(lines[2].gained, 3);
    assert_eq!(s.scores, vec![6, 16, 33]);
    assert_eq!(s.round, 1);
    assert_eq!(s.phase, Phase::Seed);
    // The fewest points lead the next round.
    assert_eq!(s.first_leader, Seat(0));
    assert!(s.kings.iter().all(|&k| k));
    assert_eq!(s.center, 0);
}

#[test]
fn equal_lowest_scores_go_clockwise_from_the_last_leader() {
    let mut s = table(&["r10 g1", "r11 g2", "r12 g3"], "", 1);
    s.triples = 9;
    play(&mut s, 1, "r11");
    play(&mut s, 2, "r12");
    play(&mut s, 0, "r10");
    // Seats 0 and 1 got 4 cobras each and lose 4; seat 2 scores 0.
    assert_eq!(s.scores, vec![-4, -4, 0]);
    // After seat 1 clockwise: seat 2 (not lowest), then seat 0.
    assert_eq!(s.first_leader, Seat(0));
}

#[test]
fn the_game_ends_at_100_or_75_at_five() {
    let mut s = table(&["r10 g1", "r11 g2", "r12 g3"], "", 0);
    s.triples = 9;
    s.baskets = vec![0, 0, 7];
    s.scores = vec![0, 0, 80];
    play(&mut s, 0, "r10");
    play(&mut s, 1, "r11");
    play(&mut s, 2, "r12");
    assert_eq!(s.scores[2], 101);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Cobras::outcome(&s), Some(Outcome::Ranked(vec![1, 1, 0])));

    let five = table(&["r1", "r2", "r3", "r4", "r5"], "", 0);
    assert_eq!(five.target(), 75);
}

#[test]
fn an_empty_hand_ends_the_round() {
    // Seat 0 played its King earlier and holds one card more.
    let mut s = table(&["r3 g4", "r9", "b5"], "", 0);
    play(&mut s, 0, "r3");
    play(&mut s, 1, "r9");
    play(&mut s, 2, "b5");
    assert_eq!(s.round, 1);
}

#[test]
fn the_hidden_dummy_pile_shows_nothing() {
    let shown = Cobras::new(&Config::default(), 2, 3);
    let hidden = Cobras::new(
        &Config {
            dummy: DummyPile::Hidden,
        },
        2,
        3,
    );
    for mut s in [shown, hidden] {
        for seat in [Seat(0), Seat(1)] {
            let card = s.hands[seat.index()][0];
            Cobras::apply(&mut s, seat, &Action::Seed { card }).unwrap();
        }
        let v = Cobras::view(&s, Viewer::Seat(Seat(0)));
        let top = v.dummy.unwrap().top;
        match s.config.dummy {
            DummyPile::Shown => assert_eq!(top, Some(s.dummy[0])),
            DummyPile::Hidden => assert_eq!(top, None),
        }
    }
}

#[test]
fn the_timeout_plays_the_lowest_card() {
    let s = table(&["r3 g4 r1", "r9", "b5"], "", 0);
    assert_eq!(
        Cobras::timeout_action(&s, Seat(0)),
        Some(Action::Play { card: c("r1") })
    );
    assert_eq!(cards("r1 g2").len(), 2);
}
