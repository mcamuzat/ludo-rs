// Mirrors games/siberian-dice/src (View, CellView, DieView, Step, Source,
// Action, LastMove, End).

export type Source = "Reserve" | { Cell: number };

export type SiberianDiceAction = { Pick: Source } | { Place: number };

export type Step = "Choose" | { Place: { from: Source; value: number } };

export interface DieView {
  /** 0 blue, 1 red. */
  owner: number;
  value: number;
  perfect: boolean;
}

/** A cell and its printed number `ab`. */
export interface CellView {
  a: number;
  b: number;
  die: DieView | null;
}

export interface LastMove {
  seat: number;
  from: Source;
  to: number;
  value: number;
}

export type End = { Won: { seat: number } } | "BothSix" | "QuietLimit";

export interface SiberianDiceView {
  cells: CellView[];
  reserve: number[];
  perfect: number[];
  turn: number;
  step: Step;
  last: LastMove | null;
  quietLeft: number;
  end: End | null;
  winners: number[];
}
