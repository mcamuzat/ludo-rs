//! The rules of [R], one test per point. Scenes are laid on an emptied
//! board, on the land rows 3 to 5 (x 4 to 12), away from the rivers.

use ludo_core::{Determinize, Game, Outcome, Seat, Viewer};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use tigre_euphrate::{
    final_points, Action, Color, Config, ConflictKind, EndReason, Phase, Piece, Reinforcements,
    State, TigreEuphrate,
};

use Color::{Black, Blue, Green, Red};

const S0: Seat = Seat(0);
const S1: Seat = Seat(1);
const S2: Seat = Seat(2);

fn at(x: u8, y: u8) -> u8 {
    y * 16 + x
}

/// A game whose board is empty (no temples, no treasures), seat 0 to play.
fn empty(players: u8, reinforcements: Reinforcements) -> State {
    let mut s = TigreEuphrate::new(&Config { reinforcements }, players, 7);
    s.cells.iter_mut().for_each(|c| *c = None);
    s.treasure.iter_mut().for_each(|t| *t = false);
    for i in 0..usize::from(players) {
        s.hands[i] = [0; 4];
    }
    s
}

fn tile(s: &mut State, x: u8, y: u8, color: Color) {
    s.cells[usize::from(at(x, y))] = Some(Piece::Tile {
        color,
        flipped: false,
    });
}

fn leader(s: &mut State, x: u8, y: u8, seat: Seat, color: Color) {
    s.cells[usize::from(at(x, y))] = Some(Piece::Leader { seat, color });
}

fn play(s: &mut State, seat: Seat, a: Action) {
    TigreEuphrate::apply(s, seat, &a).unwrap_or_else(|e| panic!("{a:?} refusé : {e}"));
}

fn refused(s: &State, seat: Seat, a: Action) -> bool {
    let mut copy = s.clone();
    let refused = TigreEuphrate::apply(&mut copy, seat, &a).is_err();
    assert!(!refused || copy == *s, "un refus ne change pas l'état");
    refused
}

#[test]
fn a_tile_scores_for_its_leader_else_the_king() {
    let mut s = empty(2, Reinforcements::English);
    tile(&mut s, 5, 4, Red);
    leader(&mut s, 6, 4, S1, Green);
    leader(&mut s, 5, 5, S0, Black);
    s.hands[0] = [1, 0, 0, 2];
    // A market next to the trader: the trader's owner scores green.
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Green,
            to: at(7, 4),
        },
    );
    assert_eq!(s.scores[1][Green.index()], 1);
    // A settlement: no black leader but the king himself; he scores.
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Black,
            to: at(4, 4),
        },
    );
    assert_eq!(s.scores[0][Black.index()], 1);
}

#[test]
fn a_tile_outside_any_kingdom_scores_nothing() {
    let mut s = empty(2, Reinforcements::English);
    s.hands[0] = [1, 0, 0, 0];
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Black,
            to: at(8, 4),
        },
    );
    assert_eq!(s.scores, vec![[0; 4]; 2]);
}

#[test]
fn blue_tiles_go_on_rivers_and_the_others_never() {
    let mut s = empty(2, Reinforcements::English);
    s.hands[0] = [1, 0, 1, 0];
    // (0, 3) is a river square, (5, 4) land.
    assert!(refused(
        &s,
        S0,
        Action::Tile {
            color: Blue,
            to: at(5, 4)
        }
    ));
    assert!(refused(
        &s,
        S0,
        Action::Tile {
            color: Black,
            to: at(0, 3)
        }
    ));
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Blue,
            to: at(0, 3),
        },
    );
}

#[test]
fn a_leader_stands_next_to_a_temple_off_the_rivers() {
    let mut s = empty(2, Reinforcements::English);
    tile(&mut s, 5, 4, Red);
    assert!(refused(
        &s,
        S0,
        Action::Leader {
            color: Black,
            to: at(8, 4)
        }
    ));
    play(
        &mut s,
        S0,
        Action::Leader {
            color: Black,
            to: at(6, 4),
        },
    );
    // Moving it: judged as if lifted first.
    play(
        &mut s,
        S0,
        Action::Leader {
            color: Black,
            to: at(4, 4),
        },
    );
    assert_eq!(s.leader_at(S0, Black), Some(at(4, 4)));
}

#[test]
fn a_leader_may_not_join_two_kingdoms() {
    let mut s = empty(2, Reinforcements::English);
    tile(&mut s, 5, 4, Red);
    tile(&mut s, 7, 4, Red);
    leader(&mut s, 4, 4, S1, Black);
    leader(&mut s, 8, 4, S1, Blue);
    assert!(refused(
        &s,
        S0,
        Action::Leader {
            color: Red,
            to: at(6, 4)
        }
    ));
}

