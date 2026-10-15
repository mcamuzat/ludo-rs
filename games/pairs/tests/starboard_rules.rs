use ludo_core::{Game, Outcome, Pending, Seat};
use pairs::starboard::testing::table;
use pairs::starboard::{Action, Config, Event, Phase, Starboard, State, Status};
use pairs::testing::{r, ranks};

fn act(s: &mut State, seat: u8, action: Action) {
    Starboard::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn balance(s: &State) -> i64 {
    s.chips.iter().map(|&c| i64::from(c)).sum::<i64>() + i64::from(s.pot)
}

fn ended(s: &State) -> Option<(Vec<Seat>, u32, u32)> {
    s.events.iter().find_map(|e| match e {
        Event::RoundEnded {
            winners,
            share,
            left,
        } => Some((winners.clone(), *share, *left)),
        _ => None,
    })
}

#[test]
fn a_round_antes_and_opens_two_distinct_cards() {
    for players in 2..=7u8 {
        for seed in 0..10 {
            let s = Starboard::new(&Config::default(), players, seed);
            assert_eq!(s.center.len(), 2);
            assert_ne!(s.center[0], s.center[1]);
            assert_eq!(s.pot, u32::from(players));
            assert_eq!(s.turn, Some(s.button));
            assert_eq!(s.rounds_total, 2 * players);
            assert_eq!(balance(&s), 0);
        }
    }
}

#[test]
fn a_hit_costs_a_chip_and_a_pair_busts() {
    let mut s = table(3, 6, &[4, 9], &[9]);
    act(&mut s, 0, Action::Hit);
    assert_eq!(s.chips[0], -2);
    assert_eq!(s.pot, 4);
    assert_eq!(s.status[0], Status::Busted);
    // The paired card is discarded: the middle stays as it was.
    assert_eq!(s.center, ranks(&[4, 9]));
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn a_clean_draw_knocks_out_the_locked_players() {
    let mut s = table(4, 8, &[4, 9], &[1, 2]);
    act(&mut s, 0, Action::Lock);
    act(&mut s, 1, Action::Lock);
    act(&mut s, 2, Action::Hit);
    assert_eq!(s.center, ranks(&[4, 9, 2]));
    assert!(s.events.contains(&Event::Hit {
        seat: Seat(2),
        card: r(2),
        knocked: vec![Seat(0), Seat(1)],
    }));
    assert_eq!(
        s.status,
        vec![Status::Knocked, Status::Knocked, Status::In, Status::In]
    );
    assert_eq!(s.turn, Some(Seat(3)));
}

#[test]
fn the_survivors_split_the_pot_and_odd_chips_stay() {
    let mut s = table(3, 6, &[4, 9], &[1]);
    act(&mut s, 0, Action::Lock);
    act(&mut s, 1, Action::Lock);
    act(&mut s, 2, Action::Lock);
    // 3 chips, three survivors: one each.
    assert_eq!(ended(&s), Some((vec![Seat(0), Seat(1), Seat(2)], 1, 0)));
    let mut s = table(3, 6, &[4, 9], &[4]);
    act(&mut s, 0, Action::Hit); // bust: 4 chips in the pot
    act(&mut s, 1, Action::Lock);
    act(&mut s, 2, Action::Lock);
    assert_eq!(ended(&s), Some((vec![Seat(1), Seat(2)], 2, 0)));
    let mut s = table(4, 8, &[4, 9], &[4]);
    act(&mut s, 0, Action::Hit); // bust: 5 chips
    act(&mut s, 1, Action::Lock);
    act(&mut s, 2, Action::Lock);
    act(&mut s, 3, Action::Lock);
    assert_eq!(ended(&s), Some((vec![Seat(1), Seat(2), Seat(3)], 1, 2)));
    // Round 2: the button moves left; the odd chips meet the new antes.
    assert_eq!(s.button, Seat(1));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.pot, 2 + 4);
    assert_eq!(balance(&s), 0);
}

#[test]
fn the_last_player_left_locks_by_itself() {
    let mut s = table(3, 6, &[4, 9], &[9, 4]);
    act(&mut s, 0, Action::Hit); // 4: bust
    act(&mut s, 1, Action::Hit); // 9: bust
                                 // Seat 2 is alone: it locks and takes the pot (3 antes + 2 hits).
    assert!(s.events.contains(&Event::Locked {
        seat: Seat(2),
        auto: true
    }));
    assert_eq!(ended(&s), Some((vec![Seat(2)], 5, 0)));
    // A hitter that knocks out every locked player is left alone too.
    let mut s = table(2, 4, &[4, 9], &[1]);
    act(&mut s, 0, Action::Lock);
    act(&mut s, 1, Action::Hit);
    assert_eq!(ended(&s), Some((vec![Seat(1)], 3, 0)));
}

#[test]
fn the_game_ends_after_its_rounds() {
    let mut s = table(2, 1, &[4, 9], &[1]);
    act(&mut s, 0, Action::Lock);
    act(&mut s, 1, Action::Lock);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Starboard::pending(&s), Pending::none());
    assert_eq!(Starboard::outcome(&s), Some(Outcome::Ranked(vec![0, 0])));
    let mut s = Starboard::new(&Config::default(), 4, 9);
    while let Some(&seat) = Starboard::pending(&s).0.first() {
        let action = Starboard::legal_actions(&s, seat)[0].clone();
        act(&mut s, seat.0, action);
        assert_eq!(balance(&s), 0);
    }
    assert_eq!(s.round_no, 8);
}
