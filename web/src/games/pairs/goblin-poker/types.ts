// Mirrors games/pairs/src/goblin_poker (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type GoblinPokerEvent =
  | { type: "Placed"; seat: number; card: Rank }
  | { type: "TieBreak"; seat: number; card: Rank; kept: boolean }
  | { type: "Starts"; seat: number }
  | { type: "Pitched"; seat: number }
  | { type: "Folded"; seat: number; cost: number }
  | { type: "Caught"; seat: number; card: Rank; paired: boolean }
  | { type: "HandEnded"; winners: number[]; share: number };

export interface GoblinPokerSeatView {
  stack: Rank[];
  handSize: number;
  out: boolean;
  /** Has a card in the pool this round. */
  pitched: boolean;
  /** Balance since the start of the game; may be negative. */
  chips: number;
}

export interface GoblinPokerView {
  handNo: number;
  handsTotal: number;
  /** The round of the hand, from 1; 0 during setup. */
  round: number;
  phase: "Setup" | "Play" | "Over";
  seats: GoblinPokerSeatView[];
  hand: Rank[];
  placed: Rank | null;
  /** The card I have pitched this round. */
  pitched: Rank | null;
  turn: number | null;
  pot: number;
  ante: number;
  /** What folding costs now. */
  foldCost: number | null;
  deck: number;
  discard: number;
  discardSeen: Rank[];
  events: GoblinPokerEvent[];
}

export type GoblinPokerAction = "Fold" | { Place: { card: Rank } } | { Pitch: { card: Rank } };
