// Mirrors games/pairs/src/troll_guts (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type TrollGutsSource = { kind: "Deck" } | { kind: "Stack"; seat: number };

export type TrollGutsEvent =
  | { type: "HandStarted"; hand: number; first: number }
  | { type: "Dealt"; seat: number; card: Rank }
  | { type: "Replaced"; seat: number; card: Rank }
  | { type: "Arranged" }
  | { type: "Took"; seat: number; card: Rank; from: TrollGutsSource }
  | { type: "Busted"; seat: number; card: Rank; paid: number }
  | { type: "Reshuffled" }
  | { type: "PotWon"; seats: number[]; share: number };

export interface TrollGutsSeatView {
  faceup: Rank[];
  stackSize: number;
  out: boolean;
  /** Balance since the start of the game; may be negative. */
  chips: number;
}

export interface TrollGutsView {
  handNo: number;
  handsTotal: number;
  phase: "Arrange" | "Play" | "Over";
  seats: TrollGutsSeatView[];
  /** My face-down stack, top card first. */
  stack: Rank[];
  /** Arrange: the order I have chosen, top first. */
  placed: Rank[] | null;
  turn: number | null;
  pot: number;
  deck: number;
  events: TrollGutsEvent[];
}

export type TrollGutsAction =
  | { Arrange: { order: Rank[] } }
  | "Draw"
  | { Steal: { from: number } }
  | "Buy";
