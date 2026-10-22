// Mirrors games/pairs/src/market_day (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type Fate = "Scored" | "Spoiled" | "Killed";

export type MarketDayEvent =
  | { type: "Dealt"; hand: number; dealer: number }
  | { type: "Played"; seat: number; card: Rank }
  | { type: "Swapped"; seat: number; with: number; card: Rank }
  /** Each card with its owner after any trade. */
  | { type: "TrickScored"; cards: [number, Rank, Fate][]; gained: number[]; scores: number[] }
  | { type: "HandOver"; scores: number[] };

export interface MarketDaySeatView {
  handSize: number;
  score: number;
}

export interface MarketDayView {
  handNo: number;
  hands: number;
  phase: "Play" | "Tinker" | "Over";
  dealer: number;
  leader: number;
  turn: number | null;
  /** The round in progress: [seat, card] in play order. */
  trick: [number, Rank][];
  discard: Rank[];
  hand: Rank[];
  seats: MarketDaySeatView[];
  events: MarketDayEvent[];
}

export type MarketDayAction = { Play: { card: Rank } } | { Swap: { with: number } } | "Keep";
