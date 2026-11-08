// Mirrors games/tigre-euphrate (View, Action).

export type Color = "Black" | "Red" | "Blue" | "Green";
export type Reinforcements = "English" | "Free";
export type ConflictKind = "Internal" | "External";
export type EndReason = "Treasures" | "Bag" | "Quiet";

export type Piece =
  | { Tile: { color: Color; flipped: boolean } }
  | { Leader: { seat: number; color: Color } }
  | "Catastrophe";

export type TigreAction =
  | { Leader: { color: Color; to: number } }
  | { Withdraw: { color: Color } }
  | { Tile: { color: Color; to: number } }
  | { Catastrophe: { at: number } }
  | { Swap: { tiles: number[] } }
  | "EndTurn"
  | { Reinforce: { count: number } }
  | { Resolve: { color: Color } }
  | { Monument: { square: number; monument: number } }
  | "NoMonument"
  | { LeaveTreasure: { at: number } };

export interface Conflict {
  kind: ConflictKind;
  color: Color;
  attacker: number;
  defender: number;
  attackerAt: number;
  defenderAt: number;
  attackerBase: number;
  defenderBase: number;
  attackerAdded: number | null;
}

export type PhaseView =
  | "Action"
  | { Reinforce: { conflict: Conflict; options: number[] } }
  | { ChooseConflict: { colors: Color[] } }
  | { Monument: { squares: number[] } }
  | { Treasure: { seat: number; choices: number[] } }
  | "Over";

export type TigreEvent =
  | { Tile: { seat: number; color: Color; at: number; scorer: number | null } }
  | { Junction: { seat: number; color: Color; at: number } }
  | { Leader: { seat: number; color: Color; from: number | null; to: number } }
  | { Withdraw: { seat: number; color: Color } }
  | { Catastrophe: { seat: number; at: number; destroyed: Color | null } }
  | { Swap: { seat: number; count: number } }
  | {
      Conflict: {
        kind: ConflictKind;
        color: Color;
        attacker: number;
        defender: number;
        attackerTotal: number;
        defenderTotal: number;
        winner: number;
        removed: number;
      };
    }
  | { Returned: { seat: number; color: Color } }
  | { Monument: { seat: number; monument: number; at: number } }
  | { NoMonument: { seat: number } }
  | { Treasures: { seat: number; count: number } }
  | { MonumentPoint: { seat: number; color: Color } };

export interface LastTurn {
  seat: number;
  steps: TigreEvent[];
}

export interface CellView {
  river: boolean;
  piece: Piece | null;
  treasure: boolean;
  corner: boolean;
}

export interface MonumentView {
  colors: [Color, Color];
  at: number | null;
}

export interface FinalScore {
  /** Black, red, blue, green. */
  scores: number[];
  treasures: number;
  withTreasures: number[];
  weakest: number;
  rank: number;
}

export interface TigreView {
  me: number | null;
  players: number;
  reinforcements: Reinforcements;
  /** 16 × 11, row by row from the top. */
  cells: CellView[];
  monuments: MonumentView[];
  junction: number | null;
  /** My tiles: black, red, blue, green. */
  hand: number[] | null;
  handSizes: number[];
  supply: Color[][];
  catastrophes: number[];
  myScores: number[] | null;
  treasures: number[];
  bag: number;
  treasuresLeft: number;
  turn: number | null;
  actionsLeft: number;
  deciding: number | null;
  phase: PhaseView;
  steps: TigreEvent[];
  lastTurn: LastTurn | null;
  quietLeft: number;
  end: EndReason | null;
  finalScores: FinalScore[] | null;
}
