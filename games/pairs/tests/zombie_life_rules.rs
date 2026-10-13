use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::deck::DECK_SIZE;
use pairs::testing::{r, ranks};
use pairs::zombie_life::testing::{dealing, table};
use pairs::zombie_life::{Action, Config, Event, Phase, State, Status, ZombieLife, Zone};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn act(s: &mut State, seat: u8, action: Action) {
    ZombieLife::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn take(card: u8) -> Action {
    Action::Take { card: r(card) }
}

fn give(card: u8) -> Action {
    Action::Give { card: r(card) }
}

const TO_REFUGE: Action = Action::Place { zone: Zone::Refuge };
const TO_HORDE: Action = Action::Place { zone: Zone::Horde };

fn legal(s: &State, seat: u8) -> Vec<Action> {
    ZombieLife::legal_actions(s, Seat(seat))
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        ZombieLife::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!legal(s, seat).contains(&action));
}

fn cards(s: &State) -> usize {
    s.refuges.iter().map(Vec::len).sum::<usize>()
        + s.horde.len()
        + s.dealing.len()
        + usize::from(s.drawn.is_some())
        + s.supply.deck_len()
        + s.supply.discard_len()
}

#[test]
fn a_new_game_deals_one_card_each_and_the_lowest_starts() {
    for players in 3..=6u8 {
        for seed in 0..20 {
            let s = ZombieLife::new(&Config::default(), players, seed);
            assert_eq!((s.phase, s.round, s.target), (Phase::Play, 1, 21));
            assert!(s.refuges.iter().all(|refuge| refuge.len() == 1));
            assert!(s.horde.is_empty());
            assert!(s.scores.iter().all(|&score| score == 0));
            assert_eq!(cards(&s), DECK_SIZE);
            let first = s.turn.expect("someone starts");
            let mine = s.refuges[first.index()][0];
            assert!(s.refuges.iter().all(|refuge| refuge[0] >= mine));
            assert_eq!(ZombieLife::pending(&s), Pending::one(first));
        }
    }
}

#[test]
fn a_tie_for_the_lowest_card_is_drawn_from_the_seed() {
    let mut starters = Vec::new();
    for seed in 0..40 {
        let mut s = dealing(&[0, 0, 0], 0, &[3, 3, 5], &[]);
        s.seed = seed;
        act(&mut s, 0, give(3));
        act(&mut s, 0, give(5));
        assert_eq!(s.refuges, vec![ranks(&[3]), ranks(&[3]), ranks(&[5])]);
        let first = s.turn.expect("someone starts");
        assert!(first == Seat(0) || first == Seat(1), "{first:?}");
        let mut again = dealing(&[0, 0, 0], 0, &[3, 3, 5], &[]);
        again.seed = seed;
        act(&mut again, 0, give(3));
        act(&mut again, 0, give(5));
        assert_eq!(again.turn, s.turn);
        starters.push(first);
    }
    starters.sort_unstable();
    starters.dedup();
    assert_eq!(starters.len(), 2);
}

#[test]
fn a_drawn_card_goes_to_the_refuge_or_the_horde() {
    let mut s = table(&[&[5], &[6], &[7]], &[], &[9, 5, 5, 6]);
    act(&mut s, 0, Action::Draw);
    assert_eq!((s.phase, s.drawn), (Phase::Placing, Some(r(9))));
    assert_eq!(ZombieLife::pending(&s), Pending::one(Seat(0)));
    assert_eq!(legal(&s, 0), vec![TO_REFUGE, TO_HORDE]);
    act(&mut s, 0, TO_REFUGE);
    assert_eq!(s.refuges[0], ranks(&[5, 9]));
    assert_eq!(
        (s.phase, s.drawn, s.turn),
        (Phase::Play, None, Some(Seat(1)))
    );

    act(&mut s, 1, Action::Draw);
    act(&mut s, 1, TO_HORDE);
    assert_eq!(s.horde, ranks(&[5]));
    assert_eq!(s.refuges[1], ranks(&[6]));

    // The horde already holds a 5: it can only go home.
    act(&mut s, 2, Action::Draw);
    assert_eq!(legal(&s, 2), vec![TO_REFUGE]);
    refused(&s, 2, TO_HORDE);
    act(&mut s, 2, TO_REFUGE);

    // A 6 at home already: it can only go to the horde.
    act(&mut s, 0, Action::Pass);
    act(&mut s, 1, Action::Draw);
    assert_eq!(legal(&s, 1), vec![TO_HORDE]);
    refused(&s, 1, TO_REFUGE);
}

