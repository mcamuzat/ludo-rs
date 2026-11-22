// Mirrors games/five-by-fifteen (View, Action).

export type Color = "Emerald" | "Crimson" | "Azure" | "Amber" | "Violet";
export type Route = "Short" | "Medium" | "Long";
export type Status = "Playing" | "Won" | "Stuck" | "GaveUp";

export interface Tile {
  color: Color;
  number: number;
}

/** `col` 0 to 14: columns 1 to 15 of the rule (the "1"s are `anchors`). */
export interface Pos {
  row: number;
  col: number;
}

export type FiveByFifteenAction =
  | { Place: { from: Pos; to: Pos } }
  | { Shift: { row: number; to: number } }
  | "GiveUp";

/** A 15 led by its 13 and 14; `ready`: it may move into the gap after it. */
export interface Locomotive {
  row: number;
  col: number;
  ready: boolean;
}

export interface FiveByFifteenView {
  route: Route;
  minRun: number;
  anchors: Color[];
  cells: (Tile | null)[][];
  status: Status;
  moves: number;
  repeated: boolean;
  /** Tiles in place from the left of each row (0 to 14). */
  sorted: number[];
  locomotives: Locomotive[];
}
