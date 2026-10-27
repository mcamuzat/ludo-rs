// Mirrors games/cards/src/hattrick (View, Action).

export type Color = "Red" | "Blue" | "Green";

export interface HattrickCard {
  color: Color;
  /** 1 to 20. */
  value: number;
}

export type HattrickAction = { Play: { card: HattrickCard } } | { Pass: { card: HattrickCard } };

/** A card on the table: who played it. */
export type Played = [number, HattrickCard];

export interface Pile {
  color: Color;
  cards: Played[];
}

export interface TakenPile {
  color: Color;
  cards: Played[];
  winner: number;
}

export interface TakenTrick {
  moves: [number, HattrickAction][];
  piles: TakenPile[];
  leader: number;
}

export interface RoundLine {
  /** Cards won, by colour: red, blue, green. */
  won: [number, number, number];
  passed: number;
  points: number;
  total: number;
}

export interface HattrickView {
  me: number | null;
  /** By colour, highest first. */
  hand: HattrickCard[];
  handCounts: number[];
  piles: Pile[];
  passes: Played[];
  lastTrick: TakenTrick | null;
  won: [number, number, number][];
  passed: number[];
  scores: number[];
  /** From 1. */
  round: number;
  rounds: number;
  dealer: number;
  turn: number | null;
  lastRound: RoundLine[] | null;
  legal: HattrickAction[];
}
