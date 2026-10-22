// Mirrors the serde JSON of games/manoever (View, Action). The board itself
// comes from board.json (written by the test board_json_is_up_to_date).

export interface Point {
  x: number;
  y: number;
}

export interface BoardData {
  width: number;
  height: number;
  /** Square n at index n - 1. */
  squares: Point[];
  /** Place n (I to XII) at index n - 1. */
  barracks: Point[];
  discs: { square: number; kind: "White" | "Black" }[];
  grays: { square: number; value: number }[];
  endSquares: number[];
}

export interface Soldier {
  seat: number;
  number: number;
}

export type Dest = { Square: number } | "Barracks";

export type ManoeverAction = "Roll" | { Move: { soldier: number; to: Dest } };

export interface MoveRecord {
  number: number;
  from: number;
  to: Dest;
  gray: number | null;
  place: number | null;
}

export interface Mover {
  number: number;
  square: number;
  position: number;
  height: number;
  bonus: number;
  max: number;
  dests: Dest[];
}

export interface GrayView {
  square: number;
  value: number;
  takenBy: number | null;
}

export interface ManoeverView {
  players: number;
  setup: boolean;
  /** stacks[square - 1], bottom to top. */
  stacks: Soldier[][];
  waiting: number[];
  /** barracks[place - 1]. */
  barracks: (Soldier | null)[];
  grays: GrayView[];
  turn: number | null;
  toMove: Mover[];
  lastRoll: [number, number] | null;
  turnLog: MoveRecord[];
  lastTurns: MoveRecord[][];
  scores: number[];
  over: boolean;
}
