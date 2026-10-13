// Mirrors games/pairs/src/zombie_life (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type Zone = "Refuge" | "Horde";

export type ZombieLifeEvent =
  | { type: "Dealt"; seat: number; card: Rank }
  | { type: "Starts"; seat: number }
  | { type: "Drew"; seat: number; card: Rank }
  | { type: "Placed"; seat: number; card: Rank; zone: Zone }
  | { type: "Took"; seat: number; card: Rank }
  | { type: "Passed"; seat: number; hidden: number }
  | { type: "Overrun"; seat: number; card: Rank }
  | { type: "Scored"; seat: number; points: number }
  | { type: "Dealer"; seat: number }
  | { type: "Reshuffled" };

export interface ZombieLifeSeatView {
  faceUp: Rank[];
  /** Cards turned face down by passing: they score nothing. */
  faceDown: number;
  status: "In" | "Passed" | "Out";
  score: number;
}

export interface ZombieLifeView {
  round: number;
  target: number;
  phase: "Dealing" | "Play" | "Placing" | "Over";
  seats: ZombieLifeSeatView[];
  horde: Rank[];
  /** Seat to move; the dealer while dealing. */
  turn: number | null;
  /** Placing: the card `turn` has drawn. */
  drawn: Rank | null;
  /** Dealing: the dealer's cards not yet given. */
  dealing: Rank[];
  /** Dealing: the seat the next card goes to. */
  dealTo: number | null;
  deck: number;
  discard: number;
  events: ZombieLifeEvent[];
}

export type ZombieLifeAction =
  | "Draw"
  | "Pass"
  | { Place: { zone: Zone } }
  | { Take: { card: Rank } }
  | { Give: { card: Rank } };
