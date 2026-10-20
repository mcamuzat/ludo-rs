use cryptide::{
    generate, habitat, hints, Animal, Board, Cell, Clue, Color, Hint, HintTable, Kind, Layout,
    Mode, Structure, Terrain,
};

fn cell(row: u8, col: u8) -> Cell {
    Cell::new(row, col).expect("on the map")
}

fn board() -> Board {
    Board::new(
        &Layout {
            order: [0, 1, 2, 3, 4, 5],
            flipped: [false; 6],
        },
        &[
            Structure {
                kind: Kind::Stone,
                color: Color::White,
                cell: cell(4, 4),
            },
            Structure {
                kind: Kind::Shack,
                color: Color::Blue,
                cell: cell(0, 8),
            },
        ],
    )
}

fn hint(clue: Clue) -> Hint {
    Hint {
        clue,
        negated: false,
    }
}

#[test]
fn hint_counts_by_mode() {
    let normal = hints(Mode::Normal);
    assert_eq!(normal.len(), 23);
    assert!(normal
        .iter()
        .all(|h| !h.negated && h.clue != Clue::NearColor(Color::Black)));
    let advanced = hints(Mode::Advanced);
    assert_eq!(advanced.len(), 48);
    assert_eq!(advanced.iter().filter(|h| h.negated).count(), 24);
}

#[test]
fn terrains_hint_is_the_two_terrains() {
    let m = hint(Clue::Terrains(Terrain::Forest, Terrain::Swamp)).cells(&board());
    assert!(!m.contains(cell(0, 0))); // lake
    assert!(m.contains(cell(1, 0))); // swamp
    assert!(m.contains(cell(0, 4))); // forest
    assert!(!m.contains(cell(1, 3))); // desert
}

#[test]
fn near_terrain_includes_the_terrain_itself() {
    let b = board();
    let m = hint(Clue::NearTerrain(Terrain::Desert)).cells(&b);
    for c in (0..108).map(Cell) {
        if b.square(c).terrain == Terrain::Desert {
            assert!(m.contains(c));
        }
    }
    assert!(!m.contains(cell(0, 0))); // lake, beside lake and swamp
    assert!(m.contains(cell(0, 3))); // lake, beside the desert at (1, 3)
}

#[test]
fn near_animals_counts_both_territories() {
    let m = hint(Clue::NearAnimals).cells(&board());
    assert!(m.contains(cell(2, 3))); // bear
    assert!(m.contains(cell(1, 3))); // beside it
    assert!(m.contains(cell(4, 0))); // cougar
    assert!(!m.contains(cell(0, 0)));
    let bear = hint(Clue::NearAnimal(Animal::Bear)).cells(&board());
    assert!(!bear.contains(cell(4, 0)));
}

#[test]
fn near_kind_ignores_color() {
    let stone = hint(Clue::NearKind(Kind::Stone)).cells(&board());
    assert!(stone.contains(cell(4, 4)));
    assert!(stone.contains(cell(6, 4)));
    assert!(!stone.contains(cell(7, 4)));
    assert!(!stone.contains(cell(0, 8)));
}

#[test]
fn near_color_ignores_kind() {
    let blue = hint(Clue::NearColor(Color::Blue)).cells(&board());
    assert!(blue.contains(cell(0, 8)));
    assert!(blue.contains(cell(3, 8)));
    assert!(!blue.contains(cell(4, 8)));
    assert!(!blue.contains(cell(4, 4)));
    let black = hint(Clue::NearColor(Color::Black)).cells(&board());
    assert!(black.is_empty());
}

#[test]
fn negation_is_the_complement() {
    let b = board();
    for h in hints(Mode::Advanced).into_iter().filter(|h| !h.negated) {
        let not = Hint { negated: true, ..h };
        assert_eq!(not.cells(&b), !h.cells(&b));
    }
}

/// Three hints of `table` naming one space, each of them needed.
fn some_puzzle(table: &HintTable) -> Vec<usize> {
    let n = table.hints.len();
    for a in 0..n {
        for b in a + 1..n {
            for c in b + 1..n {
                if habitat(table, &[a, b, c]).is_some() {
                    return vec![a, b, c];
                }
            }
        }
    }
    panic!("no puzzle on the test map");
}

#[test]
fn habitat_refuses_bad_puzzles() {
    let table = HintTable::new(&board(), Mode::Advanced);
    let good = some_puzzle(&table);
    let home = habitat(&table, &good).expect("a puzzle");
    let all = good
        .iter()
        .fold(cryptide::Mask::full(), |m, &i| m & table.masks[i]);
    assert_eq!(all.single(), Some(home));
    // A hint twice.
    assert_eq!(habitat(&table, &[good[0], good[0], good[1], good[2]]), None);
    // Two of them name more than one space.
    assert_eq!(habitat(&table, &good[..2]), None);
    // A fourth hint containing the habitat is not needed.
    let extra = (0..table.hints.len())
        .find(|&i| !good.contains(&i) && table.masks[i].contains(home))
        .expect("a hint around the habitat");
    assert_eq!(habitat(&table, &[good[0], good[1], good[2], extra]), None);
    // Three "two terrains" hints.
    let terrains: Vec<usize> = (0..table.hints.len())
        .filter(|&i| matches!(table.hints[i].clue, Clue::Terrains(..)))
        .collect();
    for a in &terrains {
        for b in &terrains {
            for c in &terrains {
                assert_eq!(habitat(&table, &[*a, *b, *c]), None);
            }
        }
    }
}

#[test]
fn generated_puzzles_hold() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3..=5u8 {
            for seed in 0..200 {
                let p = generate(mode, players, seed);
                let board = Board::new(&p.layout, &p.structures);
                let table = HintTable::new(&board, mode);
                let chosen: Vec<usize> = p
                    .hints
                    .iter()
                    .map(|h| {
                        table
                            .hints
                            .iter()
                            .position(|x| x == h)
                            .expect("a hint of the mode")
                    })
                    .collect();
                assert_eq!(chosen.len(), usize::from(players));
                assert_eq!(
                    habitat(&table, &chosen),
                    Some(p.habitat),
                    "{mode:?} {players} {seed}"
                );
                let expected = if mode == Mode::Normal { 6 } else { 8 };
                assert_eq!(p.structures.len(), expected);
            }
        }
    }
}

#[test]
fn the_puzzle_of_a_seed_is_pinned() {
    let p = generate(Mode::Normal, 4, 7);
    assert_eq!(p.habitat, Cell(78));
    assert_eq!(
        p.hints,
        vec![
            hint(Clue::Terrains(Terrain::Desert, Terrain::Swamp)),
            hint(Clue::NearTerrain(Terrain::Forest)),
            hint(Clue::NearTerrain(Terrain::Swamp)),
            hint(Clue::Terrains(Terrain::Desert, Terrain::Mountain)),
        ]
    );
}
