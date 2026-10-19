// Mirrors games/monad/src (View, Action).

export type Color = "Red" | "Orange" | "Yellow" | "Navy" | "Sky" | "Green";
export type Rank = "Unit" | "Bi" | "Tri" | "Quad" | "Quint";

export interface Card {
  rank: Rank;
  color: Color;
}

export type Target = { Card: Rank } | "Monad";

export type MonadAction =
  | { Trade: { cards: [Card, Card] } }
  | { Buy: { target: Target; cards: Card[] } }
  | { Leap: { colors: Color[] } }
  | "TakeBonus"
  | "DeclineBonus"
  | "EndTurn"
  | "Draw"
  | "Flip"
  | "Pass";

export interface Turn {
  transacted: boolean;
  prepared: boolean;
  bonusUsed: boolean;
  bonusOffer: Rank | null;
}

export interface EventView {
  seat: number;
  action: MonadAction;
  gained: Card[];
  monad: boolean;
  drew: boolean;
  drawn: Color | null;
}

export interface MonadView {
  players: number;
  target: number;
  identities: Color[];
  monads: number[];
  monadStock: number;
  columns: Color[][];
  discard: Color[];
  deck: number;
  hand: Card[] | null;
  handCounts: number[][];
  current: number | null;
  turn: Turn;
  thisTurn: EventView[];
  lastTurn: EventView[];
  winner: number | null;
  stalled: boolean;
  ranks: number[] | null;
}
