// Mirrors games/pairs/src/auberge (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type AubergeEvent =
  | { type: "Placed"; seat: number; card: Rank }
  | { type: "Starts"; seat: number }
  | { type: "Drew"; seat: number }
  | { type: "Attacked"; seat: number; target: number; card: Rank }
  | { type: "Dared"; seat: number; target: number; card: Rank; paired: boolean }
  /** `hand` is empty unless the viewer is the accuser or the accused. */
  | { type: "Accused"; seat: number; target: number; guilty: boolean; hand: Rank[] }
  | { type: "Out"; seat: number }
  | { type: "RoundWon"; seat: number | null }
  | { type: "Reshuffled" };

export interface AubergeSeatView {
  front: Rank[];
  handSize: number;
  out: boolean;
  wins: number;
}

export interface AubergeView {
  round: number;
  rounds: number;
  phase: "Setup" | "Play" | "Over";
  seats: AubergeSeatView[];
  hand: Rank[];
  placed: Rank | null;
  ownPair: Rank | null;
  turn: number | null;
  deck: number;
  discard: number;
  discardSeen: Rank[];
  events: AubergeEvent[];
}

export type AubergeAction =
  | "Draw"
  | { Place: { card: Rank } }
  | { Attack: { target: number; card: Rank } }
  | { Dare: { target: number } }
  | { Accuse: { target: number } };
