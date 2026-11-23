//! [R] 30 dominos : les 15 paires sans 0 ni 6, chacune en double.

use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

pub const DOMINOES: usize = 30;

/// The 15 value pairs `a ≤ b`, in id order: (1,1), (1,2) … (1,5), (2,2) … (5,5).
const PAIRS: [(u8, u8); 15] = {
    let mut pairs = [(0, 0); 15];
    let mut i = 0;
    let mut a = 1;
    while a <= 5 {
        let mut b = a;
        while b <= 5 {
            pairs[i] = (a, b);
            i += 1;
            b += 1;
        }
        a += 1;
    }
    pairs
};

/// The values of a domino: ids 0..15 and 15..30 are the two sets.
pub fn values(id: u8) -> (u8, u8) {
    PAIRS[usize::from(id) % PAIRS.len()]
}

pub fn is_double(id: u8) -> bool {
    let (a, b) = values(id);
    a == b
}

pub struct Deal {
    /// Sorted.
    pub hands: Vec<Vec<u8>>,
    /// The starting square, top domino first.
    pub starters: [u8; 2],
    /// [R] At three, one of the three left is set aside face down.
    pub discarded: Option<u8>,
}

/// [R] 2 players: 14 each; 3: 9 each and one set aside; 4: 7 each. The two
/// left form the starting square; two doubles of the same value are a
/// misdeal, dealt again.
pub fn deal(players: u8, rng: &mut ChaCha8Rng) -> Deal {
    let size = match players {
        2 => 14,
        3 => 9,
        _ => 7,
    };
    loop {
        let mut deck: Vec<u8> = (0..DOMINOES as u8).collect();
        deck.shuffle(rng);
        let mut hands: Vec<Vec<u8>> = deck
            .chunks(size)
            .take(usize::from(players))
            .map(<[u8]>::to_vec)
            .collect();
        hands.iter_mut().for_each(|h| h.sort_unstable());
        let rest = &deck[size * usize::from(players)..];
        let (discarded, starters) = match rest {
            [d, x, y] => (Some(*d), [*x, *y]),
            [x, y] => (None, [*x, *y]),
            _ => unreachable!("two or three dominoes left"),
        };
        let [x, y] = starters;
        if is_double(x) && is_double(y) && values(x) == values(y) {
            continue;
        }
        return Deal {
            hands,
            starters,
            discarded,
        };
    }
}
