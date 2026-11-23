// Mirrors games/wu-hsing (View, Action).

export type Dir = "Right" | "Down";

/** The anchor `(row, col)` holds `a` (or `b` if flipped); the other half is to its right or below. */
export interface Placement {
  domino: number;
  row: number;
  col: number;
  dir: Dir;
  flipped: boolean;
}

export type WuHsingAction = { Place: Placement } | "Pass";

/** A laid domino: `first` is the value on the anchor. `by` is null for the starting square. */
export interface PlacedView {
  row: number;
  col: number;
  dir: Dir;
  first: number;
  second: number;
  by: number | null;
}

export interface HandDomino {
  id: number;
  a: number;
  b: number;
}

export interface LastMove {
  seat: number;
  /** Index in `placed`; null for a pass. */
  placed: number | null;
  points: number;
}

export interface LegalMove {
  action: WuHsingAction;
  points: number;
}

export interface Finish {
  penalties: number[];
  totals: number[];
}

export interface WuHsingView {
  me: number | null;
  placed: PlacedView[];
  hand: HandDomino[];
  handSizes: number[];
  scores: number[];
  turn: number | null;
  first: number;
  lastRound: boolean;
  last: LastMove | null;
  finish: Finish | null;
  /** The placements of the seat to play, with their points, for that seat only. */
  legal: LegalMove[];
}
