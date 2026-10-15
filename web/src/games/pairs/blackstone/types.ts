// Mirrors games/pairs/src/blackstone (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type BlackstoneStatus = "In" | "Folded" | "Lost";

export type BlackstoneEvent =
  | { type: "Opened"; hand: number; cards: Rank[][]; first: number; returned: number }
  | { type: "Took"; seat: number; count: number; cards: Rank[] }
  | { type: "Folded"; seat: number; paid: number }
  | { type: "Lost"; seat: number; paid: number }
  | { type: "PotWon"; seats: number[]; share: number; left: number };

export interface BlackstoneSeatView {
  blacks: number;
  status: BlackstoneStatus;
  /** Balance since the start of the game, an ante in front counted as spent. */
  chips: number;
}

export interface BlackstoneView {
  handNo: number;
  handsTotal: number;
  phase: "Play" | "Over";
  seats: BlackstoneSeatView[];
  /** The white stones dealt this hand. */
  whites: Rank[];
  minTake: number;
  /** What folding costs now, ante included. */
  foldCost: number;
  turn: number | null;
  pot: number;
  ante: number;
  deck: number;
  events: BlackstoneEvent[];
}

export type BlackstoneAction = { Take: { count: number } } | "Fold";
