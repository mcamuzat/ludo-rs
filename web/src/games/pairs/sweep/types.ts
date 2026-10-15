// Mirrors games/pairs/src/sweep (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export interface SweepRankScore {
  rank: Rank;
  counts: number[];
  winners: number[];
}

export type SweepEvent =
  | { type: "TargetsDealt"; round: number; targets: Rank[]; leader: number }
  | { type: "Shot"; seat: number; card: Rank }
  | { type: "Captured"; seat: number; cards: Rank[] }
  | { type: "Discarded"; cards: Rank[] }
  | { type: "Scored"; hands: Rank[][]; ranks: SweepRankScore[]; scores: number[] };

export interface SweepView {
  roundNo: number;
  roundsTotal: number;
  phase: "Play" | "Capture" | "Over";
  handSizes: number[];
  hand: Rank[];
  targets: Rank[];
  /** This round's shooters still on the table, per seat. */
  shooters: (Rank | null)[];
  leader: number;
  turn: number | null;
  /** The final scores (empty until the end). */
  scores: number[];
  events: SweepEvent[];
}

export type SweepAction = { Shoot: { card: Rank } } | { Capture: { card: Rank } };
