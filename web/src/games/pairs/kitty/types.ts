// Mirrors games/pairs/src/kitty (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export interface KittyRankScore {
  rank: Rank;
  /** Cards of that rank in each hand. */
  counts: number[];
  /** Who scores it. */
  winners: number[];
}

export type KittyEvent =
  | { type: "TargetRevealed"; round: number; cards: Rank[] }
  | { type: "BidsRevealed"; bids: Rank[]; winners: number[] }
  | { type: "Took"; seat: number; card: Rank }
  | { type: "Gave"; seat: number; to: number; card: Rank }
  | { type: "Discarded"; cards: Rank[] }
  | { type: "Scored"; hands: Rank[][]; ranks: KittyRankScore[]; scores: number[] };

export interface KittyView {
  roundNo: number;
  roundsTotal: number;
  soft: boolean;
  phase: "Pass" | "Bid" | "Claim" | "Over";
  handSizes: number[];
  hand: Rank[];
  /** Pass or Bid: the card I have chosen, not yet revealed. */
  placed: Rank | null;
  /** This round: the card I passed, and the one I received. */
  passed: Rank | null;
  received: Rank | null;
  /** The target cards still to claim. */
  target: Rank[];
  kittySize: number;
  /** Bids and unclaimed targets. */
  discard: Rank[];
  /** Claim: who chooses. */
  turn: number | null;
  /** The final scores (empty until the end). */
  scores: number[];
  events: KittyEvent[];
}

export type KittyAction =
  | { Pass: { card: Rank } }
  | { Bid: { card: Rank } }
  | { Take: { card: Rank } }
  | { Give: { card: Rank; to: number } };
