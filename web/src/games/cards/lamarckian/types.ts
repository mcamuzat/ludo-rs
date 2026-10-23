// Mirrors games/cards/src/lamarckian (View, Action, Event, PokerHand).
import type { Card, Rank } from "../types";

export type LamarckianAction = { Mark: { card: Card } } | { Pick: { card: Card } };

export type Phase = "Choose" | "Pick" | "Over";

export type Category =
  | "HighCard"
  | "Pair"
  | "TwoPair"
  | "Trips"
  | "Straight"
  | "Flush"
  | "FullHouse"
  | "Quads"
  | "StraightFlush";

export interface PokerHand {
  category: Category;
  /** Most significant first. */
  cards: Card[];
}

export interface ShowdownLine {
  hand: PokerHand;
  /** 0: best hand; equal hands share a place. */
  place: number;
  points: number;
  score: number;
}

export type LamarckianEvent =
  | { type: "Competition"; no: number; lowest: Rank }
  | { type: "PoolDealt"; cards: Card[] }
  | { type: "Revealed"; cards: (Card | null)[] }
  | { type: "Captured"; seat: number; mark: Card; cards: Card[] }
  | { type: "Picked"; seat: number; mark: Card; card: Card }
  | { type: "Showdown"; places: number[]; points: number[]; scores: number[] };

export interface LamarckianSeatView {
  handSize: number;
  /** The cards everyone saw this player take. */
  known: Card[];
  score: number;
  /** This turn's mark, once revealed. */
  mark: Card | null;
  waiting: boolean;
}

export interface LamarckianView {
  me: number | null;
  competition: number;
  competitions: number;
  turnNo: number;
  turns: number;
  lowest: Rank;
  phase: Phase;
  pool: Card[];
  deckSize: number;
  hand: Card[];
  best: PokerHand | null;
  chosen: Card | null;
  picking: [number, Card] | null;
  seats: LamarckianSeatView[];
  lastShowdown: ShowdownLine[] | null;
  events: LamarckianEvent[];
}
