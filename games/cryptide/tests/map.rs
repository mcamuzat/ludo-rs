use cryptide::{distance, random_layout, within, Animal, Board, Cell, Layout, Terrain, CELLS};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn cell(row: u8, col: u8) -> Cell {
    Cell::new(row, col).expect("on the map")
}

fn all_cells() -> impl Iterator<Item = Cell> {
    (0..CELLS).map(Cell)
}

#[test]
fn corners_and_edges_have_the_right_neighbours() {
    assert_eq!(within(cell(0, 0), 1).len(), 3);
    assert_eq!(within(cell(0, 11), 1).len(), 4);
    assert_eq!(within(cell(8, 0), 1).len(), 4);
    assert_eq!(within(cell(8, 11), 1).len(), 3);
    assert_eq!(within(cell(4, 5), 1).len(), 7);
    // [B] dcounter.py: space 2 touches 1, 3, 13, 14 and 15.
    let around: Vec<Cell> = within(cell(0, 1), 1).iter().collect();
    let expected = vec![
        cell(0, 0),
        cell(0, 1),
        cell(0, 2),
        cell(1, 0),
        cell(1, 1),
        cell(1, 2),
    ];
    assert_eq!(around, expected);
}

#[test]
fn distance_is_symmetric_and_bounded() {
    for a in all_cells() {
        for b in all_cells() {
            assert_eq!(distance(a, b), distance(b, a));
            assert_eq!(distance(a, b) == 0, a == b);
            assert!(distance(a, b) <= 14);
        }
    }
}

#[test]
fn within_grows_with_distance() {
    for c in all_cells() {
        for n in 0..3 {
            let (small, big) = (within(c, n), within(c, n + 1));
            assert_eq!(small & big, small);
            assert!(big.len() > small.len());
        }
        assert_eq!(within(c, 0).len(), 1);
    }
}

/// [B] gameboard.py, sections 1 to 6: terrain rows (W = Lake), then the
/// territories (B bear, C cougar, . none).
const BOT: [([&str; 3], [&str; 3]); 6] = [
    (
        ["WWWWFF", "SSWDFF", "SSDDDF"],
        ["......", "......", "...BBB"],
    ),
    (
        ["SFFFFF", "SSFDDD", "SMMMMD"],
        ["CCC...", "......", "......"],
    ),
    (
        ["SSFFFW", "SSFMWW", "MMMMWW"],
        ["......", "CC....", "C....."],
    ),
    (
        ["DDMMMM", "DDMWWW", "DDDFFF"],
        ["......", ".....C", ".....C"],
    ),
    (
        ["SSSMMM", "SDDWMM", "DDWWWW"],
        ["......", ".....B", "....BB"],
    ),
    (
        ["DDSSSF", "MMSSFF", "MWWWWF"],
        ["B.....", "B.....", "......"],
    ),
];

fn terrain_letter(t: Terrain) -> char {
    match t {
        Terrain::Lake => 'W',
        Terrain::Mountain => 'M',
        Terrain::Forest => 'F',
        Terrain::Swamp => 'S',
        Terrain::Desert => 'D',
    }
}

fn animal_letter(a: Option<Animal>) -> char {
    match a {
        Some(Animal::Bear) => 'B',
        Some(Animal::Cougar) => 'C',
        None => '.',
    }
}

#[test]
fn sections_match_cryptid_bot() {
    for (section, (terrains, animals)) in BOT.iter().enumerate() {
        // The section alone at position 0, the others after it.
        let mut order = [0, 1, 2, 3, 4, 5];
        order.swap(0, section);
        let board = Board::new(
            &Layout {
                order,
                flipped: [false; 6],
            },
            &[],
        );
        for row in 0..3u8 {
            for col in 0..6u8 {
                let sq = &board.squares[usize::from(cell(row, col).0)];
                let r = usize::from(row);
                let c = usize::from(col);
                assert_eq!(
                    terrain_letter(sq.terrain),
                    terrains[r].as_bytes()[c] as char,
                    "section {} ({row},{col})",
                    section + 1
                );
                assert_eq!(
                    animal_letter(sq.animal),
                    animals[r].as_bytes()[c] as char,
                    "section {} ({row},{col})",
                    section + 1
                );
            }
        }
    }
}

#[test]
fn sections_take_their_positions_in_reading_order() {
    // Position 1 is the right half of the top band, position 4 the left
    // half of the bottom band.
    let board = Board::new(
        &Layout {
            order: [2, 0, 3, 4, 5, 1],
            flipped: [false; 6],
        },
        &[],
    );
    assert_eq!(
        board.squares[usize::from(cell(0, 6).0)].terrain,
        Terrain::Lake
    ); // section 1 (0,0)
    assert_eq!(
        board.squares[usize::from(cell(6, 0).0)].terrain,
        Terrain::Desert
    ); // section 6 (0,0)
    assert_eq!(
        board.squares[usize::from(cell(6, 0).0)].animal,
        Some(Animal::Bear)
    );
}

#[test]
fn a_flipped_section_is_rotated() {
    let plain = Board::new(
        &Layout {
            order: [0, 1, 2, 3, 4, 5],
            flipped: [false; 6],
        },
        &[],
    );
    let mut flipped = [false; 6];
    flipped[0] = true;
    let turned = Board::new(
        &Layout {
            order: [0, 1, 2, 3, 4, 5],
            flipped,
        },
        &[],
    );
    for row in 0..3u8 {
        for col in 0..6u8 {
            let a = &plain.squares[usize::from(cell(row, col).0)];
            let b = &turned.squares[usize::from(cell(2 - row, 5 - col).0)];
            assert_eq!(a.terrain, b.terrain);
            assert_eq!(a.animal, b.animal);
        }
    }
}

#[test]
fn the_layout_of_a_seed_is_pinned() {
    let layout = random_layout(&mut ChaCha8Rng::seed_from_u64(7));
    let mut sorted = layout.order;
    sorted.sort();
    assert_eq!(sorted, [0, 1, 2, 3, 4, 5]);
    assert_eq!(
        layout,
        Layout {
            order: [4, 3, 2, 0, 5, 1],
            flipped: [false, true, true, true, true, true]
        }
    );
}

/// The advanced puzzle card the user photographed: sections 3, 5 / 4, 1 /
/// 2, 6, the 3 and the 4 upright. Its first and last rows, read on the
/// photo.
#[test]
fn photo_card_layout() {
    let board = Board::new(
        &Layout {
            order: [2, 4, 3, 0, 1, 5],
            flipped: [false, true, false, true, true, true],
        },
        &[],
    );
    let row = |r: u8| -> String {
        (0..12)
            .map(|c| terrain_letter(board.squares[usize::from(cell(r, c).0)].terrain))
            .collect()
    };
    assert_eq!(row(0), "SSFFFWWWWWDD");
    assert_eq!(row(3), "DDMMMMFDDDSS");
    assert_eq!(row(8), "FFFFFSFSSSDD");
}
