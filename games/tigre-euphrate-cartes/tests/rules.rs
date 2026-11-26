use ludo_core::{Game, Seat};
use tigre_euphrate_cartes::{
    sorted_points, Action, Color, Config, ConflictKind, EndReason, Face, Leader, Phase, Pos, Slot,
    State, TigreEuphrateCartes as G, ACTIONS, HAND,
};

use Color::{Black, Blue, Green, Red};

/// A table with empty hands, seat 0 to play.
fn game(players: u8) -> State {
    let mut s = G::new(&Config::default(), players, 1);
    s.turn = Seat(0);
    s.hands = vec![[0; 4]; usize::from(players)];
    s
}

fn civ(c: Color) -> Slot {
    Slot {
        face: Face::Civ(c),
        leader: None,
    }
}

fn led(c: Color, seat: u8, color: Color) -> Slot {
    Slot {
        face: Face::Civ(c),
        leader: Some(Leader {
            seat: Seat(seat),
            color,
        }),
    }
}

fn hand(s: &mut State, seat: u8, black: u8, red: u8, blue: u8, green: u8) {
    s.hands[usize::from(seat)] = [black, red, blue, green];
}

fn play(s: &mut State, seat: u8, a: Action) {
    G::apply(s, Seat(seat), &a).unwrap_or_else(|e| panic!("{a:?} : {e}"));
}

#[test]
fn the_deal_follows_the_table_size() {
    let s = G::new(&Config::default(), 2, 9);
    assert_eq!(s.deck.len(), 185 - 2 * 8 - 30);
    assert_eq!(s.removed.len(), 30);
    let s = G::new(&Config::default(), 4, 9);
    assert_eq!(s.deck.len(), 185 - 4 * 8);
    assert!(s.removed.is_empty());
    assert!(s.hands.iter().all(|h| h.iter().sum::<u8>() == HAND));
    assert_eq!(s.treasures_left(), 8);
    assert_eq!(s.table.gaps.iter().flatten().count(), 0);
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let s = G::new(&Config::default(), 3, 2026);
    assert_eq!(s.hands, vec![[3, 3, 2, 0], [3, 2, 1, 2], [1, 3, 2, 2]]);
    assert_eq!(s.turn, Seat(0));
}

#[test]
fn a_card_scores_for_the_leader_of_its_colour() {
    let mut s = game(2);
    s.table.heads[0].leader = Some(Leader {
        seat: Seat(1),
        color: Red,
    });
    hand(&mut s, 0, 0, 2, 0, 0);
    hand(&mut s, 1, 0, 1, 0, 0);
    play(
        &mut s,
        0,
        Action::Extend {
            color: Red,
            column: 0,
        },
    );
    assert_eq!(s.scores[1], [0, 1, 0, 0]);
    assert_eq!(s.hands[1], [0, 0, 0, 0], "the point is laid from the hand");
    assert_eq!(s.hands[0], [0, 1, 0, 0]);
    // No red card left in hand: no point.
    play(
        &mut s,
        0,
        Action::Extend {
            color: Red,
            column: 0,
        },
    );
    assert_eq!(s.scores[1], [0, 1, 0, 0]);
}

#[test]
fn the_king_scores_when_the_colour_has_no_leader() {
    let mut s = game(2);
    s.table.columns[0] = vec![led(Red, 0, Black)];
    hand(&mut s, 0, 0, 0, 0, 2);
    play(
        &mut s,
        0,
        Action::Extend {
            color: Green,
            column: 0,
        },
    );
    assert_eq!(s.scores[0], [0, 0, 0, 1]);
    // A merchant there takes it from the king, card in hand or not.
    s.table.columns[0][1].leader = Some(Leader {
        seat: Seat(1),
        color: Green,
    });
    hand(&mut s, 0, 0, 0, 0, 2);
    play(
        &mut s,
        0,
        Action::Extend {
            color: Green,
            column: 0,
        },
    );
    assert_eq!(s.scores[0], [0, 0, 0, 1]);
    assert_eq!(s.scores[1], [0, 0, 0, 0]);
}

