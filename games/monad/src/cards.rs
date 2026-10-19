//! The Monad deck: six colours, five ranks of cards, and the Monads above them.

use serde::{Deserialize, Serialize};

/// [M] Three warm colours, then three cool ones: the reference order for
/// display and sorting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Navy,
    Sky,
    Green,
}

impl Color {
    pub const ALL: [Color; 6] = [
        Color::Red,
        Color::Orange,
        Color::Yellow,
        Color::Navy,
        Color::Sky,
        Color::Green,
    ];

    /// [M] Red, orange and yellow are warm; navy, sky and green cool.
    pub fn warm(self) -> bool {
        matches!(self, Color::Red | Color::Orange | Color::Yellow)
    }

    /// [M] The bonus combinations: red/navy, orange/sky, yellow/green.
    pub fn bonus_partner(self) -> Color {
        match self {
            Color::Red => Color::Navy,
            Color::Orange => Color::Sky,
            Color::Yellow => Color::Green,
            Color::Navy => Color::Red,
            Color::Sky => Color::Orange,
            Color::Green => Color::Yellow,
        }
    }
}

/// [M] The sets of cards, lowest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    Unit,
    Bi,
    Tri,
    Quad,
    Quint,
}

impl Rank {
    pub const ALL: [Rank; 5] = [Rank::Unit, Rank::Bi, Rank::Tri, Rank::Quad, Rank::Quint];

    /// [M] The point value printed in the centre of the card.
    pub fn value(self) -> u32 {
        match self {
            Rank::Unit => 1,
            Rank::Bi => 3,
            Rank::Tri => 7,
            Rank::Quad => 16,
            Rank::Quint => 36,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// What two cards of this rank trade for.
    pub fn next(self) -> Target {
        match self {
            Rank::Unit => Target::Card(Rank::Bi),
            Rank::Bi => Target::Card(Rank::Tri),
            Rank::Tri => Target::Card(Rank::Quad),
            Rank::Quad => Target::Card(Rank::Quint),
            Rank::Quint => Target::Monad,
        }
    }

    pub fn below(self) -> Option<Rank> {
        self.index().checked_sub(1).map(|i| Rank::ALL[i])
    }

    /// The table column of this rank (Bi = 0 … Quint = 3); Units have none.
    pub fn column(self) -> Option<usize> {
        self.index().checked_sub(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub color: Color,
}

/// What a trade, a buy or a leap brings: a card from a column, or a Monad.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    Card(Rank),
    Monad,
}

impl Target {
    /// [M] A Monad is bought with 80 points or more.
    pub fn value(self) -> u32 {
        match self {
            Target::Card(rank) => rank.value(),
            Target::Monad => 80,
        }
    }

    /// The rank just below: the set two cards of which trade for this.
    pub fn below(self) -> Rank {
        match self {
            Target::Card(rank) => rank.below().unwrap_or(Rank::Unit),
            Target::Monad => Rank::Quint,
        }
    }

    /// Whether `rank` is strictly lower than this target.
    pub fn above(self, rank: Rank) -> bool {
        match self {
            Target::Card(r) => rank < r,
            Target::Monad => true,
        }
    }
}
