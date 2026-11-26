// Mirrors games/tigre-euphrate-cartes (View, Action).

export type Color = "Black" | "Red" | "Blue" | "Green";
export type ConflictKind = "Internal" | "External";
export type EndReason = "Treasures" | "Deck" | "Quiet";

export type Pos = { Head: number } | { Gap: number } | { Card: [number, number] };
export type Face = { Civ: Color } | "Treasure" | "Exchanged" | { Ship: Color };

export interface Leader {
  seat: number;
  color: Color;
}

export interface Slot {
  face: Face;
  leader: Leader | null;
}

export type TecAction =
  | { Leader: { color: Color; to: Pos } }
  | { Extend: { color: Color; column: number } }
  | { Join: { color: Color; gap: number } }
  | { Catastrophe: { at: Pos } }
  | "EndTurn"
  | { Commit: { count: number } }
  | { Resolve: { color: Color } }
  | { Ship: { ship: Color } }
  | "NoShip";

export interface Conflict {
  kind: ConflictKind;
  color: Color;
  attacker: number;
  defender: number;
  attackerBase: number;
  defenderBase: number;
  attackerAdded: number | null;
}

export type PhaseView =
  | "Action"
  | { Conflict: { conflict: Conflict; most: number | null } }
  | { ChooseConflict: { colors: Color[] } }
  | { Ship: { column: number; options: Color[] } }
  | "Over";

export type TecEvent =
  | { Leader: { seat: number; color: Color; moved: boolean; to: Pos } }
  | { Extend: { seat: number; color: Color; column: number; scorer: number | null } }
  | { Join: { seat: number; color: Color; gap: number } }
  | { Catastrophe: { seat: number; at: Pos; color: Color } }
  | {
      Conflict: {
        kind: ConflictKind;
        color: Color;
        attacker: number;
        defender: number;
        attackerTotal: number;
        defenderTotal: number;
        winner: number;
        scored: boolean;
        removed: number;
      };
    }
  | { Ship: { seat: number; ship: Color; column: number } }
  | { Returned: { seat: number; color: Color } }
  | { Treasure: { seat: number; column: number } }
  | { ShipPoint: { seat: number; color: Color } };

export interface LastTurn {
  seat: number;
  steps: TecEvent[];
}

export interface FinalScore {
  scores: number[];
  treasures: number;
  withTreasures: number[];
  weakest: number;
  rank: number;
}

export interface TecView {
  me: number | null;
  players: number;
  heads: Slot[];
  gaps: (Slot | null)[];
  columns: Slot[][];
  join: number | null;
  hand: number[] | null;
  handSizes: number[];
  supply: Color[][];
  catastrophes: boolean[];
  scores: number[][];
  treasures: number[];
  ships: Color[];
  deck: number;
  treasuresLeft: number;
  turn: number | null;
  deciding: number | null;
  actionsLeft: number;
  phase: PhaseView;
  quietLeft: number;
  steps: TecEvent[];
  lastTurn: LastTurn | null;
  end: EndReason | null;
  finalScores: FinalScore[] | null;
}
