// Mirrors games/black-pot/src (View, Action, Expr, Group, TurnRecord,
// Alignment, End) and board.json (written by the test board_json_is_up_to_date).

export type Kind = "Lemon" | "Cherry" | "BlackPot";
export type Dir = "Row" | "Column" | "Diagonal" | "AntiDiagonal";
export type Op = "Add" | "Sub" | "Mul" | "Div";

export type Expr = { Die: number } | { Op: { op: Op; left: Expr; right: Expr } };

export interface Group {
  expr: Expr;
  /** null exactly when the calculation gives 0. */
  cell: number | null;
}

export type BlackPotAction = { Play: { groups: Group[] } } | "NotFound";

export interface PlayedGroup {
  text: string;
  value: number;
  cell: number | null;
}

export type TurnRecord =
  | { Played: { dice: number[]; groups: PlayedGroup[]; points: number } }
  | { NotFound: { dice: number[] } }
  | { Passed: { dice: number[] } };

export interface Alignment {
  team: number;
  dir: Dir;
  cells: number[];
  points: number;
}

export type End = { AllPlaced: { team: number } } | "QuietLimit";

export interface Pawn {
  team: number;
  flipped: boolean;
}

export interface TeamView {
  seats: number[];
  pawnsLeft: number;
  score: number;
}

export interface BlackPotView {
  players: number;
  pawns: (Pawn | null)[];
  dice: number[];
  roller: number;
  turn: number | null;
  queue: number[];
  teams: TeamView[];
  alignments: Alignment[];
  last: (TurnRecord | null)[];
  quietLeft: number;
  end: End | null;
  /** Teams ranked first once the game is over (both on a tie). */
  winners: number[];
}

export interface BoardCell {
  row: number;
  col: number;
  value: number;
  kind: Kind;
}

export interface BoardData {
  cells: BoardCell[];
}
