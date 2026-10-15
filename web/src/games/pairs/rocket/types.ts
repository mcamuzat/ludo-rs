// Mirrors games/pairs/src/rocket (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type RocketStatus = "In" | "Folded" | "Busted";

export type RocketEvent =
  | { type: "Dealt"; round: number; cards: Rank[]; dealer: Rank }
  | { type: "Hit"; seat: number; card: Rank }
  | { type: "Busted"; seat: number; card: Rank; paid: number }
  | { type: "Folded"; seat: number; paid: number }
  | { type: "DealerHit"; card: Rank }
  | { type: "DealerLost"; card: Rank; seats: number[]; amount: number }
  | { type: "Reshuffled" };

export interface RocketView {
  roundNo: number;
  roundsTotal: number;
  phase: "Play" | "Over";
  stacks: Rank[][];
  status: RocketStatus[];
  dealer: Rank[];
  turn: number | null;
  /** Each player's balance against the house. */
  chips: number[];
  /** What folding would cost the player to move. */
  foldCost: number | null;
  deck: number;
  events: RocketEvent[];
}

export type RocketAction = "Hit" | "Fold";
