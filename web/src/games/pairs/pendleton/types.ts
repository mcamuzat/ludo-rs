// Mirrors games/pairs/src/pendleton (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export interface PendletonColumnScore {
  target: Rank;
  /** Points each seat played there. */
  played: number[];
  /** Points each seat scores for it. */
  scored: number[];
}

export type PendletonEvent =
  | { type: "RoundStarted"; targets: Rank[]; first: number }
  | { type: "Played"; seat: number; card: Rank; column: number; target: Rank }
  | {
      type: "RoundScored";
      seat: number;
      /** The card no column accepts; null when the draw pile ran out. */
      card: Rank | null;
      columns: PendletonColumnScore[];
      round: number[];
      totals: number[];
    };

export interface PendletonColumnView {
  target: Rank;
  /** Each seat's side, in the order played. */
  sides: Rank[][];
  points: number[];
}

export interface PendletonView {
  roundNo: number;
  targetScore: number;
  phase: "Play" | "Over";
  columns: PendletonColumnView[];
  /** The card the player to move has drawn. */
  drawn: Rank | null;
  turn: number | null;
  scores: number[];
  deckSize: number;
  events: PendletonEvent[];
}

export type PendletonAction = { Play: { column: number } };
