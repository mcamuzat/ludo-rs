// Mirrors games/pairs/src/monster (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type MonsterEvent =
  | { type: "GameStarted"; game: number; button: number; ante: number }
  | { type: "Dealt"; round: number; seats: number[] }
  | { type: "Discarded"; seat: number; count: number }
  | { type: "Folded"; seat: number }
  | {
      type: "Showdown";
      /** null: folded or not in the round. */
      hands: (Rank[] | null)[];
      monster: Rank[];
      champions: number[];
      won: boolean;
    }
  | { type: "Penalty"; seats: number[] }
  | { type: "PotWon"; seats: number[]; share: number }
  | { type: "Replay" };

export interface MonsterSeatView {
  inGame: boolean;
  inRound: boolean;
  handSize: number;
  acted: boolean;
  folded: boolean;
  discards: number;
  /** Balance since the start of the match; may be negative. */
  chips: number;
}

export interface MonsterView {
  gameNo: number;
  gamesTotal: number;
  roundNo: number;
  phase: "Act" | "Over";
  seats: MonsterSeatView[];
  hand: Rank[];
  /** What I gave the Monster this round. */
  discarded: Rank[];
  monsterSize: number;
  button: number;
  turn: number | null;
  pot: number;
  events: MonsterEvent[];
}

export type MonsterAction = { Discard: { cards: Rank[] } } | "Fold";
