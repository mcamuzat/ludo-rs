// Mirrors games/pairs/src/horn_of_plenty (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type HornOfPlentyEvent =
  | {
      type: "Dealt";
      round: number;
      upcards: Rank[];
      /** [seat, card], thrown away after the tie is broken. */
      tie_breaks: [number, Rank][];
      leader: number;
    }
  | { type: "Declared"; seat: number; declared: Rank }
  | { type: "Passed"; seat: number }
  | { type: "Answered"; seat: number; agreed: boolean; card: Rank; right: boolean; chooser: number }
  | { type: "Unanswered"; card: Rank }
  | { type: "Given"; by: number; to: number; card: Rank; paired: boolean }
  | { type: "Scored"; points: number[]; gained: number[]; scores: number[] };

export interface HornOfPlentySeatView {
  shown: Rank[];
  /** The ranks of the face-up hand added up. */
  cardPoints: number;
  handSize: number;
  score: number;
}

export interface HornOfPlentyTableView {
  poser: number;
  declared: Rank;
  /** Once revealed, or to its player. */
  card: Rank | null;
  /** Who must answer now. */
  asked: number | null;
}

export interface HornOfPlentyView {
  roundNo: number;
  target: number;
  phase: "Declare" | "Answer" | "Place" | "Over";
  seats: HornOfPlentySeatView[];
  /** My face-down cards. */
  hand: Rank[];
  table: HornOfPlentyTableView | null;
  turn: number | null;
  chooser: number | null;
  events: HornOfPlentyEvent[];
}

export type HornOfPlentyAction =
  | { Play: { card: Rank; declared: Rank } }
  | "Agree"
  | "Disagree"
  | "Pass"
  | { Give: { to: number } };
