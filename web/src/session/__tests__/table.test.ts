import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AiRequest, Seat, Snapshot } from "../../engine/types";
import { StaleActionError } from "../rev";
import { AI_MIN_DELAY_MS, Table, type AiLike, type EngineLike, type TableSeat } from "../table";

/** Two seats taking turns; each has `legal` to choose from; "t" is the default. */
class FakeEngine implements EngineLike {
  rev = 0;
  current: Seat = 0;
  legal: unknown[] = ["a", "b"];
  applied: [Seat, unknown][] = [];
  async newMatch() {}
  async importLog() {}
  async apply(seat: Seat, action: unknown) {
    if (seat !== this.current) throw new Error("pas ton tour");
    this.applied.push([seat, action]);
    this.rev++;
    this.current = 1 - this.current;
  }
  async snapshot(viewers: (Seat | null)[]): Promise<Snapshot> {
    return {
      rev: this.rev,
      pending: [this.current],
      outcome: null,
      views: viewers.map((viewer) => ({
        viewer,
        view: {},
        legalActions: viewer === this.current ? this.legal : [],
        pending: [this.current],
      })),
    };
  }
  async exportState() {
    return { rev: this.rev };
  }
  async exportLog() {
    return [];
  }
  async timeoutAction(seat: Seat) {
    return seat === this.current ? "t" : null;
  }
  dispose() {}
}

/** Seats 0 and 1 answer at the same time; the game moves on once both have. */
class SimultaneousEngine extends FakeEngine {
  answered = new Set<Seat>();
  override async apply(seat: Seat, action: unknown) {
    if (this.answered.has(seat)) throw new Error("déjà répondu");
    this.applied.push([seat, action]);
    this.rev++;
    this.answered.add(seat);
    if (this.answered.size === 2) this.answered.clear();
  }
  override async snapshot(viewers: (Seat | null)[]): Promise<Snapshot> {
    const pending = [0, 1].filter((x) => !this.answered.has(x));
    return {
      rev: this.rev,
      pending,
      outcome: null,
      views: viewers.map((viewer) => ({
        viewer,
        view: { answered: viewer !== null && this.answered.has(viewer), round: this.applied.length - this.answered.size },
        legalActions: viewer !== null && pending.includes(viewer) ? this.legal : [],
        // Like a call window: each seat only sees itself.
        pending: viewer !== null && pending.includes(viewer) ? [viewer] : [],
      })),
    };
  }
}

class FakeAi implements AiLike {
  constructor(private impl: (req: AiRequest) => Promise<unknown>) {}
  move(req: AiRequest) {
    return this.impl(req);
  }
  dispose() {}
}

const hooks = () => ({ onSnapshot: vi.fn(), onError: vi.fn() });
const remote = (name: string): TableSeat => ({ kind: "remote", name });

