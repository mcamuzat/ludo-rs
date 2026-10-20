use ludo_core::{Game, Outcome, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::testing::{r, ranks};
use pairs::the_judge::testing::table;
use pairs::the_judge::{
    target, Action, Config, Event, Phase, Reveal, State, TheJudge, HAND_SIZE, JUDGE_SIZE,
    MAX_ROUNDS,
};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn act(s: &mut State, seat: u8, action: Action) {
    TheJudge::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn add(card: u8) -> Action {
    Action::Add { card: r(card) }
}

/// Everyone still cooking passes: the contest of this round is played.
fn pass_all(s: &mut State) {
    let round = s.round_no;
    while s.phase == Phase::Cook && s.round_no == round {
        let seat = s.turn.0;
        act(s, seat, Action::Pass);
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        TheJudge::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
}

fn cards(s: &State) -> usize {
    s.hands
        .iter()
        .chain(&s.recipes)
        .map(Vec::len)
        .sum::<usize>()
        + s.judge.len()
        + s.supply.deck_len()
        + s.supply.discard_len()
}

fn contest(s: &State) -> (Vec<Reveal>, Vec<Seat>, Vec<u16>, Vec<u16>) {
    s.events
        .iter()
        .find_map(|e| match e {
            Event::Contest {
                reveals,
                survivors,
                gained,
                scores,
            } => Some((
                reveals.clone(),
                survivors.clone(),
                gained.clone(),
                scores.clone(),
            )),
            _ => None,
        })
        .expect("no contest")
}

#[test]
fn a_round_deals_seven_cards_and_an_eight_card_judge() {
    for players in 2..=6u8 {
        let s = TheJudge::new(&Config::default(), players, 9);
        assert_eq!(s.phase, Phase::Cook);
        assert!(s.hands.iter().all(|h| h.len() == HAND_SIZE));
        assert_eq!(s.judge.len(), JUDGE_SIZE);
        assert_eq!(cards(&s), DECK_SIZE);
        assert_eq!(s.turn, s.leader);
        // Nobody sees the Judge.
        let view = TheJudge::view(&s, Viewer::Seat(Seat(0)));
        assert_eq!(view.judge_size, JUDGE_SIZE);
        assert_eq!(view.target, target(players));
    }
    assert_eq!([2, 3, 4, 5, 6].map(target), [50, 50, 40, 30, 30]);
}

#[test]
fn a_recipe_never_holds_a_pair() {
    let s = table(&[&[4, 4, 7], &[2, 3]], &[&[7], &[]]);
    assert_eq!(
        TheJudge::legal_actions(&s, Seat(0)),
        vec![add(4), Action::Pass]
    );
    refused(&s, 0, add(7));
    refused(&s, 1, add(2));
}

#[test]
fn a_pass_is_final_and_turns_skip_it() {
    let mut s = table(&[&[4, 5], &[2, 3], &[6, 8]], &[&[], &[], &[]]);
    act(&mut s, 0, Action::Pass);
    act(&mut s, 1, add(2));
    act(&mut s, 2, add(6));
    assert_eq!(s.turn, Seat(1));
    assert!(TheJudge::legal_actions(&s, Seat(0)).is_empty());
}

#[test]
fn a_seat_with_nothing_to_add_passes_by_itself() {
    let mut s = table(&[&[4, 5], &[3], &[6, 8]], &[&[], &[3], &[]]);
    act(&mut s, 0, add(4));
    assert_eq!(s.turn, Seat(2));
    assert!(s.passed[1]);
    assert!(s.events.contains(&Event::Passed {
        seat: Seat(1),
        forced: true
    }));
}

#[test]
fn the_judge_eliminates_recipes_until_one_is_left() {
    let mut s = table(&[&[], &[], &[]], &[&[1, 4, 6], &[2, 9], &[3, 6]]);
    s.judge = ranks(&[5, 6, 9, 1, 2, 3, 7, 8]);
    pass_all(&mut s);
    let (reveals, survivors, gained, scores) = contest(&s);
    // 5: nobody; 6: seats 0 and 2; then seat 1 is left alone.
    assert_eq!(
        reveals,
        vec![
            Reveal {
                card: r(5),
                eliminated: vec![]
            },
            Reveal {
                card: r(6),
                eliminated: vec![Seat(0), Seat(2)]
            },
        ]
    );
    assert_eq!(
        (survivors, gained, scores),
        (vec![Seat(1)], vec![0, 11, 0], vec![0, 11, 0])
    );
    // [R] The best score leads the next round.
    assert_eq!((s.round_no, s.leader), (2, Seat(1)));
}

#[test]
fn survivors_of_the_whole_judge_all_score() {
    let mut s = table(&[&[], &[]], &[&[1, 4], &[2, 9]]);
    s.judge = ranks(&[5, 6, 7, 8, 10, 10, 3, 3]);
    pass_all(&mut s);
    let (reveals, survivors, gained, _) = contest(&s);
    assert_eq!(reveals.len(), JUDGE_SIZE);
    assert_eq!((survivors, gained), (vec![Seat(0), Seat(1)], vec![5, 11]));
}

#[test]
fn one_card_can_eliminate_everyone_and_the_same_player_leads() {
    let mut s = table(&[&[], &[]], &[&[1, 4], &[4, 9]]);
    s.leader = Seat(1);
    s.judge = ranks(&[4, 6, 7, 8, 10, 10, 3, 3]);
    pass_all(&mut s);
    let (_, survivors, gained, _) = contest(&s);
    assert!(survivors.is_empty());
    assert_eq!(gained, vec![0, 0]);
    assert_eq!(s.leader, Seat(1));
}

#[test]
fn an_empty_recipe_is_not_entered() {
    // Seat 1 cooked nothing: seat 0 is alone and scores without a reveal.
    let mut s = table(&[&[], &[]], &[&[3, 8], &[]]);
    s.judge = ranks(&[3, 8, 1, 2, 4, 5, 6, 7]);
    pass_all(&mut s);
    let (reveals, survivors, gained, _) = contest(&s);
    assert!(reveals.is_empty());
    assert_eq!((survivors, gained), (vec![Seat(0)], vec![11, 0]));
}

#[test]
fn tied_best_scores_lead_from_the_previous_leader() {
    let mut s = table(&[&[], &[], &[]], &[&[5], &[2, 3], &[1, 4]]);
    s.leader = Seat(1);
    s.judge = ranks(&[6, 7, 8, 9, 10, 10, 10, 10]);
    pass_all(&mut s);
    // Everyone scored 5: seat 1, the previous leader, leads again.
    assert_eq!(contest(&s).2, vec![5, 5, 5]);
    assert_eq!(s.leader, Seat(1));
}

#[test]
fn the_game_ends_at_the_target_unless_the_top_is_tied() {
    let mut s = table(&[&[], &[]], &[&[10], &[1]]);
    s.scores = vec![45, 49];
    s.judge = ranks(&[2, 3, 4, 5, 6, 7, 8, 9]);
    pass_all(&mut s);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.scores, vec![55, 50]);
    assert_eq!(TheJudge::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));

    let mut s = table(&[&[], &[]], &[&[10], &[1]]);
    s.scores = vec![45, 54];
    s.judge = ranks(&[2, 3, 4, 5, 6, 7, 8, 9]);
    pass_all(&mut s);
    assert_eq!(s.scores, vec![55, 55]);
    assert_eq!((s.phase, s.round_no), (Phase::Cook, 2));
}

#[test]
fn the_game_stops_after_the_last_round() {
    let mut s = table(&[&[], &[]], &[&[1], &[1]]);
    s.round_no = MAX_ROUNDS;
    s.judge = ranks(&[2, 3, 4, 5, 6, 7, 8, 9]);
    pass_all(&mut s);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(TheJudge::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let s = TheJudge::new(&Config::default(), 3, 7);
    let hand: Vec<u8> = s.hands[0].iter().map(|c| c.value()).collect();
    let judge: Vec<u8> = s.judge.iter().map(|c| c.value()).collect();
    assert_eq!(
        (hand, judge, s.leader),
        (
            vec![8, 9, 7, 6, 5, 7, 1],
            vec![9, 5, 10, 7, 8, 10, 3, 8],
            Seat(1)
        )
    );
}

#[test]
fn random_games_end_and_conserve_the_cards() {
    let mut rng = SmallRng::seed_from_u64(11);
    for players in 2..=6u8 {
        for _ in 0..20 {
            let mut s = TheJudge::new(&Config::default(), players, rng.gen());
            let mut steps = 0;
            while let Some(&seat) = TheJudge::pending(&s).0.first() {
                assert_eq!(cards(&s), DECK_SIZE);
                let legal = TheJudge::legal_actions(&s, seat);
                let action = legal.choose(&mut rng).unwrap().clone();
                act(&mut s, seat.0, action);
                steps += 1;
                assert!(steps < 100_000);
            }
            let top = *s.scores.iter().max().unwrap();
            assert!(top >= target(players) || s.round_no == MAX_ROUNDS);
        }
    }
}
