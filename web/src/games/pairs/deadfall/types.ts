// Mirrors games/pairs/src/deadfall (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type DeadfallEvent =
  | { type: "DoorsRevealed"; cards: Rank[]; leader: number | null }
  | { type: "Played"; seat: number; card: Rank }
  | {
      type: "Showdown";
      caller: number;
      target: number;
      rank: Rank;
      dead: boolean;
      /** Every hand, shown. */
      hands: Rank[][];
      winner: number;
      paid: number;
      pot: number;
    };

export interface DeadfallSeatView {
  /** Face-up cards; the top is the last. */
  stack: Rank[];
  handSize: number;
  /** Balance since the start of the game; may be negative. */
  chips: number;
}

export interface DeadfallView {
  handNo: number;
  handsTotal: number;
  phase: "Door" | "Play" | "Over";
  seats: DeadfallSeatView[];
  hand: Rank[];
  /** Door: the card I have chosen, not yet revealed. */
  placed: Rank | null;
  turn: number | null;
  pot: number;
  ante: number;
  events: DeadfallEvent[];
}

export type DeadfallAction = { Door: { card: Rank } } | { Play: { card: Rank } } | { Call: { seat: number } };
