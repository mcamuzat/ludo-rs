// Mirrors games/yavalath/src (View, Action).

export type Color = "White" | "Black" | "Red";

export interface Cell {
  q: number;
  r: number;
}

export interface CellView extends Cell {
  stone: Color | null;
}

export interface YavalathView {
  cells: CellView[];
  seatColors: Color[];
  current: number | null;
  eliminated: number[];
  forced: Cell[];
  lastMove: Cell | null;
  lines: Cell[][];
  swapped: boolean;
}

export type YavalathAction = { Place: Cell } | "Swap";
