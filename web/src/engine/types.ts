// Mirrors the JSON of crates/ludo-core and crates/ludo-wasm.

export type Seat = number;
export type Level = "Easy" | "Medium" | "Hard";

/** A setting the lobby offers (`ludo_core::OptionKind`). */
export type OptionKind =
  | { type: "toggle"; default: boolean }
  | { type: "choice"; values: string[]; default: string }
  | { type: "number"; min: number; max: number; default: number };

export interface GameOption {
  /** A field of the game's `Config`. */
  key: string;
  kind: OptionKind;
}

/** A game's `Config`, as sent to `newMatch`. */
export type GameConfig = Record<string, boolean | string | number>;

export interface GameMeta {
  id: string;
  name: string;
  minPlayers: number;
  maxPlayers: number;
  hiddenInfo: boolean;
  clock: boolean;
  options: GameOption[];
}

/** One viewer's share of a snapshot (`viewer: null` = spectator). */
export interface SeatView {
  viewer: Seat | null;
  view: unknown;
  legalActions: unknown[];
  /** The pending seats this viewer may know about. */
  pending: Seat[];
}

/** Time left for a seat's current decision, when it was sent. */
export interface SeatClockView {
  seat: Seat;
  baseMs: number;
  bankMs: number;
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
  clocks: SeatClockView[];
}

export type EngineRequest =
  | { kind: "listGames" }
  | { kind: "newMatch"; gameId: string; players: number; seed: number; config: unknown }
  | { kind: "importLog"; gameId: string; log: unknown }
  | { kind: "apply"; seat: Seat; action: unknown }
  | { kind: "snapshot"; viewers: (Seat | null)[] }
  | { kind: "exportState" }
  | { kind: "exportLog" }
  | { kind: "timeoutAction"; seat: Seat };

export interface AiRequest {
  gameId: string;
  state: unknown;
  seat: Seat;
  level: Level;
  seed: number;
}

export function updateFor(s: Snapshot, viewer: Seat | null, clocks: SeatClockView[] = []): Update {
  const seen = s.views.find((v) => v.viewer === viewer);
  if (!seen) throw new Error(`Pas de vue pour ${viewer === null ? "le spectateur" : `le siège ${viewer}`}.`);
  return {
    rev: s.rev,
    view: seen.view,
    legalActions: seen.legalActions,
    pending: seen.pending,
    outcome: s.outcome,
    clocks: clocks.filter((c) => seen.pending.includes(c.seat)),
  };
}
