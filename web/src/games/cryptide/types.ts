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

/** What a riddle asks (riddle.rs). */
export type RiddleLevel = "Warmup" | "Hint" | "Easy" | "Full";

/** The token that rules a hint out: a disc outside it, or a cube inside. */
export type Mark = "Disc" | "Cube";

export interface Elimination {
  hint: Hint;
  cell: Cell;
  mark: Mark;
}

/** What a combination of hints gives on the spaces without a cube. */
export type Outcome = { Habitat: Cell } | "Nowhere" | { Several: number } | "Repeated";

export interface Combination {
  /** One hint per seat, the solver's included. */
  hints: Hint[];
  outcome: Outcome;
}

export interface Combinations {
  working: Combination[];
  /** A few fallen ones, as examples. */
  fallen: Combination[];
  nowhere: number;
  several: number;
  repeated: number;
  complete: boolean;
}

/** A riddle of one level: a map, maybe a table of AIs stopped at the right moment (riddle.rs). */
export interface Riddle {
  level: RiddleLevel;
  players: number;
  mode: Mode;
  squares: Square[];
  tokens: Tokens[];
  log: CryptideEvent[];
  /** The seat in focus: whose hint is given (Easy, Full) or sought (Hint); 0 at the warm-up. */
  solver: number;
  hint: Hint;
  habitat: Cell;
  hints: Hint[];
  /** The hints shown to the player, by seat. */
  given: (Hint | null)[];
  /** By seat, the spaces its hint keeps. */
  hint_cells: Cell[][];
  /** What the solver can tell of each other seat's hint. */
  deduction: Hint[][];
  /** Level Hint: a token ruling out each other hint of the mode. */
  eliminations: Elimination[];
  /** Level Full: the combinations and why they work or fall. */
  combinations: Combinations;
  mode_hints: Hint[];
}

export type CryptideAction =
  | { Cube: Cell }
  | { Ask: { cell: Cell; target: number } }
  | { Search: Cell }
  | { Disc: Cell };
