use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::horn_of_plenty::testing::table;
use pairs::horn_of_plenty::{
    round_points, Action, Config, Event, HornOfPlenty, Phase, State, HAND_SIZE, MAX_ROUNDS, TARGET,
};
use pairs::testing::{r, ranks};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn act(s: &mut State, seat: u8, action: Action) {
    HornOfPlenty::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn play(card: u8, declared: u8) -> Action {
    Action::Play {
        card: r(card),
        declared: r(declared),
    }
}

fn give(to: u8) -> Action {
    Action::Give { to: Seat(to) }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        HornOfPlenty::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
}

fn cards(s: &State) -> usize {
    s.hands.iter().chain(&s.shown).map(Vec::len).sum::<usize>()
        + s.table.iter().count()
        + s.supply.deck_len()
        + s.supply.discard_len()
}

#[test]
fn a_round_deals_one_card_up_and_four_down() {
    for players in 2..=6u8 {
        let s = HornOfPlenty::new(&Config::default(), players, 5);
        assert_eq!(s.phase, Phase::Declare);
        assert!(s.shown.iter().all(|h| h.len() == 1));
        assert!(s.hands.iter().all(|h| h.len() == HAND_SIZE));
        assert_eq!(cards(&s), DECK_SIZE);
        // [R] The lowest face-up card leads.
        let low = s.shown.iter().map(|h| h[0]).min().unwrap();
        assert_eq!(s.shown[s.turn.index()][0], low);
    }
}

#[test]
fn tie_break_cards_are_thrown_away() {
    for seed in 0..200 {
        let s = HornOfPlenty::new(&Config::default(), 6, seed);
        let Some(Event::Dealt { tie_breaks, .. }) = s.events.first() else {
            panic!("no deal event");
        };
        if !tie_breaks.is_empty() {
            assert!(s.shown.iter().all(|h| h.len() == 1));
            assert_eq!(s.supply.discard_len(), tie_breaks.len());
            return;
        }
    }
    panic!("no tie in 200 deals");
}

#[test]
fn the_card_stays_hidden_until_revealed() {
    let mut s = table(&[&[3, 9], &[5, 6], &[7, 8]], &[&[1], &[2], &[4]], &[]);
    act(&mut s, 0, play(3, 9));
    let theirs = HornOfPlenty::view(&s, Viewer::Seat(Seat(1)));
    let table = theirs.table.unwrap();
    assert_eq!(
        (table.declared, table.card, table.asked),
        (r(9), None, Some(Seat(1)))
    );
    let mine = HornOfPlenty::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(mine.table.unwrap().card, Some(r(3)));
    assert_eq!(
        HornOfPlenty::view(&s, Viewer::Spectator)
            .table
            .unwrap()
            .card,
        None
    );
    // Only the asked player answers.
    refused(&s, 2, Action::Agree);
    refused(&s, 0, Action::Agree);
}

#[test]
fn a_right_answer_lets_the_answerer_decide() {
    // A true declaration, agreed: seat 1 decides between itself and seat 0.
    let mut s = table(&[&[6, 9], &[5, 7], &[7, 8]], &[&[1], &[2], &[4]], &[]);
    act(&mut s, 0, play(6, 6));
    act(&mut s, 1, Action::Agree);
    assert_eq!((s.phase, s.chooser), (Phase::Place, Some(Seat(1))));
    assert_eq!(
        HornOfPlenty::legal_actions(&s, Seat(1)),
        vec![give(0), give(1)]
    );
    refused(&s, 1, give(2));
    act(&mut s, 1, give(1));
    assert_eq!(s.shown[1], ranks(&[2, 6]));
    // A bluff, called: seat 2 is right too.
    act(&mut s, 1, play(5, 10));
    act(&mut s, 2, Action::Disagree);
    assert_eq!(s.chooser, Some(Seat(2)));
}

#[test]
fn a_wrong_answer_lets_the_player_decide() {
    let mut s = table(&[&[6, 9], &[5, 7], &[7, 8]], &[&[1], &[2], &[4]], &[]);
    act(&mut s, 0, play(6, 6));
    act(&mut s, 1, Action::Disagree);
    assert_eq!(s.chooser, Some(Seat(0)));
    let Some(Event::Answered { right, card, .. }) = s.events.last() else {
        panic!("no answer event");
    };
    assert_eq!((*right, *card), (false, r(6)));
    act(&mut s, 0, give(1));
    assert_eq!(s.shown[1], ranks(&[2, 6]));
    // [R] Play proceeds to the left of the card's player.
    assert_eq!((s.phase, s.turn), (Phase::Declare, Seat(1)));
}

#[test]
fn passed_all_the_way_round_its_player_places_it_anywhere() {
    let mut s = table(&[&[6, 9], &[5, 7], &[7, 8]], &[&[1], &[2], &[4]], &[]);
    act(&mut s, 0, play(6, 3));
    act(&mut s, 1, Action::Pass);
    assert_eq!(s.table.as_ref().unwrap().asked, Seat(2));
    act(&mut s, 2, Action::Pass);
    assert_eq!((s.phase, s.chooser), (Phase::Place, Some(Seat(0))));
    assert_eq!(
        HornOfPlenty::legal_actions(&s, Seat(0)),
        vec![give(0), give(1), give(2)]
    );
    assert_eq!(
        HornOfPlenty::view(&s, Viewer::Seat(Seat(2)))
            .table
            .unwrap()
            .card,
        Some(r(6))
    );
    act(&mut s, 0, give(2));
    assert_eq!(s.shown[2], ranks(&[4, 6]));
}

#[test]
fn a_pair_is_thrown_away_at_once() {
    let mut s = table(&[&[8, 9], &[5, 7], &[7, 2]], &[&[1], &[10, 8], &[4]], &[]);
    act(&mut s, 0, play(8, 8));
    act(&mut s, 1, Action::Disagree);
    act(&mut s, 0, give(1));
    assert_eq!(s.shown[1], ranks(&[10]));
    assert_eq!(s.supply.seen(), ranks(&[8, 8]).as_slice());
    assert!(matches!(
        s.events.last(),
        Some(Event::Given { paired: true, .. })
    ));
}

#[test]
fn ties_take_the_higher_points_and_skip_the_next_place() {
    assert_eq!(round_points(&[18, 18, 12, 5, 0]), vec![5, 5, 3, 2, 1]);
    assert_eq!(round_points(&[7, 9]), vec![1, 2]);
    assert_eq!(round_points(&[4, 4, 4]), vec![3, 3, 3]);
}

/// Plays the last cards of a two-seat round: each seat keeps its own card.
fn finish_round(s: &mut State) {
    while s.phase != Phase::Over && s.round_no == 1 {
        let seat = s.turn.0;
        let card = s.hands[seat as usize][0].value();
        act(s, seat, play(card, card));
        act(s, 1 - seat, Action::Disagree);
        act(s, seat, give(seat));
    }
}

#[test]
fn the_round_scores_by_card_points() {
    let mut s = table(&[&[10], &[3]], &[&[1], &[2]], &[]);
    s.scores = vec![4, 4];
    finish_round(&mut s);
    let Some(Event::Scored {
        points,
        gained,
        scores,
    }) = s
        .events
        .iter()
        .find(|e| matches!(e, Event::Scored { .. }))
        .cloned()
    else {
        panic!("no score event");
    };
    assert_eq!(
        (points, gained, scores),
        (vec![11, 5], vec![2, 1], vec![6, 5])
    );
    assert_eq!((s.round_no, s.phase), (2, Phase::Declare));
}

#[test]
fn the_game_ends_at_eleven_unless_the_top_is_tied() {
    let mut s = table(&[&[10], &[3]], &[&[1], &[2]], &[]);
    s.scores = vec![TARGET - 1, TARGET - 1];
    finish_round(&mut s);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(HornOfPlenty::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));

    // 11 and 11: another round.
    let mut s = table(&[&[3], &[3]], &[&[1], &[1]], &[]);
    s.scores = vec![TARGET - 2, TARGET - 2];
    finish_round(&mut s);
    assert_eq!(s.scores, vec![TARGET, TARGET]);
    assert_eq!((s.round_no, s.phase), (2, Phase::Declare));
}