#[test]
fn a_tile_may_not_join_three_kingdoms() {
    let mut s = empty(3, Reinforcements::English);
    for (x, y, seat) in [(5, 4, S0), (7, 4, S1), (6, 3, S2)] {
        leader(&mut s, x, y, seat, Black);
    }
    s.hands[0] = [1, 0, 0, 0];
    assert!(refused(
        &s,
        S0,
        Action::Tile {
            color: Black,
            to: at(6, 4)
        }
    ));
}

/// Seat 0's priest at (6, 4) with one temple, seat 1's at (8, 4) with two.
fn revolt(reinforcements: Reinforcements) -> State {
    let mut s = empty(2, reinforcements);
    tile(&mut s, 5, 4, Red);
    tile(&mut s, 9, 4, Red);
    tile(&mut s, 8, 5, Red);
    tile(&mut s, 7, 4, Black);
    leader(&mut s, 8, 4, S1, Red);
    s
}

#[test]
fn an_internal_conflict_is_fought_with_adjacent_temples() {
    let mut s = revolt(Reinforcements::Free);
    s.hands[0] = [0, 2, 0, 0];
    s.hands[1] = [0, 1, 0, 0];
    play(
        &mut s,
        S0,
        Action::Leader {
            color: Red,
            to: at(6, 4),
        },
    );
    let Phase::Reinforce(c) = s.phase else {
        panic!("conflit attendu")
    };
    assert_eq!(
        (c.kind, c.attacker, c.defender),
        (ConflictKind::Internal, S0, S1)
    );
    assert_eq!((c.attacker_base, c.defender_base), (1, 2));
    play(&mut s, S0, Action::Reinforce { count: 2 });
    play(&mut s, S1, Action::Reinforce { count: 1 });
    // 3 against 3: the defender wins, scores one red point; the attacker's
    // priest goes home and every reinforcement is discarded.
    assert_eq!(s.leader_at(S0, Red), None);
    assert_eq!(s.leader_at(S1, Red), Some(at(8, 4)));
    assert_eq!(s.scores[1][Red.index()], 1);
    assert_eq!((s.hands[0][Red.index()], s.hands[1][Red.index()]), (0, 0));
}

#[test]
fn english_reinforcements_only_to_get_ahead_or_draw_level() {
    let mut s = revolt(Reinforcements::English);
    s.hands[0] = [0, 3, 0, 0];
    s.hands[1] = [0, 4, 0, 0];
    play(
        &mut s,
        S0,
        Action::Leader {
            color: Red,
            to: at(6, 4),
        },
    );
    let Phase::Reinforce(c) = s.phase else {
        panic!("conflit attendu")
    };
    // 1 against 2: the attacker adds 2 or 3, or nothing.
    assert_eq!(s.reinforce_options(&c), vec![0, 2, 3]);
    play(&mut s, S0, Action::Reinforce { count: 3 });
    let Phase::Reinforce(c) = s.phase else {
        panic!("conflit attendu")
    };
    // 4 against 2: the defender adds exactly 2, or nothing.
    assert_eq!(s.reinforce_options(&c), vec![0, 2]);
}

/// Two kingdoms on row 4, to be joined at (8, 4): seat 0's trader at
/// (5, 4) with three markets, seat 1's at (11, 4) with one.
fn war() -> State {
    let mut s = empty(2, Reinforcements::Free);
    tile(&mut s, 4, 4, Red);
    leader(&mut s, 5, 4, S0, Green);
    tile(&mut s, 6, 4, Green);
    tile(&mut s, 7, 4, Green);
    tile(&mut s, 6, 5, Green);
    tile(&mut s, 9, 4, Green);
    tile(&mut s, 10, 4, Red);
    leader(&mut s, 11, 4, S1, Green);
    tile(&mut s, 12, 4, Red);
    s
}

#[test]
fn an_external_conflict_removes_the_losers_supporters() {
    let mut s = war();
    s.hands[0] = [0, 0, 0, 2];
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Green,
            to: at(8, 4),
        },
    );
    let Phase::Reinforce(c) = s.phase else {
        panic!("conflit attendu")
    };
    // The junction tile counts for neither side.
    assert_eq!(
        (c.kind, c.attacker_base, c.defender_base),
        (ConflictKind::External, 3, 1)
    );
    assert_eq!(s.junction, Some(at(8, 4)));
    play(&mut s, S0, Action::Reinforce { count: 0 });
    // The defender has nothing to add: decided at once. He loses his
    // trader and his market; the winner scores 1 + 1 green.
    assert_eq!(s.leader_at(S1, Green), None);
    assert_eq!(s.cells[usize::from(at(9, 4))], None);
    assert_eq!(s.scores[0][Green.index()], 2);
    assert_eq!(s.junction, None);
    // No point for the junction tile itself.
    assert_eq!(s.scores[1], [0; 4]);
}