#[test]
fn a_junction_needs_three_cards_under_both_treasures() {
    let mut s = game(2);
    hand(&mut s, 0, 1, 0, 0, 0);
    s.table.columns[0] = vec![civ(Blue); 3];
    s.table.columns[1] = vec![civ(Blue); 2];
    assert!(!s.may_join(0));
    assert!(G::apply(
        &mut s,
        Seat(0),
        &Action::Join {
            color: Black,
            gap: 0
        }
    )
    .is_err());
    s.table.columns[1].push(civ(Blue));
    play(
        &mut s,
        0,
        Action::Join {
            color: Black,
            gap: 0,
        },
    );
    assert_eq!(s.table.gaps[0], Some(civ(Black)));
    assert_eq!(s.table.span(1, None), (0, 1));
    assert_eq!(s.scores[0], [0; 4], "a junction scores nothing");
}

/// [R] internal conflict example: Arc places his merchant where Lion's
/// stands; Lion's king on a red card gives him 1.
#[test]
fn internal_conflict_from_the_rulebook() {
    let mut s = game(2);
    // Arc is seat 0, Lion seat 1.
    s.table.heads[0].leader = Some(Leader {
        seat: Seat(1),
        color: Black,
    });
    s.table.columns[0] = vec![led(Green, 1, Green), civ(Blue)];
    hand(&mut s, 0, 0, 4, 0, 0);
    hand(&mut s, 1, 0, 1, 0, 0);
    play(
        &mut s,
        0,
        Action::Leader {
            color: Green,
            to: Pos::Card(0, 1),
        },
    );
    let Phase::Conflict(c) = s.phase else {
        panic!("conflict expected, got {:?}", s.phase)
    };
    assert_eq!(c.kind, ConflictKind::Internal);
    assert_eq!((c.attacker, c.defender), (Seat(0), Seat(1)));
    assert_eq!((c.attacker_base, c.defender_base), (0, 1));
    play(&mut s, 0, Action::Commit { count: 4 });
    play(&mut s, 1, Action::Commit { count: 0 });
    assert_eq!(s.scores[0], [0, 1, 0, 0]);
    assert_eq!(s.discards, 3);
    assert_eq!(
        s.supply(Seat(1)),
        vec![Red, Blue, Green],
        "Lion's merchant goes home"
    );
    assert_eq!(s.phase, Phase::Action);
    assert_eq!(s.actions_left, ACTIONS - 1);
}

#[test]
fn ties_go_to_the_defender_who_scores_from_his_hand() {
    let mut s = game(2);
    s.table.columns[0] = vec![led(Blue, 1, Blue), civ(Blue)];
    hand(&mut s, 0, 0, 0, 1, 0);
    hand(&mut s, 1, 0, 2, 0, 0);
    // The attacker has no red card: not asked.
    play(
        &mut s,
        0,
        Action::Leader {
            color: Blue,
            to: Pos::Card(0, 1),
        },
    );
    play(&mut s, 1, Action::Commit { count: 0 });
    assert_eq!(s.scores[1], [0, 1, 0, 0]);
    assert_eq!(s.hands[1], [0, 1, 0, 0]);
    assert_eq!(s.supply(Seat(0)), vec![Black, Red, Blue, Green]);
}

/// [R] external conflict example: Vase joins kingdoms 1 and 2; green and
/// red conflicts. Green first: Lion attacks with 3 + 2, Taureau defends
/// with 1 + 4 and wins the tie, taking Lion's three green cards, the old
/// junction among them; the kingdom splits and the red conflict is off.
#[test]
fn external_conflict_from_the_rulebook() {
    let mut s = game(4);
    let (vase, lion, taureau) = (0u8, 1u8, 2u8);
    s.table.heads[0].leader = Some(Leader {
        seat: Seat(lion),
        color: Red,
    });
    s.table.columns[0] = vec![civ(Red), civ(Blue), civ(Black)];
    s.table.gaps[0] = Some(civ(Green));
    s.table.columns[1] = vec![led(Red, lion, Green), civ(Green), civ(Green)];
    s.table.columns[2] = vec![
        led(Blue, taureau, Green),
        civ(Green),
        led(Red, taureau, Red),
    ];
    hand(&mut s, vase, 0, 0, 0, 1);
    hand(&mut s, lion, 0, 0, 0, 2);
    hand(&mut s, taureau, 0, 0, 0, 4);
    play(
        &mut s,
        vase,
        Action::Join {
            color: Green,
            gap: 1,
        },
    );
    assert_eq!(s.phase, Phase::ChooseConflict(vec![Red, Green]));
    play(&mut s, vase, Action::Resolve { color: Green });
    let Phase::Conflict(c) = s.phase else {
        panic!("conflict expected")
    };
    assert_eq!((c.attacker, c.defender), (Seat(lion), Seat(taureau)));
    assert_eq!((c.attacker_base, c.defender_base), (3, 1));
    play(&mut s, lion, Action::Commit { count: 2 });
    play(&mut s, taureau, Action::Commit { count: 4 });
    assert_eq!(s.scores[usize::from(taureau)], [0, 0, 0, 4]);
    assert_eq!(s.discards, 5);
    assert_eq!(s.table.gaps[0], None, "the old junction is taken");
    assert_eq!(s.table.columns[1], vec![civ(Red)]);
    assert_eq!(s.supply(Seat(lion)), vec![Black, Blue, Green]);
    // The priests are apart now: no red conflict; the junction turns up.
    assert_eq!(s.phase, Phase::Action);
    assert_eq!(s.join, None);
    assert_eq!(s.table.gaps[1], Some(civ(Green)));
    assert_eq!(s.treasures[usize::from(taureau)], 0, "no red card to swap");
}

