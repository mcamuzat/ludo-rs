use serde::{Deserialize, Serialize};

/// Coin values in cents, smallest first ([C]: 1, 2, 3, 5, 10, 20 and 25¢).
pub const COINS: [u8; 7] = [1, 2, 3, 5, 10, 20, 25];

fn slot(cents: u8) -> Option<usize> {
    COINS.iter().position(|&c| c == cents)
}

/// A handful of coins (a player's purse or the pot): a count per value.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Purse([u8; COINS.len()]);

/// The coins of one value, for display.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    pub cents: u8,
    pub count: u8,
}

impl Purse {
    /// Coins worth `cents` (0 for a value that is not a coin).
    pub fn count(&self, cents: u8) -> u8 {
        slot(cents).map_or(0, |i| self.0[i])
    }

    /// Panics on a value that is not a coin: callers only add coins they hold.
    pub fn add(&mut self, cents: u8) {
        let i = slot(cents).unwrap_or_else(|| panic!("{cents}¢ n'est pas une pièce"));
        self.0[i] += 1;
    }

    /// Removes one coin worth `cents`; false (and no change) if there is none.
    pub fn remove(&mut self, cents: u8) -> bool {
        match slot(cents) {
            Some(i) if self.0[i] > 0 => {
                self.0[i] -= 1;
                true
            }
            _ => false,
        }
    }

    pub fn total(&self) -> u32 {
        COINS
            .iter()
            .zip(self.0)
            .map(|(&c, n)| u32::from(c) * u32::from(n))
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&n| n == 0)
    }

    /// The values held, smallest first.
    pub fn stacks(&self) -> Vec<Stack> {
        COINS
            .iter()
            .zip(self.0)
            .filter(|&(_, count)| count > 0)
            .map(|(&cents, count)| Stack { cents, count })
            .collect()
    }

    /// [J] Every distinct assortment of these coins worth the most that stays
    /// strictly below `below` cents, each listed largest coin first. A single
    /// empty assortment when nothing can be taken.
    pub fn max_takes(&self, below: u8) -> Vec<Vec<u8>> {
        let mut best = 0;
        let mut takes = Vec::new();
        self.collect_takes(
            COINS.len(),
            u32::from(below),
            0,
            &mut Vec::new(),
            &mut best,
            &mut takes,
        );
        takes
    }

    /// Chooses how many coins of each value below slot `upto` to take,
    /// largest value first, keeping the assortments of the best sum. Each
    /// count vector is visited once, so no assortment appears twice.
    fn collect_takes(
        &self,
        upto: usize,
        below: u32,
        sum: u32,
        current: &mut Vec<u8>,
        best: &mut u32,
        takes: &mut Vec<Vec<u8>>,
    ) {
        let Some(i) = upto.checked_sub(1) else {
            if sum > *best {
                *best = sum;
                takes.clear();
            }
            if sum == *best {
                takes.push(current.clone());
            }
            return;
        };
        let cents = u32::from(COINS[i]);
        let mut taken = 0;
        let mut sum = sum;
        loop {
            self.collect_takes(i, below, sum, current, best, takes);
            if taken == self.0[i] || sum + cents >= below {
                break;
            }
            current.push(COINS[i]);
            taken += 1;
            sum += cents;
        }
        current.truncate(current.len() - usize::from(taken));
    }
}

/// The four characters of the boxed edition, each a starting purse [C].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Character {
    /// The example hand of [J].
    #[default]
    Cody,
    Darlene,
    Sugar,
    Taylor,
}

impl Character {
    pub fn purse(self) -> Purse {
        let coins: &[(u8, u8)] = match self {
            Character::Cody => &[(1, 4), (5, 3), (10, 2), (25, 1)],
            Character::Darlene => &[(1, 3), (3, 3), (10, 2), (20, 1)],
            Character::Sugar => &[(1, 2), (2, 2), (5, 2), (10, 1)],
            Character::Taylor => &[(1, 3), (5, 2), (10, 1)],
        };
        let mut purse = Purse::default();
        for &(cents, count) in coins {
            for _ in 0..count {
                purse.add(cents);
            }
        }
        purse
    }
}
