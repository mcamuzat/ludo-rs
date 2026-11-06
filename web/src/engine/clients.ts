import { WorkerRpc } from "./rpc";
import type { AiRequest, AiWorkerRequest, EngineRequest, GameMeta, Seat, Snapshot } from "./types";

export class EngineClient {
  private rpc = new WorkerRpc<EngineRequest>(
    new Worker(new URL("./engine.worker.ts", import.meta.url), { type: "module" }),
  );

  listGames() {
    return this.rpc.call<GameMeta[]>({ kind: "listGames" });
  }

  newMatch(gameId: string, players: number, seed: number, config: unknown = null) {
    return this.rpc.call<null>({ kind: "newMatch", gameId, players, seed, config });
  }

  importLog(gameId: string, log: unknown) {
    return this.rpc.call<null>({ kind: "importLog", gameId, log });
  }

  /** Rejects with the engine's message when the action is illegal. */
  apply(seat: Seat, action: unknown) {
    return this.rpc.call<null>({ kind: "apply", seat, action });
  }

  snapshot(viewers: (Seat | null)[]) {
    return this.rpc.call<Snapshot>({ kind: "snapshot", viewers });
  }

  exportState() {
    return this.rpc.call<unknown>({ kind: "exportState" });
  }

  exportLog() {
    return this.rpc.call<unknown>({ kind: "exportLog" });
  }

  /** The action the host plays when `seat`'s time runs out, or null. */
  timeoutAction(seat: Seat) {
    return this.rpc.call<unknown | null>({ kind: "timeoutAction", seat });
  }

  /** Rewinds to just before `seat`'s last action. */
  takeBack(seat: Seat) {
    return this.rpc.call<null>({ kind: "takeBack", seat });
  }

  /** A Cryptide riddle (games/cryptide/src/riddle.rs); needs no match. */
  cryptideRiddle<T>(mode: string, players: number, seed: number) {
    return this.rpc.call<T>({ kind: "cryptideRiddle", mode, players, seed });
  }

  dispose() {
    this.rpc.dispose();
  }
}

function aiWorker() {
  return new WorkerRpc<AiWorkerRequest>(new Worker(new URL("./ai.worker.ts", import.meta.url), { type: "module" }));
}

export class AiClient {
  private rpc = aiWorker();

  move(req: AiRequest) {
    return this.rpc.call<unknown>({ kind: "move", ...req });
  }

  dispose() {
    this.rpc.dispose();
  }
}

/**
 * TwixT's coach: its own AI worker, so a question never waits behind the
 * AI's move. Answers are the engine's JSON (games/twixt/src/coach.rs).
 */
export class CoachClient {
  private rpc = aiWorker();

  advise<T>(moves: unknown[]) {
    return this.rpc.call<T>({ kind: "twixtAdvise", moves });
  }

  reviewPoint<T>(moves: unknown[], ply: number) {
    return this.rpc.call<T>({ kind: "twixtReviewPoint", moves, ply });
  }

  review<T>(moves: unknown[], points: unknown[]) {
    return this.rpc.call<T>({ kind: "twixtReview", moves, points });
  }

  position<T>(moves: unknown[]) {
    return this.rpc.call<T>({ kind: "twixtPosition", moves });
  }

  dispose() {
    this.rpc.dispose();
  }
}

export function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** The games the engine offers (asked from a short-lived worker). */
export async function loadCatalog(): Promise<GameMeta[]> {
  const engine = new EngineClient();
  try {
    return await engine.listGames();
  } finally {
    engine.dispose();
  }
}