describe("Table", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("plays a lone legal action after a short delay", async () => {
    const engine = new FakeEngine();
    engine.legal = ["only"];
    const table = new Table("g", [remote("A"), remote("B")], hooks(), { engine, ai: new FakeAi(async () => "x"), now: Date.now });
    await table.start(1);
    await vi.advanceTimersByTimeAsync(299);
    expect(engine.applied).toEqual([]);
    await vi.advanceTimersByTimeAsync(2);
    expect(engine.applied[0]).toEqual([0, "only"]);
    table.dispose();
  });

  it("plays the default action when the clock runs out", async () => {
    const engine = new FakeEngine();
    const table = new Table("g", [remote("A"), remote("B")], hooks(), {
      engine,
      ai: new FakeAi(async () => "x"),
      clock: { baseMs: 1000, bankMs: 500 },
      now: Date.now,
    });
    await table.start(1);
    await vi.advanceTimersByTimeAsync(1499);
    expect(engine.applied).toEqual([]);
    await vi.advanceTimersByTimeAsync(2);
    expect(engine.applied).toEqual([[0, "t"]]);
    expect(table.banks()).toEqual([[0, 0]]);
    const [next] = table.clockViews();
    expect(next.seat).toBe(1);
    expect(next.bankMs).toBe(500);
    expect(next.baseMs).toBeGreaterThan(990);
    table.dispose();
  });

  it("refuses an answer that arrives after the default was played", async () => {
    const engine = new FakeEngine();
    const table = new Table("g", [remote("A"), remote("B")], hooks(), {
      engine,
      ai: new FakeAi(async () => "x"),
      clock: { baseMs: 1000, bankMs: 0 },
      now: Date.now,
    });
    await table.start(1);
    const rev = table.update(0)!.rev;
    await vi.advanceTimersByTimeAsync(1001);
    await expect(table.submit(0, "a", rev)).rejects.toBeInstanceOf(StaleActionError);
    expect(engine.applied).toEqual([[0, "t"]]);
    table.dispose();
  });

  it("does not time the AI seats", async () => {
    const engine = new FakeEngine();
    const table = new Table("g", [{ kind: "ai", level: "Easy", name: "IA" }, remote("B")], hooks(), {
      engine,
      ai: new FakeAi(() => new Promise(() => {})),
      clock: { baseMs: 1000, bankMs: 0 },
      now: Date.now,
    });
    await table.start(1);
    await vi.advanceTimersByTimeAsync(5000);
    expect(engine.applied).toEqual([]);
    expect(table.clockViews()).toEqual([]);
    table.dispose();
  });

  it("shows an AI move only after the minimum delay, however fast it thinks", async () => {
    const engine = new FakeEngine();
    const table = new Table("g", [{ kind: "ai", level: "Easy", name: "IA" }, remote("B")], hooks(), {
      engine,
      ai: new FakeAi(async () => "b"),
      now: Date.now,
    });
    await table.start(1);
    await vi.advanceTimersByTimeAsync(AI_MIN_DELAY_MS - 1);
    expect(engine.applied).toEqual([]);
    await vi.advanceTimersByTimeAsync(2);
    expect(engine.applied).toEqual([[0, "b"]]);
    table.dispose();
  });

  it("an AI that fails twice plays the default action and reports it", async () => {
    const engine = new FakeEngine();
    const h = hooks();
    let calls = 0;
    const ai = new FakeAi(async () => {
      calls++;
      throw new Error("recherche impossible");
    });
    const table = new Table("g", [{ kind: "ai", level: "Easy", name: "IA" }, remote("B")], h, { engine, ai, now: Date.now });
    await table.start(1);
    await vi.advanceTimersByTimeAsync(2000);
    expect(calls).toBe(2);
    expect(engine.applied).toEqual([[0, "t"]]);
    expect(h.onError).toHaveBeenCalledTimes(1);
    table.dispose();
  });

  it("an AI that fails once tries again and plays its move", async () => {
    const engine = new FakeEngine();
    let calls = 0;
    const ai = new FakeAi(async () => {
      calls++;
      if (calls === 1) throw new Error("hoquet");
      return "b";
    });
    const table = new Table("g", [{ kind: "ai", level: "Easy", name: "IA" }, remote("B")], hooks(), { engine, ai, now: Date.now });
    await table.start(1);
    await vi.advanceTimersByTimeAsync(2000);
    expect(engine.applied).toEqual([[0, "b"]]);
    table.dispose();
  });

  it("another seat's answer neither shows in my updates nor makes my answer stale", async () => {
    const engine = new SimultaneousEngine();
    const table = new Table("g", [remote("A"), remote("B"), remote("C")], hooks(), { engine, ai: new FakeAi(async () => "x"), now: Date.now });
    await table.start(1);
    const mine = table.update(0)!;
    const bystander = table.update(2)!;
    await table.submit(1, "a", table.update(1)!.rev);
    expect(table.update(0)).toEqual(mine);
    expect(table.update(2)).toEqual(bystander);
    await table.submit(0, "b", mine.rev);
    expect(engine.applied).toEqual([
      [1, "a"],
      [0, "b"],
    ]);
    expect(table.update(0)!.rev).not.toBe(mine.rev);
    table.dispose();
  });
});