#[test]
fn a_card_in_both_zones_overruns_the_player() {
    let mut s = table(&[&[5, 2], &[6], &[7]], &[5], &[5]);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.status[0], Status::Out);
    assert!(s.refuges[0].is_empty());
    assert_eq!(
        (s.phase, s.drawn, s.turn),
        (Phase::Play, None, Some(Seat(1)))
    );
    assert!(s.events.contains(&Event::Overrun {
        seat: Seat(0),
        card: r(5)
    }));
    assert_eq!(s.supply.discard_len(), 3);
    assert_eq!(s.horde, ranks(&[5]));
    assert_eq!(s.eliminated, vec![Seat(0)]);
}

#[test]
fn taking_moves_a_card_from_the_horde() {
    let mut s = table(&[&[5], &[6], &[7]], &[5, 8], &[1]);
    assert_eq!(legal(&s, 0), vec![Action::Draw, take(8), Action::Pass]);
    act(&mut s, 0, take(8));
    assert_eq!(s.refuges[0], ranks(&[5, 8]));
    assert_eq!(s.horde, ranks(&[5]));
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn the_last_player_in_cannot_take_from_the_horde() {
    let mut s = table(&[&[5], &[6], &[7]], &[8], &[1, 2, 3, 4]);
    act(&mut s, 0, Action::Pass);
    act(&mut s, 1, Action::Pass);
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(legal(&s, 2), vec![Action::Draw, Action::Pass]);
    refused(&s, 2, take(8));
    // It may still draw, and feed the horde.
    act(&mut s, 2, Action::Draw);
    assert_eq!(legal(&s, 2), vec![TO_REFUGE, TO_HORDE]);
    act(&mut s, 2, TO_HORDE);
    assert_eq!(s.turn, Some(Seat(2)));
    act(&mut s, 2, Action::Pass);
    assert_eq!(s.round, 2);
}

#[test]
fn the_turn_skips_players_who_passed_or_are_out() {
    let mut s = table(&[&[5], &[6], &[7, 2], &[8]], &[2], &[9, 2, 9]);
    act(&mut s, 0, Action::Pass);
    assert_eq!(s.turn, Some(Seat(1)));
    act(&mut s, 1, Action::Draw);
    act(&mut s, 1, TO_REFUGE);
    act(&mut s, 2, Action::Draw);
    assert_eq!(s.status[2], Status::Out);
    assert_eq!(s.turn, Some(Seat(3)));
    act(&mut s, 3, Action::Draw);
    act(&mut s, 3, TO_REFUGE);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(ZombieLife::pending(&s), Pending::one(Seat(1)));
}

#[test]
fn passing_hides_one_card_per_player_still_in() {
    let mut s = table(&[&[1, 2, 3, 4], &[5], &[6], &[7]], &[], &[1]);
    act(&mut s, 0, Action::Pass);
    assert_eq!((s.status[0], s.hidden[0]), (Status::Passed, 3));
    assert_eq!(
        s.events,
        vec![Event::Passed {
            seat: Seat(0),
            hidden: 3
        }]
    );
    // Fewer cards than players still in: the whole refuge, no more.
    act(&mut s, 1, Action::Pass);
    assert_eq!(s.hidden[1], 1);
    let view = ZombieLife::view(&s, Viewer::Spectator);
    assert_eq!(
        (view.seats[0].face_up.len(), view.seats[0].face_down),
        (1, 3)
    );
    assert_eq!(
        (view.seats[1].face_up.len(), view.seats[1].face_down),
        (0, 1)
    );
    assert_eq!(view.seats[0].status, Status::Passed);
}

#[test]
fn an_elimination_turns_one_card_back_up() {
    let mut s = table(
        &[&[1, 2, 3, 4], &[5], &[6, 9], &[7, 9]],
        &[9],
        &[9, 9, 1, 2, 3, 4],
    );
    act(&mut s, 0, Action::Pass);
    act(&mut s, 1, Action::Pass);
    assert_eq!(s.hidden, vec![3, 1, 0, 0]);
    act(&mut s, 2, Action::Draw);
    assert_eq!(s.status[2], Status::Out);
    assert_eq!(s.hidden, vec![2, 0, 0, 0]);
    // Nothing left to turn up for seat 1.
    act(&mut s, 3, Action::Draw);
    assert_eq!(s.round, 2);
    assert_eq!(s.scores, vec![3, 1, 0, 0]);
}

#[test]
fn passed_players_score_their_face_up_cards() {
    let mut s = table(&[&[1, 2, 3, 4], &[5, 6], &[7]], &[], &[1, 2, 3]);
    act(&mut s, 0, Action::Pass);
    act(&mut s, 1, Action::Pass);
    act(&mut s, 2, Action::Pass);
    assert_eq!(s.scores, vec![2, 1, 1]);
    for (seat, points) in [(0, 2), (1, 1), (2, 1)] {
        assert!(s.events.contains(&Event::Scored {
            seat: Seat(seat),
            points
        }));
    }
    // Everything goes to the discards before the next deal.
    assert!(s.horde.is_empty());
    assert_eq!(s.status, vec![Status::In; 3]);
    assert_eq!(s.hidden, vec![0; 3]);
}

#[test]
fn a_round_everyone_loses_scores_nothing() {
    let mut s = table(&[&[5], &[6], &[7]], &[5, 6, 7], &[5, 6, 7, 1, 2, 3]);
    let total = cards(&s);
    for seat in 0..3 {
        act(&mut s, seat, Action::Draw);
    }
    assert_eq!(s.scores, vec![0, 0, 0]);
    assert_eq!((s.round, s.phase), (2, Phase::Dealing));
    // All tied at 0: the first eliminated deals.
    assert_eq!(s.turn, Some(Seat(0)));
    assert!(s.events.contains(&Event::Dealer { seat: Seat(0) }));
    assert_eq!(s.dealing, ranks(&[1, 2, 3]));
    assert_eq!(cards(&s), total);
    assert!(s.eliminated.is_empty());
}

#[test]
fn the_draw_pile_is_rebuilt_from_the_discards() {
    let mut s = table(&[&[5], &[6], &[7]], &[5, 6, 7], &[5, 6, 7]);
    let total = cards(&s);
    for seat in 0..3 {
        act(&mut s, seat, Action::Draw);
    }
    assert!(s.events.contains(&Event::Reshuffled));
    assert_eq!(s.round, 2);
    assert_eq!(cards(&s), total);
}

#[test]
fn the_lowest_score_deals_the_next_round() {
    // Seats 1 and 2 tie for the lowest score: seat 2 was eliminated first.
    let mut s = table(&[&[5], &[6], &[7]], &[6, 7], &[7, 6, 1, 2, 3]);
    s.scores = vec![4, 2, 2];
    s.turn = Some(Seat(2));
    act(&mut s, 2, Action::Draw);
    act(&mut s, 0, Action::Pass);
    act(&mut s, 1, Action::Draw);
    assert_eq!(s.scores, vec![5, 2, 2]);
    assert_eq!((s.phase, s.turn), (Phase::Dealing, Some(Seat(2))));
    assert!(s.events.contains(&Event::Dealer { seat: Seat(2) }));
    assert_eq!(s.deal_to(), Some(Seat(0)));

    // A lowest score alone deals, eliminated or not.
    let mut s = table(&[&[5], &[6], &[7]], &[6, 7], &[7, 1, 2, 3]);
    s.scores = vec![4, 0, 2];
    s.turn = Some(Seat(2));
    act(&mut s, 2, Action::Draw);
    act(&mut s, 0, Action::Pass);
    act(&mut s, 1, Action::Pass);
    assert_eq!(s.turn, Some(Seat(1)));

    // Tied and nobody eliminated: one of the tied seats, from the seed.
    let mut dealers = Vec::new();
    for seed in 0..40 {
        let mut s = table(&[&[], &[], &[5]], &[], &[1, 2, 3]);
        s.seed = seed;
        s.scores = vec![0, 0, 5];
        for seat in 0..3 {
            act(&mut s, seat, Action::Pass);
        }
        let dealer = s.turn.expect("a dealer");
        assert!(dealer == Seat(0) || dealer == Seat(1), "{dealer:?}");
        dealers.push(dealer);
    }
    dealers.sort_unstable();
    dealers.dedup();
    assert_eq!(dealers.len(), 2);
}

#[test]
fn the_dealer_gives_one_card_to_each_seat() {
    let mut s = dealing(&[3, 0, 3], 1, &[4, 8, 2], &[1]);
    assert_eq!(ZombieLife::pending(&s), Pending::one(Seat(1)));
    assert_eq!(legal(&s, 1), vec![give(2), give(4), give(8)]);
    assert!(legal(&s, 0).is_empty());
    // From the dealer's left, the dealer last.
    assert_eq!(s.deal_to(), Some(Seat(2)));
    act(&mut s, 1, give(8));
    assert_eq!(s.refuges[2], ranks(&[8]));
    assert_eq!(s.deal_to(), Some(Seat(0)));
    act(&mut s, 1, give(2));
    // The last card needs no choice.
    assert_eq!(s.refuges, vec![ranks(&[2]), ranks(&[4]), ranks(&[8])]);
    assert!(s.dealing.is_empty());
    assert_eq!((s.phase, s.turn), (Phase::Play, Some(Seat(0))));
    assert_eq!(
        s.events,
        vec![
            Event::Dealt {
                seat: Seat(0),
                card: r(2)
            },
            Event::Dealt {
                seat: Seat(1),
                card: r(4)
            },
            Event::Starts { seat: Seat(0) },
        ]
    );
}

#[test]
fn identical_cards_are_dealt_without_asking() {
    let mut s = table(&[&[5], &[6], &[7]], &[], &[9, 9, 9, 1]);
    for seat in 0..3 {
        act(&mut s, seat, Action::Pass);
    }
    assert_eq!((s.round, s.phase), (2, Phase::Play));
    assert_eq!(s.refuges, vec![ranks(&[9]); 3]);
    assert!(s.turn.is_some());
}

#[test]
fn the_game_ends_at_the_target_and_ties_are_shared() {
    let mut s = table(&[&[1, 2, 3], &[4, 5, 6], &[7]], &[], &[1, 2, 3]);
    s.scores = vec![20, 19, 3];
    for seat in 0..3 {
        assert_eq!(ZombieLife::outcome(&s), None);
        act(&mut s, seat, Action::Pass);
    }
    assert_eq!(s.scores, vec![21, 21, 4]);
    assert_eq!((s.phase, s.turn), (Phase::Over, None));
    assert!(ZombieLife::pending(&s).is_empty());
    assert_eq!(
        ZombieLife::outcome(&s),
        Some(Outcome::Ranked(vec![0, 0, 2]))
    );
    for seat in 0..3 {
        assert!(legal(&s, seat).is_empty());
        refused(&s, seat, Action::Draw);
    }
}

#[test]
fn the_target_comes_from_the_config() {
    let target = |t| ZombieLife::new(&Config { target: t }, 3, 1).target;
    assert_eq!(target(None), 21);
    assert_eq!(target(Some(0)), 21);
    assert_eq!(target(Some(5)), 5);
    assert_eq!(target(Some(255)), 255);
}

#[test]
fn absurd_actions_are_refused() {
    let s = table(&[&[5], &[6], &[7]], &[5, 8], &[9]);
    refused(&s, 0, take(5));
    refused(&s, 0, take(9));
    refused(&s, 0, TO_REFUGE);
    refused(&s, 0, give(5));
    refused(&s, 1, Action::Draw);
    refused(&s, 7, Action::Draw);

    let mut placing = s.clone();
    act(&mut placing, 0, Action::Draw);
    refused(&placing, 0, Action::Draw);
    refused(&placing, 0, Action::Pass);
    refused(&placing, 0, take(8));
    refused(&placing, 1, TO_REFUGE);

    let d = dealing(&[0, 0, 0], 1, &[4, 8, 2], &[]);
    refused(&d, 1, give(5));
    refused(&d, 1, Action::Draw);
    refused(&d, 1, Action::Pass);
    refused(&d, 0, give(4));
}

#[test]
fn the_view_is_the_same_for_everyone_and_holds_no_seed() {
    let mut s = dealing(&[3, 0, 3], 1, &[4, 8, 2], &[1]);
    s.seed = 123_456_789;
    let spectator = ZombieLife::view(&s, Viewer::Spectator);
    for seat in 0..3 {
        assert_eq!(ZombieLife::view(&s, Viewer::Seat(Seat(seat))), spectator);
    }
    assert_eq!(spectator.deal_to, Some(Seat(2)));
    assert_eq!(spectator.dealing, ranks(&[4, 8, 2]));
    let text = serde_json::to_string(&spectator).unwrap();
    assert!(!text.contains("seed") && !text.contains("supply"), "{text}");
    assert!(!text.contains("123456789"));
    assert!(text.contains("\"dealTo\":2") && text.contains("\"faceUp\""));
}

#[test]
fn actions_have_the_json_the_board_sends() {
    let json = |a: &Action| serde_json::to_string(a).unwrap();
    assert_eq!(json(&Action::Draw), "\"Draw\"");
    assert_eq!(json(&Action::Pass), "\"Pass\"");
    assert_eq!(json(&TO_REFUGE), "{\"Place\":{\"zone\":\"Refuge\"}}");
    assert_eq!(json(&take(4)), "{\"Take\":{\"card\":4}}");
    assert_eq!(json(&give(4)), "{\"Give\":{\"card\":4}}");
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let s = ZombieLife::new(&Config::default(), 4, 7);
    let dealt: Vec<u8> = s.refuges.iter().map(|refuge| refuge[0].value()).collect();
    assert_eq!((dealt, s.turn), (vec![4, 4, 4, 10], Some(Seat(2))));
}

#[test]
fn random_games_end_and_conserve_the_cards() {
    let mut rng = SmallRng::seed_from_u64(11);
    for players in 3..=6u8 {
        for _ in 0..25 {
            let mut s = ZombieLife::new(&Config::default(), players, rng.gen());
            let mut steps = 0;
            while let Some(&seat) = ZombieLife::pending(&s).0.first() {
                let actions = ZombieLife::legal_actions(&s, seat);
                let action = actions.choose(&mut rng).expect("a pending seat can act");
                ZombieLife::apply(&mut s, seat, action).unwrap();
                assert_eq!(cards(&s), DECK_SIZE);
                for refuge in &s.refuges {
                    let mut sorted = refuge.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    assert_eq!(sorted.len(), refuge.len(), "a pair in a refuge");
                }
                steps += 1;
                assert!(steps < 50_000, "the game does not end");
            }
            assert!(s.scores.iter().any(|&score| score >= 21));
            assert!(ZombieLife::outcome(&s).is_some());
        }
    }
}
