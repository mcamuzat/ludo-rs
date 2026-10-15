// Mirrors games/cards/src/president (View, Action, Event).
import type { Card, Face, Rank } from "../types";

export type Title = "President" | "VicePresident" | "Neutral" | "Secretary" | "Scum";

export interface Trick {
  seat: number;
  /** null: jokers alone. */
  rank: Rank | null;
  cards: Card[];
}

export type PresidentEvent =
  | { type: "RoundStarted"; round: number; dealer: number; leader: number; aside: Card[] }
  | { type: "Exchanged"; from: number; to: number; count: number }
  | { type: "Played"; seat: number; rank: Rank | null; cards: Card[] }
  | { type: "Passed"; seat: number }
  | { type: "TrickClosed"; leader: number }
  | { type: "Revolution"; reversed: boolean }
  | { type: "Finished"; seat: number; fouled: boolean }
  | { type: "RoundScored"; order: number[]; points: number[]; scores: number[] };

export interface PresidentView {
  roundNo: number;
  roundsTotal: number;
  phase: "Exchange" | "Play" | "Over";
  reversed: boolean;
  handSizes: number[];
  /** My hand, weakest first. */
  hand: Card[];
  trick: Trick | null;
  passes: number[];
  turn: number | null;
  /** From the previous round (empty during the first). */
  titles: Title[];
  scores: number[];
  out: number[];
  fouled: number[];
  setAside: Card[];
  /** This round's exchange, as far as I took part in it. */
  received: Card[];
  given: Card[];
  toGive: number[];
  events: PresidentEvent[];
}

export type PresidentAction =
  | { Play: { rank: Rank | null; naturals: number; jokers: number } }
  | "Pass"
  | { Give: { faces: Face[] } };
