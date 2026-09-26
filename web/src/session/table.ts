import { AiClient, EngineClient, randomSeed } from "../engine/clients";
import type { Level, Seat, Snapshot } from "../engine/types";
import { errorMessage } from "./names";
import { StaleActionError, isStale } from "./rev";

export type TableSeat =
  | { kind: "local"; name: string }
  | { kind: "ai"; level: Level; name: string }
  | { kind: "remote"; name: string };

interface TableHooks {
  onSnapshot(s: Snapshot): void;
  /** After each snapshot: the match log, for saving. */
  onLog?(log: unknown, s: Snapshot): void;
  onError(message: string): void;
}

/** AI moves are shown after at least this long, so humans can follow. */
const AI_MIN_DELAY_MS = 400;

/**
 * The authoritative game: engine + AI seats. Used by local play and by the
 * host. Engine calls are serialized, and every change produces a snapshot
 * for all seats.
 */
export class Table {
  private engine = new EngineClient();
  private ai = new AiClient();
  private queue: Promise<unknown> = Promise.resolve();
  private thinking = new Set<Seat>();
  private disposed = false;
  snapshot: Snapshot | null = null;

  constructor(
    readonly gameId: string,
    readonly seats: TableSeat[],
    private hooks: TableHooks,
  ) {}

  start(seed: number, config: unknown = null): Promise<void> {
    return this.serial(async () => {
      await this.engine.newMatch(this.gameId, this.seats.length, seed, config);
      await this.refresh();
    });
  }

  resume(log: unknown): Promise<void> {
    return this.serial(async () => {
      await this.engine.importLog(this.gameId, log);
      await this.refresh();
    });
  }

  /**
   * Applies `action` for `seat`. Rejects with the engine's message when it
   * is illegal, or with `StaleActionError` when `rev` is given and the game
   * has moved on since.
   */
  submit(seat: Seat, action: unknown, rev?: number): Promise<void> {
    return this.serial(async () => {
      if (rev !== undefined && this.snapshot && isStale(rev, this.snapshot.rev)) throw new StaleActionError();
      await this.engine.apply(seat, action);
      await this.refresh();
    });
  }

  dispose() {
    this.disposed = true;
    this.engine.dispose();
    this.ai.dispose();
  }

  private serial<T>(task: () => Promise<T>): Promise<T> {
    const run = this.queue.then(task);
    this.queue = run.catch(() => undefined);
    return run;
  }

  private async refresh() {
    const viewers: (Seat | null)[] = [...this.seats.keys(), null];
    const s = await this.engine.snapshot(viewers);
    if (this.disposed) return;
    this.snapshot = s;
    this.hooks.onSnapshot(s);
    if (this.hooks.onLog) this.hooks.onLog(await this.engine.exportLog(), s);
    this.runAi(s);
  }

  private runAi(s: Snapshot) {
    for (const seat of s.pending) {
      const setup = this.seats[seat];
      if (setup?.kind !== "ai" || this.thinking.has(seat)) continue;
      this.thinking.add(seat);
      void this.playAi(seat, setup.level);
    }
  }

  private async playAi(seat: Seat, level: Level) {
    const started = performance.now();
    try {
      const { state, rev } = await this.serial(async () => ({
        state: await this.engine.exportState(),
        rev: this.snapshot?.rev ?? 0,
      }));
      const action = await this.ai.move({ gameId: this.gameId, state, seat, level, seed: randomSeed() });
      await sleep(AI_MIN_DELAY_MS - (performance.now() - started));
      if (this.disposed) return;
      this.thinking.delete(seat);
      await this.submit(seat, action, rev);
    } catch (e) {
      this.thinking.delete(seat);
      if (this.disposed) return;
      // Computed on an old state: the latest snapshot decides whether this seat still has to play.
      if (e instanceof StaleActionError) {
        if (this.snapshot) this.runAi(this.snapshot);
      } else {
        this.hooks.onError(errorMessage(e));
      }
    }
  }
}

function sleep(ms: number) {
  return new Promise<void>((resolve) => setTimeout(resolve, Math.max(0, ms)));
}
