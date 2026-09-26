// Mirrors the JSON of crates/ludo-core and crates/ludo-wasm.

export type Seat = number;
export type Level = "Easy" | "Medium" | "Hard";

export interface GameMeta {
  id: string;
  name: string;
  minPlayers: number;
  maxPlayers: number;
  hiddenInfo: boolean;
}

/** One viewer's share of a snapshot (`viewer: null` = spectator). */
export interface SeatView {
  viewer: Seat | null;
  view: unknown;
  legalActions: unknown[];
}

/** The game after an action, for every viewer at once. */
export interface Snapshot {
  rev: number;
  pending: Seat[];
  /** Rank of each seat (0 = first), once the game is over. */
  outcome: number[] | null;
  views: SeatView[];
}

/** What one seat receives. */
export interface Update {
  rev: number;
  view: unknown;
  legalActions: unknown[];
  pending: Seat[];
  outcome: number[] | null;
}

export type EngineRequest =
  | { kind: "listGames" }
  | { kind: "newMatch"; gameId: string; players: number; seed: number; config: unknown }
  | { kind: "importLog"; gameId: string; log: unknown }
  | { kind: "apply"; seat: Seat; action: unknown }
  | { kind: "snapshot"; viewers: (Seat | null)[] }
  | { kind: "exportState" }
  | { kind: "exportLog" };

export interface AiRequest {
  gameId: string;
  state: unknown;
  seat: Seat;
  level: Level;
  seed: number;
}

export function updateFor(s: Snapshot, viewer: Seat | null): Update {
  const seen = s.views.find((v) => v.viewer === viewer);
  if (!seen) throw new Error(`Pas de vue pour ${viewer === null ? "le spectateur" : `le siège ${viewer}`}.`);
  return { rev: s.rev, view: seen.view, legalActions: seen.legalActions, pending: s.pending, outcome: s.outcome };
}
