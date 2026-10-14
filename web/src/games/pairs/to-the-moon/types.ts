// Mirrors games/pairs/src/to_the_moon (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type ToTheMoonEvent =
  | { type: "Dealt"; round: number; dealer: number }
  | { type: "Led"; seat: number; card: Rank }
  | { type: "Built"; seat: number; card: Rank }
  | { type: "Tested"; seat: number; card: Rank; ok: boolean }
  | { type: "Withdrew"; seat: number; took: Rank | null }
  | { type: "TrickWon"; seat: number; cards: number; perfect: boolean }
  | { type: "RoundEnded"; round: number };

export interface ToTheMoonSeatView {
  handSize: number;
  /** Cards of this hand everyone has seen go in. */
  known: Rank[];
  /** Out of the current trick. */
  out: boolean;
  score: number;
}

export interface ToTheMoonView {
  roundNo: number;
  roundsTotal: number;
  phase: "Play" | "Over";
  dealer: number;
  leader: number;
  seats: ToTheMoonSeatView[];
  hand: Rank[];
  /** The countdown, in play order. */
  trick: Rank[];
  turn: number | null;
  deck: number;
  events: ToTheMoonEvent[];
}

export type ToTheMoonAction = "Test" | { Build: { card: Rank } } | { Withdraw: { take: Rank | null } };