#[test]
fn a_war_of_priests_spares_temples_with_a_treasure_or_another_leader() {
    let mut s = empty(2, Reinforcements::Free);
    tile(&mut s, 4, 4, Red);
    tile(&mut s, 4, 5, Red);
    leader(&mut s, 5, 4, S0, Red);
    tile(&mut s, 6, 4, Red);
    tile(&mut s, 7, 4, Red);
    // Seat 1's side: priest at (10, 4); temples at (9, 4) with a
    // treasure, (11, 4) next to its king, (10, 5) plain.
    tile(&mut s, 9, 4, Red);
    s.treasure[usize::from(at(9, 4))] = true;
    leader(&mut s, 10, 4, S1, Red);
    tile(&mut s, 11, 4, Red);
    leader(&mut s, 12, 4, S1, Black);
    tile(&mut s, 10, 5, Red);
    s.hands[0] = [0, 2, 0, 0];
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Red,
            to: at(8, 4),
        },
    );
    let Phase::Reinforce(c) = s.phase else {
        panic!("conflit attendu")
    };
    assert_eq!((c.attacker_base, c.defender_base), (4, 3));
    play(&mut s, S0, Action::Reinforce { count: 0 });
    assert_eq!(s.leader_at(S1, Red), None);
    assert!(
        s.cells[usize::from(at(9, 4))].is_some(),
        "temple au trésor gardé"
    );
    assert!(
        s.cells[usize::from(at(11, 4))].is_some(),
        "temple voisin du roi gardé"
    );
    assert_eq!(s.cells[usize::from(at(10, 5))], None);
    // One point for the priest, one for the single temple removed.
    assert_eq!(s.scores[0][Red.index()], 2);
}

#[test]
fn the_player_orders_several_conflicts_and_a_split_ends_the_rest() {
    let mut s = war();
    // Kings on both sides too.
    tile(&mut s, 4, 3, Red);
    leader(&mut s, 5, 3, S0, Black);
    leader(&mut s, 12, 3, S1, Black);
    s.hands[0] = [0, 0, 0, 2];
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Green,
            to: at(8, 4),
        },
    );
    assert_eq!(s.phase, Phase::ChooseConflict(vec![Black, Green]));
    play(&mut s, S0, Action::Resolve { color: Green });
    play(&mut s, S0, Action::Reinforce { count: 0 });
    // The market at (9, 4) is gone: the kingdoms are apart again, and the
    // kings' conflict does not happen.
    assert_eq!(s.leader_at(S1, Black), Some(at(12, 3)));
    assert_eq!(s.phase, Phase::Action);
}

#[test]
fn a_square_of_four_raises_a_monument_and_it_scores_at_the_end_of_turn() {
    let mut s = empty(2, Reinforcements::English);
    tile(&mut s, 4, 4, Red);
    leader(&mut s, 4, 5, S0, Black);
    leader(&mut s, 4, 3, S0, Blue);
    tile(&mut s, 5, 4, Green);
    tile(&mut s, 6, 4, Green);
    tile(&mut s, 5, 5, Green);
    s.hands[0] = [0, 0, 0, 1];
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Green,
            to: at(6, 5),
        },
    );
    assert_eq!(s.phase, Phase::Monument(vec![at(5, 4)]));
    // Monument 2 is black and green.
    play(
        &mut s,
        S0,
        Action::Monument {
            square: at(5, 4),
            monument: 2,
        },
    );
    assert_eq!(
        s.cells[usize::from(at(6, 5))],
        Some(Piece::Tile {
            color: Green,
            flipped: true
        })
    );
    let before = s.scores[0];
    play(&mut s, S0, Action::EndTurn);
    // The king scores the black monument; the farmer gets nothing.
    assert_eq!(s.scores[0][Black.index()], before[Black.index()] + 1);
    assert_eq!(s.scores[0][Blue.index()], before[Blue.index()]);
}

#[test]
fn a_declined_square_never_gets_a_monument() {
    let mut s = empty(2, Reinforcements::English);
    for (x, y) in [(5, 4), (6, 4), (5, 5)] {
        tile(&mut s, x, y, Black);
    }
    s.hands[0] = [1, 0, 0, 0];
    play(
        &mut s,
        S0,
        Action::Tile {
            color: Black,
            to: at(6, 5),
        },
    );
    play(&mut s, S0, Action::NoMonument);
    assert_eq!(s.declined, vec![at(5, 4)]);
}

