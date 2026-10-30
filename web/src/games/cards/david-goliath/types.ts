// Mirrors games/cards/src/david_goliath (View, Action).

export type DgColor = "Red" | "Yellow" | "Green" | "Blue" | "Violet";

export interface DgCard {
  color: DgColor;
  /** 1 to 18. */
  value: number;
}

export type DgAction = { Play: { card: DgCard } };

/** A card on the table: who played it. */
export type Played = [number, DgCard];

export interface TakenTrick {
  cards: Played[];
  /** Took the rest, leads next. */
  highest: number;
  /** Took the highest card. */
  lowest: number;
}

export interface RoundLine {
  points: number;
  total: number;
}

export interface DgView {
  me: number | null;
  /** By colour, highest first. */
  hand: DgCard[];
  handCounts: number[];
  trick: Played[];
  lastTrick: TakenTrick | null;
  /** By seat, sorted by colour then value. */
  won: DgCard[][];
  /** What each seat's won cards are worth now. */
  points: number[];
  scores: number[];
  /** From 1. */
  round: number;
  rounds: number;
  dealer: number;
  turn: number | null;
  lastRound: RoundLine[] | null;
  legal: DgAction[];
}
