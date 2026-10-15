// Mirrors games/pairs/src/starboard (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type StarboardStatus = "In" | "Locked" | "Busted" | "Knocked";

export type StarboardEvent =
  | { type: "RoundStarted"; round: number; button: number; center: Rank[] }
  | { type: "Hit"; seat: number; card: Rank; knocked: number[] }
  | { type: "Busted"; seat: number; card: Rank }
  | { type: "Locked"; seat: number; auto: boolean }
  | { type: "RoundEnded"; winners: number[]; share: number; left: number };

export interface StarboardSeatView {
  status: StarboardStatus;
  /** Balance since the start of the game; may be negative. */
  chips: number;
}

export interface StarboardView {
  roundNo: number;
  roundsTotal: number;
  phase: "Play" | "Over";
  center: Rank[];
  seats: StarboardSeatView[];
  button: number;
  turn: number | null;
  pot: number;
  events: StarboardEvent[];
}

export type StarboardAction = "Hit" | "Lock";
