import { AiClient, EngineClient, randomSeed } from "../engine/clients";
import { updateFor, type AiRequest, type Level, type Seat, type SeatClockView, type Snapshot, type Update } from "../engine/types";
import { Clocks, type ClockSettings } from "./clock";
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

/** What `Table` needs from the engine worker (a fake in tests). */
export interface EngineLike {
  newMatch(gameId: string, players: number, seed: number, config?: unknown): Promise<unknown>;
  importLog(gameId: string, log: unknown): Promise<unknown>;
  apply(seat: Seat, action: unknown): Promise<unknown>;
  snapshot(viewers: (Seat | null)[]): Promise<Snapshot>;
  exportState(): Promise<unknown>;
  exportLog(): Promise<unknown>;
  timeoutAction(seat: Seat): Promise<unknown | null>;
  dispose(): void;
}

export interface AiLike {
  move(req: AiRequest): Promise<unknown>;
  dispose(): void;
}

export interface TableOptions {
  /** Decision clock for human seats (online play). */
  clock?: ClockSettings | null;
  /** Saved banks, when resuming. */
  banks?: [Seat, number][];
  engine?: EngineLike;
  ai?: AiLike;
  now?: () => number;
}

/**
 * AI moves are shown after at least this long (from when the AI starts
 * thinking), so humans can follow a run of AI moves.
 */
export const AI_MIN_DELAY_MS = 1000;
/** A seat with a single legal action plays it after this long. */
export const AUTO_PLAY_MS = 300;

/**
 * The authoritative game: engine, AI seats, clocks. Used by local play and by
 * the host. Engine calls are serialized, and every change produces a
 * snapshot for all seats.
 */
export class Table {
  private engine: EngineLike;
  private ai: AiLike;
  private clocks: Clocks | null;
  private now: () => number;
  private queue: Promise<unknown> = Promise.resolve();
  private thinking = new Set<Seat>();
  private timers = new Set<ReturnType<typeof setTimeout>>();
  private disposed = false;
  snapshot: Snapshot | null = null;
  /**
   * One revision per viewer, moved only when what that viewer receives
   * changes: a seat never learns from a revision that others were asked or
   * answered, and another seat's answer never makes its own answer stale.
   * Starts at a random value, so an action meant for a previous table is stale.
   */
  private revs = new Map<Seat | null, { rev: number; key: string }>();
  private firstRev = randomSeed();

