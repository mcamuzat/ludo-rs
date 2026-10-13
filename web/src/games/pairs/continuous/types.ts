// Mirrors games/pairs/src/continuous (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type ContinuousEvent =
  | { type: "Dealt"; seat: number; card: Rank }
  | { type: "Replaced"; seat: number; card: Rank }
  | { type: "Paired"; seat: number; card: Rank }
  | { type: "Folded"; seat: number; from: number; card: Rank }
  | { type: "Reshuffled" };

export interface ContinuousSeatView {
  stack: Rank[];
  scored: Rank[];
  score: number;
}

export interface ContinuousView {
  seats: ContinuousSeatView[];
  target: number;
  turn: number | null;
  loser: number | null;
  deck: number;
  discard: number;
  /** Index = rank − 1. */
  unlocated: number[];
  lowest: Rank | null;
  events: ContinuousEvent[];
}

export type ContinuousAction = "Hit" | { Fold: { seat: number; card: Rank } };