#[test]
fn the_game_stops_after_the_last_round() {
    let mut s = table(&[&[3], &[3]], &[&[1], &[1]], &[]);
    s.round_no = MAX_ROUNDS;
    s.scores = vec![TARGET, TARGET];
    while s.phase != Phase::Over {
        let seat = s.turn.0;
        let card = s.hands[seat as usize][0].value();
        act(&mut s, seat, play(card, card));
        act(&mut s, 1 - seat, Action::Disagree);
        act(&mut s, seat, give(seat));
    }
    assert_eq!(HornOfPlenty::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let s = HornOfPlenty::new(&Config::default(), 3, 7);
    let up: Vec<u8> = s.shown.iter().map(|h| h[0].value()).collect();
    let hand: Vec<u8> = s.hands[0].iter().map(|c| c.value()).collect();
    assert_eq!(
        (up, hand, s.turn),
        (vec![8, 8, 5], vec![9, 7, 6, 5], Seat(2))
    );
}

#[test]
fn random_games_end_and_conserve_the_cards() {
    let mut rng = SmallRng::seed_from_u64(11);
    for players in 2..=6u8 {
        for _ in 0..20 {
            let mut s = HornOfPlenty::new(&Config::default(), players, rng.gen());
            let mut steps = 0;
            while let Some(&seat) = HornOfPlenty::pending(&s).0.first() {
                assert_eq!(cards(&s), DECK_SIZE);
                let legal = HornOfPlenty::legal_actions(&s, seat);
                let action = legal.choose(&mut rng).unwrap().clone();
                act(&mut s, seat.0, action);
                steps += 1;
                assert!(steps < 100_000);
            }
            assert!(HornOfPlenty::outcome(&s).is_some());
            let top = *s.scores.iter().max().unwrap();
            assert!(top >= TARGET || s.round_no == MAX_ROUNDS);
        }
    }
}
