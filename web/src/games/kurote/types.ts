// Mirrors games/kurote (View, Action).

/** Owner 0 is green (bottom), 1 yellow (top). */
export interface Wizard {
  owner: number;
  exhausted: boolean;
}

export type KuroteAction = { Move: { from: number; to: number } } | { Place: { to: number } } | "Pass";

export type End = { Captures: { winner: number } } | "Blocked" | "Limit";

export interface Last {
  seat: number;
  action: KuroteAction;
  /** The cells whose wizards it captured. */
  captured: number[];
}

export interface KuroteView {
  me: number | null;
  /** 25 cells, row by row from yellow's end (top). */
  cells: (Wizard | null)[];
  reserve: [number, number];
  /** The wizards each seat has captured. */
  captured: [number, number];
  /** Each seat's end of the board (row number). */
  homeRows: [number, number];
  turn: number | null;
  /** Turns left before the draw for lack of captures. */
  quietLeft: number;
  last: Last | null;
  end: End | null;
  legal: KuroteAction[];
}