  constructor(
    readonly gameId: string,
    readonly seats: TableSeat[],
    private hooks: TableHooks,
    options: TableOptions = {},
  ) {
    this.engine = options.engine ?? new EngineClient();
    this.ai = options.ai ?? new AiClient();
    this.clocks = options.clock ? new Clocks(options.clock, options.banks) : null;
    this.now = options.now ?? (() => performance.now());
  }

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
      if (rev !== undefined && isStale(rev, this.revFor(seat))) throw new StaleActionError();
      await this.engine.apply(seat, action);
      this.clocks?.finish(seat, this.now());
      await this.refresh();
    });
  }

  /** The revision of what `viewer` currently receives. */
  revFor(viewer: Seat | null): number {
    return this.revs.get(viewer)?.rev ?? this.firstRev;
  }

  /** What `viewer` receives now, stamped with its own revision. */
  update(viewer: Seat | null): Update | null {
    if (!this.snapshot) return null;
    return { ...updateFor(this.snapshot, viewer, this.clockViews()), rev: this.revFor(viewer) };
  }

  /** Time left for every timed seat that is deciding. */
  clockViews(): SeatClockView[] {
    return this.clocks?.views(this.now()) ?? [];
  }

  /** Remaining banks, to save with the game. */
  banks(): [Seat, number][] {
    return this.clocks?.toJSON() ?? [];
  }

  dispose() {
    this.disposed = true;
    for (const t of this.timers) clearTimeout(t);
    this.timers.clear();
    this.engine.dispose();
    this.ai.dispose();
  }

  private serial<T>(task: () => Promise<T>): Promise<T> {
    const run = this.queue.then(task);
    this.queue = run.catch(() => undefined);
    return run;
  }

  private later(ms: number, task: () => void) {
    const timer = setTimeout(() => {
      this.timers.delete(timer);
      if (!this.disposed) task();
    }, Math.max(0, ms));
    this.timers.add(timer);
  }

  private async refresh() {
    const viewers: (Seat | null)[] = [...this.seats.keys(), null];
    const s = await this.engine.snapshot(viewers);
    if (this.disposed) return;
    this.snapshot = s;
    this.stampRevisions(s);
    for (const t of this.timers) clearTimeout(t);
    this.timers.clear();
    this.syncClocks(s);
    this.hooks.onSnapshot(s);
    if (this.hooks.onLog) this.hooks.onLog(await this.engine.exportLog(), s);
    this.autoPlay(s);
    this.runAi(s);
  }

  private stampRevisions(s: Snapshot) {
    for (const v of s.views) {
      const key = JSON.stringify([v.view, v.legalActions, v.pending, s.outcome]);
      const previous = this.revs.get(v.viewer);
      if (previous?.key !== key) this.revs.set(v.viewer, { rev: (previous?.rev ?? this.firstRev) + 1, key });
    }
  }

  private humanPending(s: Snapshot): Seat[] {
    return s.pending.filter((seat) => this.seats[seat]?.kind !== "ai");
  }

  private syncClocks(s: Snapshot) {
    if (!this.clocks) return;
    const now = this.now();
    const humans = this.humanPending(s);
    this.clocks.sync(humans, now);
    for (const seat of humans) {
      const deadline = this.clocks.deadline(seat);
      if (deadline !== null) {
        const rev = this.revFor(seat);
        this.later(deadline - now, () => void this.expire(seat, rev));
      }
    }
  }

  /** The seat's time ran out: the game's default action, through the normal path. */
  private async expire(seat: Seat, rev: number) {
    try {
      const action = await this.serial(() => this.engine.timeoutAction(seat));
      if (action !== null) await this.submit(seat, action, rev);
    } catch (e) {
      if (!(e instanceof StaleActionError)) this.hooks.onError(errorMessage(e));
    }
  }

  /** A human seat with a single legal action plays it. */
  private autoPlay(s: Snapshot) {
    for (const seat of this.humanPending(s)) {
      const legal = s.views.find((v) => v.viewer === seat)?.legalActions ?? [];
      if (legal.length !== 1) continue;
      const rev = this.revFor(seat);
      this.later(AUTO_PLAY_MS, () => {
        this.submit(seat, legal[0], rev).catch((e) => {
          if (!(e instanceof StaleActionError)) this.hooks.onError(errorMessage(e));
        });
      });
    }
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
    const started = this.now();
    for (let attempt = 1; ; attempt++) {
      try {
        const { state, rev } = await this.serial(async () => ({
          state: await this.engine.exportState(),
          rev: this.revFor(seat),
        }));
        const action = await this.ai.move({ gameId: this.gameId, state, seat, level, seed: randomSeed() });
        await sleep(AI_MIN_DELAY_MS - (this.now() - started));
        if (this.disposed) return;
        this.thinking.delete(seat);
        await this.submit(seat, action, rev);
        return;
      } catch (e) {
        if (this.disposed) return;
        if (e instanceof StaleActionError) {
          // Computed on an old state: the latest snapshot decides whether this seat still has to play.
          this.thinking.delete(seat);
          if (this.snapshot) this.runAi(this.snapshot);
          return;
        }
        if (attempt < 2) continue;
        this.thinking.delete(seat);
        this.hooks.onError(`${this.seats[seat].name} : ${errorMessage(e)}`);
        await this.playFallback(seat);
        return;
      }
    }
  }

  /** The AI could not decide: the game's default action, else its first legal action. */
  private async playFallback(seat: Seat) {
    const rev = this.revFor(seat);
    const fallback =
      (await this.serial(() => this.engine.timeoutAction(seat))) ??
      this.snapshot?.views.find((v) => v.viewer === seat)?.legalActions[0];
    if (fallback === undefined || fallback === null) return;
    await this.submit(seat, fallback, rev).catch(() => undefined);
  }
}

function sleep(ms: number) {
  return new Promise<void>((resolve) => setTimeout(resolve, Math.max(0, ms)));
}
