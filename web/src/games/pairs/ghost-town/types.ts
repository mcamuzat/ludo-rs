// Mirrors games/pairs/src/ghost_town (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type GhostTownEvent =
  | { type: "Placed"; seat: number; card: Rank }
  | { type: "TieBreak"; seat: number; card: Rank }
  | { type: "Starts"; seat: number }
  | { type: "Played"; seat: number; to: number; card: Rank }
  | { type: "Drew"; seat: number; card: Rank; won: number }
  | { type: "Busted"; seat: number; card: Rank }
  | { type: "Folded"; seat: number }
  | { type: "HandEnded"; winner: number | null; pot: number }
  | { type: "Reshuffled" };

export interface GhostTownSeatView {
  house: Rank[];
  handSize: number;
  out: boolean;
  /** Balance since the start of the game; may be negative. */
  chips: number;
}

export interface GhostTownView {
  handNo: number;
  handsTotal: number;
  phase: "Setup" | "Play" | "Over";
  seats: GhostTownSeatView[];
  street: Rank[];
  hand: Rank[];
  placed: Rank | null;
  /** The seat whose house I may play into besides my own. */
  neighbour: number | null;
  turn: number | null;
  pot: number;
  ante: number;
  playCost: number;
  deck: number;
  discard: number;
  discardSeen: Rank[];
  events: GhostTownEvent[];
}

export type GhostTownAction =
  | "Draw"
  | "Fold"
  | { Place: { card: Rank } }
  | { Play: { card: Rank; to: number } };
