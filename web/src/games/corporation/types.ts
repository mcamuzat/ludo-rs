// Mirrors games/corporation (View, Action).

export type Color = "Red" | "Blue" | "Green" | "Yellow";
export type Phase = "Draft" | "Swap" | "Entry" | "Promote" | "Over";
export type TableSetup = "Draft" | "Random";

export type CorporationAction =
  | { Place: { circle: number; color: Color } }
  | { Swap: { a: number; b: number } }
  | { Enter: { color: Color; desk: number } }
  | "Skip"
  | { Promote: { from: number } };

export type Step =
  | { Swap: { a: number; b: number } }
  | { Enter: { color: Color; desk: number } }
  | { Promote: { color: Color; from: number; to: number } };

export type End = { Won: { seat: number } } | "Quiet";

export interface LastTurn {
  seat: number;
  steps: Step[];
}

export interface CorporationView {
  me: number | null;
  /** Each seat's colour; the colours past them are neutral. */
  colors: Color[];
  /** 12 circles: circle 2k is promotion k + 1's employee, 2k + 1 the colour beneath. */
  table: (Color | null)[];
  /** Counters left to lay while setting up: red, blue, green, yellow. */
  supply: number[];
  /** 13 positions: desks 0–3, offices 4–11 from the ground floor, boardroom 12 (null). */
  rooms: (Color | null)[];
  /** Each position's stack, bottom first. */
  stacks: Color[][];
  /** Employees not yet entered: red, blue, green, yellow. */
  outside: number[];
  /** Employees in the boardroom: red, blue, green, yellow. */
  arrived: number[];
  phase: Phase;
  turn: number | null;
  /** Positions whose top employee a promotion applies to. */
  eligible: number[];
  steps: Step[];
  lastTurn: LastTurn | null;
  quietLeft: number;
  end: End | null;
  legal: CorporationAction[];
}
