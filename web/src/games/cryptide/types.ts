// Mirrors games/cryptide (View, Action, Event, Hint).

export type Terrain = "Lake" | "Mountain" | "Forest" | "Swamp" | "Desert";
export type Animal = "Bear" | "Cougar";
export type Kind = "Stone" | "Shack";
export type StructureColor = "White" | "Green" | "Blue" | "Black";
export type Mode = "Normal" | "Advanced";
/** A space: row * 12 + column. */
export type Cell = number;

export type Clue =
  | { Terrains: [Terrain, Terrain] }
  | { NearTerrain: Terrain }
  | "NearAnimals"
  | { NearKind: Kind }
  | { NearAnimal: Animal }
  | { NearColor: StructureColor };

export interface Hint {
  clue: Clue;
  negated: boolean;
}

export interface Square {
  terrain: Terrain;
  animal: Animal | null;
  structure: [Kind, StructureColor] | null;
}

export interface Tokens {
  cube: number | null;
  discs: number[];
}

export type Phase =
  | { type: "Setup" }
  | { type: "Turn" }
  | { type: "Relocate"; search: Cell }
  | { type: "Penalty" }
  | { type: "Over" };

export type Answer = "Disc" | "Cube";
export type Why = "Setup" | "Answer" | "Search" | "Penalty";

export type CryptideEvent =
  | { type: "Placed"; seat: number; cell: Cell; answer: Answer; why: Why }
  | { type: "Asked"; by: number; target: number; cell: Cell }
  | { type: "Searched"; by: number; cell: Cell }
  | { type: "Won"; seat: number };

export interface Reveal {
  hints: Hint[];
  habitat: Cell;
}

export interface CryptideView {
  players: number;
  mode: Mode;
  squares: Square[];
  tokens: Tokens[];
  phase: Phase;
  turn: number;
  focus: Cell | null;
  my_hint: Hint | null;
  hints: Hint[];
  log: CryptideEvent[];
  reveal: Reveal | null;
}

export type CryptideAction =
  | { Cube: Cell }
  | { Ask: { cell: Cell; target: number } }
  | { Search: Cell }
  | { Disc: Cell };