#[test]
fn the_merchant_swaps_one_treasure_for_a_red_card() {
    let mut s = game(2);
    s.table.columns[0] = vec![led(Black, 1, Green), civ(Blue), civ(Blue)];
    s.table.columns[1] = vec![civ(Blue); 3];
    s.table.columns[2] = vec![civ(Blue); 3];
    s.table.gaps[1] = Some(civ(Blue));
    hand(&mut s, 0, 1, 0, 0, 0);
    hand(&mut s, 1, 0, 2, 0, 0);
    play(
        &mut s,
        0,
        Action::Join {
            color: Black,
            gap: 0,
        },
    );
    assert_eq!(
        s.treasures[1], 1,
        "one treasure only, though three are there"
    );
    assert_eq!(s.hands[1], [0, 1, 0, 0]);
    assert_eq!(s.table.heads[0].face, Face::Exchanged);
    assert_eq!(s.treasures_left(), 7);
}

#[test]
fn four_blue_cards_build_any_ship() {
    let mut s = game(2);
    s.table.columns[3] = vec![civ(Red), civ(Blue), led(Blue, 1, Black), civ(Blue)];
    hand(&mut s, 0, 0, 0, 1, 0);
    play(
        &mut s,
        0,
        Action::Extend {
            color: Blue,
            column: 3,
        },
    );
    assert_eq!(
        s.phase,
        Phase::Ship {
            column: 3,
            options: vec![Black, Red, Green]
        }
    );
    play(&mut s, 0, Action::Ship { ship: Red });
    let ship = Slot {
        face: Face::Ship(Red),
        leader: None,
    };
    assert_eq!(s.table.columns[3], vec![civ(Red), ship]);
    assert_eq!(s.ships, vec![Black, Green]);
    assert_eq!(s.supply(Seat(1)), vec![Black, Red, Blue, Green]);
    assert!(!s.leader_may_stand(Pos::Card(3, 1)), "never on a ship");
    assert!(!s.may_destroy(Pos::Card(3, 1)));
}

#[test]
fn four_green_cards_need_the_green_ship() {
    let mut s = game(2);
    s.table.columns[0] = vec![civ(Green); 3];
    s.ships = vec![Black, Red];
    hand(&mut s, 0, 0, 0, 0, 2);
    play(
        &mut s,
        0,
        Action::Extend {
            color: Green,
            column: 0,
        },
    );
    assert_eq!(s.phase, Phase::Action, "no blue-green ship left");
    s.ships = vec![Green];
    s.table.columns[2] = vec![civ(Green); 3];
    play(
        &mut s,
        0,
        Action::Extend {
            color: Green,
            column: 2,
        },
    );
    assert!(matches!(s.phase, Phase::Ship { .. }));
    play(&mut s, 0, Action::NoShip);
    assert_eq!(s.table.columns[2].len(), 4);
}

