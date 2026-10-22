// Mirrors games/pairs/src/the_cave (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type TheCaveEvent =
  | { type: "Dealt"; round: number; dealer: number; first: number }
  | { type: "Played"; seat: number; card: Rank; captured: Rank[] }
  | { type: "Refilled"; cards: Rank[] }
  | { type: "RoundOver"; points: number[]; scores: number[] };

export interface TheCaveSeatView {
  /** Captured this round, sorted. */
  captured: Rank[];
  handSize: number;
  score: number;
}

export interface TheCaveView {
  roundNo: number;
  rounds: number;
  phase: "Play" | "Over";
  dealer: number;
  turn: number | null;
  cave: Rank[];
  deckSize: number;
  hand: Rank[];
  seats: TheCaveSeatView[];
  events: TheCaveEvent[];
}

export type TheCaveAction = { Play: { card: Rank } };
