//! Imposed walls, for tests and tools (puzzles, reproductions).

use crate::Tile;

/// A first-hand wall in draw order. `hands[0]` is the dealer (seat 0); each
/// hand is 13 space-separated tiles; `draws` are the next draws in turn
/// order. The rest (and the 14-tile dead wall) is filled with unused ids.
pub fn wall(hands: [&str; 4], draws: &[&str]) -> Vec<u8> {
    let mut used = [false; 136];
    let mut take = |text: &str| -> u8 {
        let tile = Tile::parse(text).unwrap_or_else(|| panic!("tuile {text}"));
        let base = tile.kind() * 4;
        let id = (base..base + 4)
            .find(|&id| !used[usize::from(id)] && tile.matches_id(id))
            .unwrap_or_else(|| panic!("plus de {text} disponible"));
        used[usize::from(id)] = true;
        id
    };
    let hands: Vec<Vec<u8>> = hands
        .iter()
        .map(|h| {
            let ids: Vec<u8> = h.split_whitespace().map(&mut take).collect();
            assert_eq!(ids.len(), 13, "{h}");
            ids
        })
        .collect();
    let mut order = Vec::with_capacity(136);
    for block in 0..3 {
        for hand in &hands {
            order.extend_from_slice(&hand[block * 4..block * 4 + 4]);
        }
    }
    for hand in &hands {
        order.push(hand[12]);
    }
    for d in draws {
        order.push(take(d));
    }
    order.extend((0..136u8).filter(|&id| !used[usize::from(id)]));
    order
}