#[test]
fn a_ship_scores_the_players_leaders_at_the_end_of_his_turn() {
    let mut s = game(2);
    s.table.columns[0] = vec![
        led(Red, 0, Blue),
        Slot {
            face: Face::Ship(Black),
            leader: None,
        },
    ];
    s.table.heads[0].leader = Some(Leader {
        seat: Seat(0),
        color: Black,
    });
    s.table.columns[1] = vec![led(Red, 0, Green)];
    s.hands = vec![[2, 2, 2, 2], [8, 0, 0, 0]];
    play(&mut s, 0, Action::EndTurn);
    assert_eq!(s.scores[0], [1, 0, 1, 0], "the farmer and the king");
    assert_eq!(s.turn, Seat(1));
    assert_eq!(s.hands[0].iter().sum::<u8>(), HAND, "hands are refilled");
}

#[test]
fn a_catastrophe_removes_a_free_card_and_the_rest_moves_up() {
    let mut s = game(2);
    s.table.columns[0] = vec![civ(Red), led(Blue, 1, Blue), civ(Green), civ(Black)];
    assert!(!s.may_destroy(Pos::Head(0)));
    assert!(!s.may_destroy(Pos::Card(0, 1)));
    play(
        &mut s,
        0,
        Action::Catastrophe {
            at: Pos::Card(0, 2),
        },
    );
    assert_eq!(
        s.table.columns[0],
        vec![civ(Red), led(Blue, 1, Blue), civ(Black)]
    );
    assert!(!s.catastrophes[0]);
    assert!(G::apply(
        &mut s,
        Seat(0),
        &Action::Catastrophe {
            at: Pos::Card(0, 0)
        }
    )
    .is_err());
}

#[test]
fn two_actions_make_a_turn() {
    let mut s = game(2);
    hand(&mut s, 0, 0, 0, 2, 0);
    hand(&mut s, 1, 1, 1, 1, 1);
    play(
        &mut s,
        0,
        Action::Extend {
            color: Blue,
            column: 0,
        },
    );
    assert_eq!(s.turn, Seat(0));
    play(
        &mut s,
        0,
        Action::Leader {
            color: Black,
            to: Pos::Head(5),
        },
    );
    assert_eq!(s.turn, Seat(1));
    assert_eq!(s.actions_left, ACTIONS);
    assert!(s.hands.iter().all(|h| h.iter().sum::<u8>() == HAND));
    assert_eq!(s.last_turn.as_ref().unwrap().steps.len(), 2);
}

#[test]
fn the_game_ends_when_the_deck_cannot_refill_every_hand() {
    let mut s = game(2);
    s.hands = vec![[0, 0, 1, 0], [8, 0, 0, 0]];
    s.deck = vec![Red; 6];
    play(&mut s, 0, Action::EndTurn);
    assert_eq!(s.end, Some(EndReason::Deck));
    assert!(G::outcome(&s).is_some());
}

#[test]
fn the_game_ends_with_one_treasure_left() {
    let mut s = game(2);
    for head in &mut s.table.heads[1..] {
        head.face = Face::Exchanged;
    }
    play(&mut s, 0, Action::EndTurn);
    assert_eq!(s.end, Some(EndReason::Treasures));
}

#[test]
fn a_refused_action_leaves_the_state_alone() {
    let mut s = game(2);
    let before = s.clone();
    for a in [
        Action::Extend {
            color: Red,
            column: 0,
        },
        Action::Join { color: Red, gap: 0 },
        Action::Commit { count: 1 },
        Action::Catastrophe { at: Pos::Head(0) },
        Action::NoShip,
    ] {
        assert!(G::apply(&mut s, Seat(0), &a).is_err(), "{a:?}");
    }
    assert!(G::apply(&mut s, Seat(1), &Action::EndTurn).is_err());
    assert_eq!(s, before);
}

#[test]
fn treasures_are_jokers_and_ties_look_at_the_next_colour() {
    assert_eq!(sorted_points(&[5, 7, 6, 5], 0), [5, 5, 6, 7]);
    assert_eq!(sorted_points(&[3, 7, 6, 5], 2), [5, 5, 6, 7]);
    let mut s = game(3);
    s.scores = vec![[5, 7, 6, 5], [6, 4, 6, 6], [4, 9, 5, 6]];
    // [R] example: Arc (seat 0) beats Lion and Taureau; a treasure lifts
    // Taureau above Arc on the third colour.
    assert_eq!(s.ranks(), vec![0, 1, 2]);
    s.treasures = vec![0, 0, 1];
    assert_eq!(s.ranks(), vec![1, 2, 0]);
}
