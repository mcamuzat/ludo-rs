use cards::hattrick::testing::{c, table};
use cards::hattrick::{round_points, Action, Color, Config, Hattrick, State};
use ludo_core::{Game, Outcome, Seat};

fn play(s: &mut State, seat: u8, card: &str) {
    Hattrick::apply(s, Seat(seat), &Action::Play { card: c(card) }).unwrap();
}

fn pass(s: &mut State, seat: u8, card: &str) {
    Hattrick::apply(s, Seat(seat), &Action::Pass { card: c(card) }).unwrap();
}

#[test]
fn the_deal_matches_the_table_size() {
    for (players, size) in [(4u8, 15usize), (5, 12), (6, 10)] {
        let s = Hattrick::new(&Config::default(), players, 7);
        assert!(s.hands.iter().all(|h| h.len() == size));
        let mut all: Vec<_> = s.hands.concat();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 60);
        // The player at the dealer's left leads.
        assert_eq!(s.turn, Some(Seat(1)));
    }
}

#[test]
fn any_card_while_one_pile_is_open() {
    let mut s = table(&["r5 b3 g9", "r7 b8 g2", "r1 b1 g1", "r2 b2 g3"], 0);
    // No pass with fewer than two piles.
    assert!(Hattrick::legal_actions(&s, Seat(0))
        .iter()
        .all(|a| matches!(a, Action::Play { .. })));
    play(&mut s, 0, "r5");
    assert_eq!(Hattrick::legal_actions(&s, Seat(1)).len(), 3);
    assert!(Hattrick::apply(&mut s, Seat(1), &Action::Pass { card: c("g2") }).is_err());
}

#[test]
fn a_third_colour_only_passes() {
    let mut s = table(&["r5 b3 g9", "b8 r7 g2", "r1 b1 g1", "r2 b2 g3"], 0);
    play(&mut s, 0, "r5");
    play(&mut s, 1, "b8");
    assert_eq!(s.pile_colors(), vec![Color::Red, Color::Blue]);
    let before = s.clone();
    assert!(Hattrick::apply(&mut s, Seat(2), &Action::Play { card: c("g1") }).is_err());
    assert!(Hattrick::apply(&mut s, Seat(2), &Action::Pass { card: c("r1") }).is_err());
    assert_eq!(s, before, "an illegal move changes nothing");
    let legal = Hattrick::legal_actions(&s, Seat(2));
    assert!(legal.contains(&Action::Pass { card: c("g1") }));
    assert!(legal.contains(&Action::Play { card: c("r1") }));
    pass(&mut s, 2, "g1");
    assert_eq!(s.passed[2], 1);
}

#[test]
fn each_pile_goes_to_its_highest_card() {
    let mut s = table(&["r5 b3 g9", "b8 r7 g2", "r1 b1 g1", "b2 r2 g3"], 0);
    play(&mut s, 0, "r5");
    play(&mut s, 1, "b8");
    pass(&mut s, 2, "g1");
    play(&mut s, 3, "r2");
    // Red 5 and 2 to seat 0; blue 8 to seat 1.
    assert_eq!(s.won[0], [2, 0, 0]);
    assert_eq!(s.won[1], [0, 1, 0]);
    let last = s.last_trick.clone().unwrap();
    assert_eq!(last.piles.len(), 2);
    // Blue 8 is the strongest card: seat 1 leads.
    assert_eq!(last.leader, Seat(1));
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn equal_tops_go_to_the_second_card() {
    let mut s = table(&["r8 b3 g9", "b8 r7 g2", "r1 b1 g1", "b2 r2 g3"], 0);
    play(&mut s, 0, "r8");
    play(&mut s, 1, "b8");
    play(&mut s, 2, "b1");
    play(&mut s, 3, "r2");
    // 8-2 beats 8-1: the red pile's winner leads.
    assert_eq!(s.turn, Some(Seat(0)));
}

#[test]
fn the_last_card_is_not_played_and_the_round_is_scored() {
    let mut s = table(&["r5 g9", "b8 g2", "r1 g1", "r2 g3"], 0);
    play(&mut s, 0, "r5");
    play(&mut s, 1, "b8");
    pass(&mut s, 2, "g1");
    play(&mut s, 3, "r2");
    // One card each left: the round is over and scored.
    assert_eq!(s.round, 1);
    let lines = s.last_round.clone().unwrap();
    assert_eq!(lines[0].points, 2);
    assert_eq!(lines[1].points, 1);
    assert_eq!(lines[2].points, -2);
    assert_eq!(s.scores, vec![2, 1, -2, 0]);
    assert_eq!(
        s.hands.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![15; 4]
    );
}

#[test]
fn the_example_of_the_rule_scores_one() {
    // 9 red, 2 blue, 4 green and a card put down: 9 - 2 - 4 - 2.
    assert_eq!(round_points([9, 2, 4], 1), 1);
    assert_eq!(round_points([5, 5, 0], 0), 0);
    assert_eq!(round_points([0, 0, 0], 2), -4);
}

#[test]
fn the_game_lasts_two_rounds_per_player() {
    let mut s = table(&["r5 g9", "b8 g2", "r1 g1", "r2 g3"], 0);
    s.round = 7;
    play(&mut s, 0, "r5");
    play(&mut s, 1, "b8");
    pass(&mut s, 2, "g1");
    play(&mut s, 3, "r2");
    assert_eq!(s.turn, None);
    assert_eq!(
        Hattrick::outcome(&s),
        Some(Outcome::Ranked(vec![0, 1, 3, 2]))
    );
}

#[test]
fn four_to_six_players() {
    assert!(Hattrick::check_config(&Config::default(), 4).is_ok());
    assert!(Hattrick::check_config(&Config::default(), 6).is_ok());
    assert!(Hattrick::check_config(&Config::default(), 3).is_err());
}
