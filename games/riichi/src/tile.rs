use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A tile as players see it: its kind (0..34 = 1-9m, 1-9p, 1-9s, then
/// E S W N P F C) and whether it is a red five.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tile {
    kind: u8,
    red: bool,
}

const SUITS: [char; 3] = ['m', 'p', 's'];
const HONORS: [&str; 7] = ["E", "S", "W", "N", "P", "F", "C"];
/// riichienv's ids for the red fives (5m, 5p, 5s).
const RED_IDS: [u8; 3] = [16, 52, 88];

impl Tile {
    /// `red` is only meaningful for a five of a suit.
    pub fn new(kind: u8, red: bool) -> Self {
        assert!(kind < 34, "tile kind {kind}");
        let is_five = kind < 27 && kind % 9 == 4;
        Self {
            kind,
            red: red && is_five,
        }
    }

    /// From riichienv's tile id (0..136, four copies per kind).
    pub fn from_id(id: u8) -> Self {
        Self {
            kind: id / 4,
            red: RED_IDS.contains(&id),
        }
    }

    pub fn kind(self) -> u8 {
        self.kind
    }

    pub fn is_red(self) -> bool {
        self.red
    }

    pub fn is_honor(self) -> bool {
        self.kind >= 27
    }

    pub fn is_terminal(self) -> bool {
        self.kind < 27 && matches!(self.kind % 9, 0 | 8)
    }

    /// Whether engine tile `id` is this tile (same kind, same redness).
    pub fn matches_id(self, id: u8) -> bool {
        Self::from_id(id) == self
    }

    pub fn notation(self) -> String {
        if self.is_honor() {
            return HONORS[usize::from(self.kind - 27)].to_owned();
        }
        let number = self.kind % 9 + 1;
        let suit = SUITS[usize::from(self.kind / 9)];
        if self.red {
            format!("{number}{suit}r")
        } else {
            format!("{number}{suit}")
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        if let Some(i) = HONORS.iter().position(|&h| h == text) {
            return Some(Self {
                kind: 27 + i as u8,
                red: false,
            });
        }
        let mut chars = text.chars();
        let number = chars.next()?.to_digit(10)? as u8;
        let suit_char = chars.next()?;
        let suit = SUITS.iter().position(|&s| s == suit_char)? as u8;
        let red = match chars.next() {
            None => false,
            Some('r') if number == 5 => true,
            Some(_) => return None,
        };
        if chars.next().is_some() || !(1..=9).contains(&number) {
            return None;
        }
        Some(Self {
            kind: suit * 9 + number - 1,
            red,
        })
    }
}

impl Serialize for Tile {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.notation())
    }
}

impl<'de> Deserialize<'de> for Tile {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Tile::parse(&text)
            .ok_or_else(|| serde::de::Error::custom(format!("tuile inconnue : {text}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tile_round_trips_through_its_notation() {
        for kind in 0..34 {
            let t = Tile::new(kind, false);
            assert_eq!(Tile::parse(&t.notation()), Some(t), "{}", t.notation());
        }
        for text in ["5mr", "5pr", "5sr"] {
            let t = Tile::parse(text).unwrap();
            assert!(t.is_red());
            assert_eq!(t.notation(), text);
        }
    }

    #[test]
    fn engine_ids_map_to_tiles() {
        assert_eq!(Tile::from_id(16).notation(), "5mr");
        assert_eq!(Tile::from_id(17).notation(), "5m");
        assert_eq!(Tile::from_id(0).notation(), "1m");
        assert_eq!(Tile::from_id(135).notation(), "C");
        assert_eq!(Tile::from_id(108).notation(), "E");
    }

    #[test]
    fn bad_notations_are_refused() {
        for text in ["", "0m", "10m", "5x", "5zr", "Er", "6mr", "m5"] {
            assert_eq!(Tile::parse(text), None, "{text}");
        }
    }

    #[test]
    fn categories() {
        assert!(Tile::parse("E").unwrap().is_honor());
        assert!(Tile::parse("9s").unwrap().is_terminal());
        assert!(!Tile::parse("5p").unwrap().is_terminal());
    }

    #[test]
    fn serde_uses_the_notation() {
        assert_eq!(
            serde_json::to_string(&Tile::parse("5pr").unwrap()).unwrap(),
            "\"5pr\""
        );
        let back: Tile = serde_json::from_str("\"N\"").unwrap();
        assert_eq!(back.notation(), "N");
        assert!(serde_json::from_str::<Tile>("\"0m\"").is_err());
    }
}