#[test]
fn the_trader_takes_all_treasures_but_one_corners_first() {
    let mut s = empty(2, Reinforcements::English);
    // (1, 1) is a corner square; (2, 1) and (1, 2) are plain land.
    for (x, y) in [(1, 1), (2, 1), (1, 2)] {
        tile(&mut s, x, y, Red);
        s.treasure[usize::from(at(x, y))] = true;
    }
    // Another treasure far away keeps the game going.
    for (x, y) in [(10, 10), (5, 9)] {
        tile(&mut s, x, y, Red);
        s.treasure[usize::from(at(x, y))] = true;
    }
    play(
        &mut s,
        S0,
        Action::Leader {
            color: Green,
            to: at(2, 2),
        },
    );
    // The corner must go; the player picks which plain one stays.
    assert_eq!(
        s.phase,
        Phase::Treasure {
            seat: S0,
            choices: vec![at(2, 1), at(1, 2)]
        }
    );
    play(&mut s, S0, Action::LeaveTreasure { at: at(1, 2) });
    assert_eq!(s.treasures[0], 2);
    assert!(!s.treasure[usize::from(at(1, 1))]);
    assert!(s.treasure[usize::from(at(1, 2))]);
}

#[test]
fn a_catastrophe_sends_home_a_leader_without_temple() {
    let mut s = empty(2, Reinforcements::English);
    tile(&mut s, 5, 4, Red);
    leader(&mut s, 6, 4, S1, Blue);
    assert!(refused(&s, S0, Action::Catastrophe { at: at(6, 4) }));
    play(&mut s, S0, Action::Catastrophe { at: at(5, 4) });
    assert_eq!(s.leader_at(S1, Blue), None);
    assert_eq!(s.catastrophes[0], 1);
    assert_eq!(s.cells[usize::from(at(5, 4))], Some(Piece::Catastrophe));
}

#[test]
fn hands_come_back_to_six_at_the_end_of_turn() {
    let mut s = TigreEuphrate::new(&Config::default(), 3, 1);
    let color = tigre_euphrate::COLORS
        .into_iter()
        .find(|c| *c != Blue && s.hands[0][c.index()] > 0)
        .expect("a land tile");
    play(
        &mut s,
        S0,
        Action::Tile {
            color,
            to: at(8, 4),
        },
    );
    play(&mut s, S0, Action::EndTurn);
    assert_eq!(s.hands[0].iter().sum::<u8>(), 6);
    assert_eq!(s.turn, S1);
    assert_eq!(s.actions_left, 2);
}

#[test]
fn the_game_ends_with_two_treasures_left() {
    let mut s = empty(2, Reinforcements::English);
    for (x, y) in [(5, 4), (7, 4)] {
        tile(&mut s, x, y, Red);
        s.treasure[usize::from(at(x, y))] = true;
    }
    play(&mut s, S0, Action::EndTurn);
    assert_eq!(s.end, Some(EndReason::Treasures));
    assert!(TigreEuphrate::outcome(&s).is_some());
}

#[test]
fn the_game_ends_when_the_bag_cannot_refill_a_hand() {
    let mut s = TigreEuphrate::new(&Config::default(), 2, 3);
    s.bag.truncate(2);
    let tiles = s.hands[0];
    play(&mut s, S0, Action::Swap { tiles });
    assert_eq!(s.end, Some(EndReason::Bag));
}

#[test]
fn treasures_fill_the_weakest_colour_and_ties_look_further() {
    assert_eq!(final_points(&[3, 5, 4, 9], 3), [5, 5, 5, 9]);
    let mut s = empty(3, Reinforcements::English);
    s.scores = vec![[10, 10, 12, 30], [10, 10, 11, 11], [11, 11, 11, 11]];
    s.end = Some(EndReason::Treasures);
    s.phase = Phase::Over;
    assert_eq!(
        TigreEuphrate::outcome(&s),
        Some(Outcome::Ranked(vec![1, 2, 0]))
    );
}

#[test]
fn others_points_stay_behind_their_screens() {
    let mut s = TigreEuphrate::new(&Config::default(), 3, 5);
    s.scores[1] = [4, 4, 4, 4];
    let view = serde_json::to_value(TigreEuphrate::view(&s, Viewer::Seat(S0))).unwrap();
    assert_eq!(view["myScores"], serde_json::json!([0, 0, 0, 0]));
    assert!(!view.to_string().contains("[4,4,4,4]"));
    // The AI's memory of them is blurred, its own hand kept.
    let d = TigreEuphrate::determinize(&s, S0, &mut SmallRng::seed_from_u64(1));
    assert_eq!(d.hands[0], s.hands[0]);
    assert_eq!(d.hands[1].iter().sum::<u8>(), 6);
    assert!(d.scores[1].iter().all(|&p| (3..=5).contains(&p)));
}
