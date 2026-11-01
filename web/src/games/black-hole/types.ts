// Mirrors games/black-hole (View, Action).

export type Order = "Ascending" | "Free" | "Random";

/** Owner 0 is red, 1 blue. */
export interface Tile {
  owner: number;
  value: number;
}

export type BlackHoleAction = { Place: { cell: number; tile: number } };

export interface Finish {
  hole: number;
  /** Red's and blue's totals around the hole. */
  sums: [number, number];
}

export interface BlackHoleView {
  me: number | null;
  order: Order;
  /** 21 cells, row by row from the apex. */
  cells: (Tile | null)[];
  /** Each seat's tiles still to lay, smallest first. */
  remaining: number[][];
  /** The tile the seat to move must lay (null in the free order). */
  next: number | null;
  turn: number | null;
  last: number | null;
  finish: Finish | null;
  /** The cells around the hole, once the game is over. */
  around: number[];
  legal: BlackHoleAction[];
}
