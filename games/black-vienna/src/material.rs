//! The cards: 27 suspects, 36 investigation cards, and the deal.

use ludo_core::Seat;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// [C][F] The suspects' letters, in the order of the investigation sheet.
const LETTERS: [char; 27] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'Ö',
];

/// [C] The 36 investigation cards, three suspects each; every suspect is on
/// exactly four of them.
pub const INVESTIGATIONS: [&str; 36] = [
    "ACL", "AGM", "AOS", "APQ", "BCY", "BHV", "BLM", "BQT", "CFI", "CSX", "DHR", "DJZ", "DLS",
    "DVY", "EGW", "ENQ", "EÖR", "EUV", "FÖY", "FRX", "FSZ", "GKO", "GPX", "HNÖ", "HUZ", "IÖW",
    "IPR", "ITZ", "JMO", "JQX", "JTY", "KMU", "KNT", "KPW", "LNV", "OUW",
];

/// [R] Investigation cards per pile.
pub const PILE: usize = 12;

/// A suspect, written as its letter (A to Z, then Ö).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Suspect(u8);

impl Suspect {
    pub const COUNT: usize = 27;

    /// 0 for A, 25 for Z, 26 for Ö.
    pub fn new(index: u8) -> Option<Self> {
        (usize::from(index) < Self::COUNT).then_some(Self(index))
    }

    pub fn from_letter(letter: char) -> Option<Self> {
        LETTERS
            .iter()
            .position(|&l| l == letter)
            .map(|i| Self(i as u8))
    }

    pub fn letter(self) -> char {
        LETTERS[self.index()]
    }

    pub fn index(self) -> usize {
        usize::from(self.0)
    }

    pub fn all() -> impl Iterator<Item = Suspect> {
        (0..Self::COUNT as u8).map(Self)
    }
}

impl TryFrom<String> for Suspect {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let mut chars = text.chars();
        match (chars.next(), chars.next()) {
            (Some(letter), None) => {
                Self::from_letter(letter).ok_or_else(|| format!("personnage inconnu : {text}"))
            }
            _ => Err(format!("personnage inconnu : {text}")),
        }
    }
}

impl From<Suspect> for String {
    fn from(s: Suspect) -> String {
        s.letter().to_string()
    }
}

/// An investigation card, by its place in `INVESTIGATIONS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct CardId(u8);

impl CardId {
    pub fn new(id: u8) -> Option<Self> {
        (usize::from(id) < INVESTIGATIONS.len()).then_some(Self(id))
    }

    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

impl TryFrom<u8> for CardId {
    type Error = String;

    fn try_from(id: u8) -> Result<Self, String> {
        Self::new(id).ok_or_else(|| format!("carte Enquête inconnue : {id}"))
    }
}

impl From<CardId> for u8 {
    fn from(c: CardId) -> u8 {
        c.0
    }
}

/// The three suspects of an investigation card.
pub fn suspects(card: CardId) -> [Suspect; 3] {
    let mut out = [Suspect(0); 3];
    for (slot, letter) in out.iter_mut().zip(INVESTIGATIONS[card.index()].chars()) {
        *slot = Suspect::from_letter(letter).unwrap_or(Suspect(0));
    }
    out
}

/// [R] Suspect cards per player: 8 at three, 6 at four, 4 at six; at five,
/// the dealer gets 4 and the others 5.
pub fn hand_sizes(players: u8, dealer: Seat) -> Vec<usize> {
    let n = usize::from(players);
    let dealt = Suspect::COUNT - 3;
    (0..n)
        .map(|i| {
            // Dealt one by one from the dealer's left: the first seats from
            // there take the remainder.
            let from_left = (i + n - usize::from(dealer.0) - 1) % n;
            dealt / n + usize::from(from_left < dealt % n)
        })
        .collect()
}

/// The cards of a new game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deal {
    /// Black Vienna, sorted.
    pub trio: [Suspect; 3],
    pub hands: Vec<Vec<Suspect>>,
    /// Face up; the top of each pile is its last card.
    pub piles: [Vec<CardId>; 3],
}

/// [R] The investigation cards in three piles of 12; the first three
/// suspects are Black Vienna, the others dealt one by one from the dealer's
/// left.
pub fn deal(players: u8, dealer: Seat, seed: u64) -> Deal {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut cards: Vec<CardId> = (0..INVESTIGATIONS.len() as u8).map(CardId).collect();
    cards.shuffle(&mut rng);
    let piles = [
        cards[..PILE].to_vec(),
        cards[PILE..2 * PILE].to_vec(),
        cards[2 * PILE..].to_vec(),
    ];
    let mut people: Vec<Suspect> = Suspect::all().collect();
    people.shuffle(&mut rng);
    let mut trio = [people[0], people[1], people[2]];
    trio.sort();
    let n = usize::from(players.max(1));
    let mut hands = vec![Vec::new(); n];
    for (k, &s) in people[3..].iter().enumerate() {
        hands[(usize::from(dealer.0) + 1 + k) % n].push(s);
    }
    for hand in &mut hands {
        hand.sort();
    }
    Deal { trio, hands, piles }
}
