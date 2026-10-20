// Mirrors games/black-vienna (View, Action, Event, Note).

/** A suspect, by its letter: "A" … "Z", "Ö". */
export type Suspect = string;
/** An investigation card, by its place in `view.investigations`. */
export type CardId = number;
export type Trio = [Suspect, Suspect, Suspect];

export type Column = { Seat: number } | "Trio";
export type Mark = "Plus" | "Minus";

export interface Note {
  suspect: Suspect;
  column: Column;
  mark: Mark;
}

export interface Answer {
  by: number;
  target: number;
  card: CardId;
  /** How many of its suspects the target holds. */
  count: number;
  /** Tokens actually taken (fewer when the reserve runs out). */
  placed: number;
  /** An earlier card laid again. */
  special: boolean;
}

export type End = "Solved" | "Reserve" | "Stuck" | "Failed";

export type BlackViennaEvent =
  | { type: "Asked"; by: number; target: number; answers: Answer[] }
  | { type: "Accused"; seat: number; right: boolean }
  | {
      type: "Revealed";
      end: End;
      trio: Trio;
      accuser: number | null;
      guesses: (Trio | null)[];
      scores: number[];
    };

export interface PileView {
  top: CardId | null;
  left: number;
}

export interface BlackViennaView {
  phase: "Turn" | "Guess" | "Over";
  turn: number | null;
  players: number;
  dealer: number;
  hand: Suspect[];
  handSizes: number[];
  reserve: number;
  piles: PileView[];
  /** The 27 suspects, in the sheet's order. */
  suspects: Suspect[];
  /** The letters of each investigation card, by card id. */
  investigations: string[];
  answers: Answer[];
  wrong: boolean[];
  asked: boolean[];
  canAccuse: boolean;
  specialAllowed: boolean;
  reusable: CardId[];
  last: number | null;
  finalAsked: boolean;
  myGuess: Trio | null;
  trio: Trio | null;
  accuser: number | null;
  guesses: (Trio | null)[];
  scores: number[];
  /** The sheet's automatic notes (direct facts only). */
  notes: Note[];
  events: BlackViennaEvent[];
}

export type BlackViennaAction =
  | { Ask: { pile: number | null; target: number; extra: CardId | null } }
  | { Accuse: { trio: Trio } }
  | { Guess: { trio: Trio } };
