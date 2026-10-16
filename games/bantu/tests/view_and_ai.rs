use bantu::testing::position;
use bantu::{Action, Bantu, Config, Dest, Spot, QUIET_ROUNDS};
use ludo_core::{Game, Heuristics, Seat, Viewer};
use rand::rngs::SmallRng;
use rand::SeedableRng;

fn on(cell: u8) -> Spot {
    Spot::On { cell }
}

#[test]
fn view_shows_lengths_protection_and_quiet_rounds() {
    let mut s = position(2, &[(0, 1, on(66)), (1, 2, on(65)), (0, 3, on(69))], 0);
    s.quiet = 5;
    let v = Bantu::view(&s, Viewer::Seat(Seat(1)));
    // Seat 0 to move: pieces 1 (in a row of two: 1 + 2) and 3 (alone); 2 and
    // 4 are in.
    assert_eq!(v.lengths, vec![3, 0, 3, 0]);
    assert_eq!(v.protected, vec![65, 66]);
    // 60 turns allowed at 2 players, 5 spent: 27.5 rounds, 28 started.
    assert_eq!(v.quiet_left, QUIET_ROUNDS - 2);
    assert!(v.legal.is_empty(), "not seat 1's turn");
    let mine = Bantu::view(&s, Viewer::Seat(Seat(0)));
    assert!(mine.legal.contains(&Action::Move {
        piece: 3,
        to: Dest::Cell(64)
    }));
}

#[test]
fn rollout_terminates() {
    let mut rng = SmallRng::seed_from_u64(3);
    for (k, short) in [(0, false), (1, true)] {
        for players in 2..=4u8 {
            for game in 0..20u64 {
                let mut s = Bantu::new(&Config { short }, players, game + k);
                let mut plies = 0;
                while let Some(seat) = s.turn {
                    let a = Bantu::rollout_action(&s, seat, &mut rng);
                    Bantu::apply(&mut s, seat, &a).unwrap();
                    plies += 1;
                    assert!(plies < 20_000, "endless game");
                }
                assert!(Bantu::outcome(&s).is_some());
            }
        }
    }
}

#[test]
fn rollout_prefers_a_capture() {
    // Piece 1 on 69 can take the lone piece on 62.
    let s = position(2, &[(0, 1, on(69)), (0, 2, Spot::Home), (1, 3, on(62))], 0);
    let mut rng = SmallRng::seed_from_u64(1);
    for _ in 0..20 {
        assert_eq!(
            Bantu::rollout_action(&s, Seat(0), &mut rng),
            Action::Move {
                piece: 1,
                to: Dest::Cell(62)
            }
        );
    }
}

#[test]
fn evaluate_cuts_long_simulations() {
    let root = Bantu::new(&Config::default(), 2, 0);
    assert!(Bantu::evaluate(&root, &root).is_none());
    let mut late = root.clone();
    late.moves = root.moves + 1000;
    let rewards = Bantu::evaluate(&root, &late).expect("cut");
    assert_eq!(rewards.len(), 2);
}

#[test]
fn board_json_is_up_to_date() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../web/src/games/bantu/board.json"
    );
    let fresh = serde_json::to_string_pretty(&bantu::board::board_json()).unwrap() + "\n";
    if std::env::var_os("UPDATE_BOARD_JSON").is_some() {
        std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        std::fs::write(path, &fresh).unwrap();
    }
    let on_disk = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        on_disk == fresh,
        "web/src/games/bantu/board.json is stale: rerun with UPDATE_BOARD_JSON=1"
    );
}
