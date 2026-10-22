// Mirrors games/pairs/src/pickup (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type PickupEvent =
  | { type: "Dealt"; deal: number; dealer: number }
  | { type: "PoolDealt"; cards: Rank[] }
  | { type: "Revealed"; cards: (Rank | null)[] }
  /** `knocked`: [seat, count] for each other pile that lost cards. */
  | { type: "Took"; seat: number; played: Rank; card: Rank; knocked: [number, number][]; pool_knocked: number }
  | { type: "Stacked"; seat: number; card: Rank }
  | { type: "DealOver"; points: number[]; scores: number[] };

export interface PickupSeatView {
  pile: Rank[];
  pilePoints: number;
  handSize: number;
  score: number;
  /** The card revealed this turn, while it has not acted yet. */
  waiting: Rank | null;
}

export interface PickupView {
  dealNo: number;
  /** Long game: the score to reach. */
  target: number | null;
  phase: "Choose" | "Take" | "Over";
  dealer: number;
  pool: Rank[];
  deckSize: number;
  hand: Rank[];
  /** My card, chosen and not yet revealed. */
  chosen: Rank | null;
  /** Who picks a card, with the card it played. */
  taking: [number, Rank] | null;
  seats: PickupSeatView[];
  events: PickupEvent[];
}

export type PickupAction = { Choose: { card: Rank } } | { Take: { card: Rank } };
