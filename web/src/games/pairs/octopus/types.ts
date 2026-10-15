// Mirrors games/pairs/src/octopus (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type OctopusKind = "Set" | "Clean" | "Dirty";

export type OctopusEvent =
  | { type: "HandStarted"; hand: number; antes: number[]; leader: number }
  | { type: "Declared"; seat: number; fold: boolean }
  | { type: "FoldedOut"; seat: number; pot: number }
  | { type: "CardsPlayed"; cards: (Rank | null)[]; caller: number | null }
  | { type: "PassCalled"; seat: number }
  | { type: "Passed" }
  | {
      type: "Showdown";
      caller: number | null;
      /** null when the hands ran down to one card. */
      best: OctopusKind | null;
      /** null: folded. */
      hands: (Rank[] | null)[];
      winners: number[];
      losers: number[];
      share: number;
    };

export interface OctopusSeatView {
  handSize: number;
  folded: boolean;
  /** Balance since the start of the game; may be negative. */
  chips: number;
}

export interface OctopusView {
  handNo: number;
  handsTotal: number;
  phase: "Declare" | "Play" | "Call" | "Pass" | "Over";
  seats: OctopusSeatView[];
  hand: Rank[];
  /** Play or Pass: the card I have chosen, not yet revealed. */
  placed: Rank | null;
  /** Since the last pass: the card I passed, and the one I received. */
  passed: Rank | null;
  received: Rank | null;
  /** The cards played this hand. */
  played: Rank[];
  /** Declare: whose turn; Call: who calls. */
  turn: number | null;
  leader: number;
  pot: number;
  events: OctopusEvent[];
}

export type OctopusAction =
  | "Stay"
  | "Fold"
  | "CallPass"
  | { Play: { card: Rank } }
  | { Pass: { card: Rank } }
  | { Show: { best: OctopusKind } };
