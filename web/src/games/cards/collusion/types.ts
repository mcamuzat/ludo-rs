// Mirrors games/cards/src/collusion (View, Action, Phrase).
import type { Card } from "../types";

export type Phrase = { type: "Aim"; n: number } | { type: "Unsure" } | { type: "WhoAims"; n: number };

export type CollusionAction = { Play: { card: Card } } | { Say: { phrase: Phrase } };

/** A card on the table: who played it. */
export type Played = [number, Card];

export interface TakenTrick {
  winner: number;
  cards: Played[];
}

export interface DealLine {
  tricks: number;
  bonus: number;
  /** No bonus, and the tricks would have reached the target: taken off. */
  deducted: boolean;
  total: number;
}

export interface CollusionView {
  me: number | null;
  /** By suit, strongest first. */
  hand: Card[];
  handCounts: number[];
  trick: Played[];
  lastTrick: TakenTrick | null;
  taken: number[];
  scores: number[];
  target: number;
  dealer: number;
  /** From 1. */
  dealNo: number;
  turn: number | null;
  lastSaid: (Phrase | null)[];
  talk: [number, Phrase][];
  lastDeal: DealLine[] | null;
  legal: CollusionAction[];
}
