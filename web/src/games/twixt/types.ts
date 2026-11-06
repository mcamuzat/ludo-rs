// Mirrors games/twixt (TwixtView, TwixtAction) and games/twixt/src/coach.rs.

export type Color = "Red" | "Black";

export interface Point {
  x: number;
  y: number;
}

export type TwixtAction = { Place: Point } | "Swap";

export type Finish = { Won: Color } | "Draw";

export interface TwixtView {
  pegs: [Point, Color][];
  links: [Color, Point, Point][];
  turn: Color;
  canSwap: boolean;
  moves: TwixtAction[];
  last: Point | null;
  result: Finish | null;
  /** The winner's group joining its two borders. */
  chain: Point[] | null;
}

export type Grade = "Best" | "Good" | "Inaccuracy" | "Mistake" | "Blunder";

export interface Candidate {
  at: Point;
  win: number;
}

export interface LastMove {
  ply: number;
  loss: number;
  grade: Grade;
}

export interface Advice {
  win: number;
  best: Candidate[];
  heat: [Point, number][];
  last: LastMove | null;
}

export interface ReviewPoint {
  /** Red's win probability after this many moves. */
  redWin: number;
  /** The side to move's best win among the network's favourite holes. */
  best: number | null;
}

export interface Review {
  grades: (Grade | null)[];
  key: number[];
}
