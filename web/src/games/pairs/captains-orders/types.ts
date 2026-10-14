// Mirrors games/pairs/src/captains_orders (View, Action, Event, RoundResult).

/** 1 to 10. */
export type Rank = number;

export type CaptainsOrdersPhase =
  | { type: "Turn" }
  | { type: "Respond" }
  | { type: "Give"; asker: number }
  | { type: "Discard"; after: number }
  | { type: "Over" };

/** A card the viewer has not seen is `null`. */
export type CaptainsOrdersEvent =
  | { type: "Ordered"; seat: number; card: Rank; belay: boolean }
  | { type: "Followed"; seat: number; owner: number; card: Rank | null }
  | { type: "OrderDone"; owner: number; card: Rank }
  | { type: "Passed"; from: number; to: number; card: Rank | null }
  | { type: "Asked"; from: number; to: number }
  | { type: "Gave"; from: number; to: number; card: Rank | null }
  | { type: "Discarded"; seat: number; card: Rank | null };

export type CaptainsOrdersEnding =
  | { type: "Accused"; accuser: number; target: number; right: boolean }
  | { type: "Emptied"; seat: number; card: Rank; right: boolean };

export interface CaptainsOrdersRoundResult {
  round: number;
  ending: CaptainsOrdersEnding;
  /** Every hand, revealed. */
  hands: Rank[][];
  captains: number[];
  points: number[];
  winners: number[];
}

export interface CaptainsOrdersSeatView {
  handSize: number;
  /** Rounds won. */
  wins: number;
}

export interface CaptainsOrdersView {
  round: number;
  winsNeeded: number;
  dealer: number;
  phase: CaptainsOrdersPhase;
  seats: CaptainsOrdersSeatView[];
  hand: Rank[];
  /** My face-down discards this round. */
  discarded: Rank[];
  order: { card: Rank; owner: number } | null;
  turn: number | null;
  log: CaptainsOrdersEvent[];
  lastRound: CaptainsOrdersRoundResult | null;
}

export type CaptainsOrdersAction =
  | { Order: { card: Rank } }
  | { Follow: { card: Rank } }
  | { Pass: { card: Rank; to: number } }
  | { Ask: { from: number } }
  | { Give: { card: Rank } }
  | { Discard: { card: Rank } }
  | { Accuse: { seat: number } };
