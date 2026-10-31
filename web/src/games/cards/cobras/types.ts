// Mirrors games/cards/src/cobras (View, Action).

export type CbSuit = "Red" | "Green" | "Blue" | "Yellow";

export interface CbCard {
  suit: CbSuit;
  /** 1 to 13. */
  value: number;
}

export type Share = "Win" | "Loss";

export type CbAction = { Seed: { card: CbCard } } | { Play: { card: CbCard } } | { King: { share: Share } };

/** A player, or the two-player dummy. */
export type Who = { Seat: number } | "Dummy";

export type Played = { Card: CbCard } | { King: Share };

export interface Play {
  who: Who;
  played: Played;
}

export interface Scored {
  seat: number;
  cobras: number;
  points: number;
}

export interface TrickResult {
  plays: Play[];
  /** Won with a card; null when only King Cobras were played. */
  winner: Who | null;
  scored: Scored[];
  losers: Who[];
  /** Cobras each loser received. */
  share: number;
  /** Left in the centre. */
  center: number;
}

export interface RoundLine {
  gained: number;
  penalty: number;
  total: number;
}

export interface DummyView {
  count: number;
  /** Its next card, when the pile is face up. */
  top: CbCard | null;
}

export type CbPhase = "Seed" | "Trick" | "Over";

export interface CbView {
  me: number | null;
  phase: CbPhase;
  /** By suit, highest first. */
  hand: CbCard[];
  /** The card the viewer picked for its basket, before the reveal. */
  chosen: CbCard | null;
  handCounts: number[];
  dummy: DummyView | null;
  /** The cards that started the baskets, once revealed. */
  revealed: (CbCard | null)[];
  baskets: number[];
  /** What each basket would score now. */
  basketPoints: number[];
  center: number;
  triples: number;
  triplesToEnd: number;
  /** The seat still holds its King Cobra. */
  kings: boolean[];
  trick: Play[];
  led: CbSuit | null;
  leader: Who;
  turn: Who | null;
  lastTrick: TrickResult | null;
  firstLeader: number;
  scores: number[];
  target: number;
  /** From 1. */
  round: number;
  dealer: number;
  lastRound: RoundLine[] | null;
  legal: CbAction[];
}
