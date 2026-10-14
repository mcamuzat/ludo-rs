import { describe, expect, it } from "vitest";
import { SeatRegistry, type HostSeat } from "../seats";

function tokens() {
  let n = 0;
  return () => `t${++n}`;
}

function table(...others: HostSeat[]) {
  return new SeatRegistry([{ kind: "host", name: "Hôte" }, ...others]);
}

describe("SeatRegistry", () => {
  it("seats guests in the open seats, in order", () => {
    const r = table({ kind: "open" }, { kind: "ai", level: "Easy" }, { kind: "open" });
    const next = tokens();
    expect(r.join("Ana", undefined, false, next)).toEqual({ ok: true, seat: 1, token: "t1", reconnected: false });
    expect(r.join("Bo", undefined, false, next)).toEqual({ ok: true, seat: 3, token: "t2", reconnected: false });
    expect(r.ready()).toBe(true);
  });

  it("names each AI after its seat, so they can be told apart", () => {
    const r = table({ kind: "ai", level: "Easy" }, { kind: "ai", level: "Easy" }, { kind: "ai", level: "Hard" });
    const names = r.info().map((s) => s.name);
    expect(names).toEqual(["Hôte", "Hugo (IA)", "Lina (IA)", "Oscar (IA)"]);
    expect(r.tableSeats().map((s) => s.name)).toEqual(names);
  });

  it("gives the last open seat to one guest only", () => {
    const r = table({ kind: "open" });
    const next = tokens();
    expect(r.join("Ana", undefined, false, next).ok).toBe(true);
    expect(r.join("Bo", undefined, false, next)).toEqual({ ok: false, reason: "La table est complète." });
  });

  it("gives a returning guest their seat back", () => {
    const r = table({ kind: "open" }, { kind: "open" });
    const next = tokens();
    r.join("Ana", undefined, false, next);
    r.join("Bo", undefined, false, next);
    r.leave(1, true);
    expect(r.info()[1].connected).toBe(false);
    expect(r.join("Ana", "t1", true, next)).toEqual({ ok: true, seat: 1, token: "t1", reconnected: true });
    expect(r.info()[1].connected).toBe(true);
  });

  it("refuses newcomers once the game has started", () => {
    const r = table({ kind: "open" });
    r.join("Ana", undefined, false, tokens());
    r.leave(1, true);
    expect(r.join("Eve", "forged", true, tokens())).toEqual({ ok: false, reason: "La partie a déjà commencé." });
  });

  it("frees a seat when a guest leaves the lobby", () => {
    const r = table({ kind: "open" });
    r.join("Ana", undefined, false, tokens());
    r.leave(1, false);
    expect(r.all[1]).toEqual({ kind: "open" });
  });

  it("survives a host reload: guests come back to their seats", () => {
    const r = table({ kind: "open" }, { kind: "ai", level: "Hard" });
    r.join("Ana", undefined, false, tokens());
    const restored = SeatRegistry.fromJSON(JSON.parse(JSON.stringify(r.toJSON())));
    expect(restored.info()[1]).toEqual({ name: "Ana", kind: "remote", connected: false });
    expect(restored.join("Ana", "t1", true, tokens())).toEqual({ ok: true, seat: 1, token: "t1", reconnected: true });
  });

  it("is not ready while a seat is open or a guest is away", () => {
    const r = table({ kind: "open" });
    expect(r.ready()).toBe(false);
    r.join("Ana", undefined, false, tokens());
    expect(r.ready()).toBe(true);
    r.leave(1, true);
    expect(r.ready()).toBe(false);
  });

  it("lets the host change only free seats", () => {
    const r = table({ kind: "open" }, { kind: "open" });
    r.join("Ana", undefined, false, tokens());
    expect(r.setSeat(0, { kind: "open" })).toBe(false);
    expect(r.setSeat(1, { kind: "ai", level: "Easy" })).toBe(false);
    expect(r.setSeat(2, { kind: "ai", level: "Easy" })).toBe(true);
    expect(r.tableSeats().map((s) => s.kind)).toEqual(["local", "remote", "ai"]);
  });

  it("cleans guest names", () => {
    const r = table({ kind: "open" });
    r.join("   ", undefined, false, tokens());
    expect(r.info()[1].name).toBe("Invité");
  });
});
