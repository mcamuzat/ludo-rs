// Mirrors games/pairs/src/bread_basket (View, Action, Event).

/** −5 for an Onion, else 1 to 10. */
export type BreadCard = number;

export type BreadBasketEvent =
  | { type: "Dealt"; round: number; dealer: number; first: number }
  | { type: "Played"; seat: number; card: BreadCard }
  | { type: "Sandwich"; seat: number; bread: BreadCard; filling: BreadCard[]; points: number }
  | { type: "Air"; seat: number; card: BreadCard }
  | { type: "Drew"; seat: number }
  | { type: "LineStarted"; card: BreadCard }
  | { type: "WentOut"; seat: number }
  | { type: "HandOver"; end: "DeckEmpty" | "WentOut"; gained: number[]; penalties: number[]; scores: number[] };

export interface BreadBasketSeatView {
  /** The middle cards picked up this hand. */
  taken: BreadCard[];
  takenPoints: number;
  handSize: number;
  score: number;
  /** Ran out of cards first this hand. */
  out: boolean;
}

export interface BreadBasketView {
  roundNo: number;
  rounds: number;
  phase: "Play" | "Over";
  dealer: number;
  turn: number | null;
  /** Oldest card first. */
  line: BreadCard[];
  deckSize: number;
  discard: BreadCard[];
  hand: BreadCard[];
  seats: BreadBasketSeatView[];
  events: BreadBasketEvent[];
}

export type BreadBasketAction = { Play: { card: BreadCard } } | "Draw";
