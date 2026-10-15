// Mirrors games/cards/src/deck.rs (Card, Face).

export type Rank =
  | "Two"
  | "Three"
  | "Four"
  | "Five"
  | "Six"
  | "Seven"
  | "Eight"
  | "Nine"
  | "Ten"
  | "Jack"
  | "Queen"
  | "King"
  | "Ace";

export type Suit = "Spades" | "Hearts" | "Diamonds" | "Clubs";

export type Card = { kind: "Natural"; rank: Rank; suit: Suit } | { kind: "Joker" };

/** What a card shows when suits do not matter. */
export type Face = { kind: "Natural"; rank: Rank } | { kind: "Joker" };
