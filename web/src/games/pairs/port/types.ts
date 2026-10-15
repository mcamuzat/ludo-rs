// Mirrors games/pairs/src/port (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type PortStatus = "Playing" | "Locked" | "Busted" | "Out";

export type PortEvent =
  | { type: "RoundStarted"; round: number; seats: number[]; first: number }
  | { type: "Dealt"; seat: number; card: Rank }
  | { type: "Replaced"; seat: number; card: Rank }
  | { type: "Busted"; seat: number; card: Rank }
  | { type: "Locked"; seat: number }
  | { type: "Reshuffled" }
  | {
      type: "RoundScored";
      /** null: busted or out of the round. */
      totals: (number | null)[];
      points: number[];
      scores: number[];
    };

export interface PortSeatView {
  stack: Rank[];
  total: number;
  status: PortStatus;
  score: number;
}

export interface PortView {
  roundNo: number;
  target: number;
  phase: "Play" | "Over";
  seats: PortSeatView[];
  turn: number | null;
  deck: number;
  discard: number;
  events: PortEvent[];
}

export type PortAction = "Hit" | "Lock";
