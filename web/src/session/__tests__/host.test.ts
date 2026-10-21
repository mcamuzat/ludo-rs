import { describe, expect, it, vi } from "vitest";
import { FakeConn, peers } from "./fakePeer";
import { PROTOCOL_VERSION } from "../protocol";

vi.mock("peerjs", async () => ({ default: (await import("./fakePeer")).FakePeer }));

const { HostSession } = await import("../host");

describe("HostSession", () => {
  it("gives one connection one seat only", () => {
    peers.length = 0;
    const host = new HostSession("yavalath", "Ana", [{ kind: "open" }, { kind: "open" }], null, null);
    peers[0].emit("open");
    const conn = new FakeConn();
    conn.open = true;
    peers[0].emit("connection", conn);
    conn.emit("data", { type: "hello", protocol: PROTOCOL_VERSION, name: "Bo" });
    conn.emit("data", { type: "hello", protocol: PROTOCOL_VERSION, name: "Bo" });
    expect(host.getState().seats.map((s) => s.kind)).toEqual(["local", "remote", "open"]);
    expect(conn.sent).toContainEqual({ type: "rejected", reason: "Tu es déjà assis.", fatal: false });
    host.dispose();
  });

  it("sends a refused guest the engine's reason after the resync, so it stays shown", async () => {
    peers.length = 0;
    const host = new HostSession("yavalath", "Ana", [{ kind: "open" }], null, null);
    peers[0].emit("open");
    const conn = new FakeConn();
    conn.open = true;
    peers[0].emit("connection", conn);
    conn.emit("data", { type: "hello", protocol: PROTOCOL_VERSION, name: "Bo" });
    // A started table whose engine refuses the move.
    (host as unknown as { table: unknown }).table = {
      submit: () => Promise.reject(new Error("6 × 4 = 24 ÷ 2 = 12, pas 14")),
      update: () => ({ rev: 3 }),
      revFor: () => 3,
    };
    conn.sent.length = 0;
    conn.emit("data", { type: "action", action: "NotFound", rev: 3 });
    await new Promise((r) => setTimeout(r, 0));
    const types = conn.sent.map((m) => (m as { type: string }).type);
    expect(types.slice(-2)).toEqual(["update", "rejected"]);
    expect(conn.sent[conn.sent.length - 1]).toMatchObject({ reason: "6 × 4 = 24 ÷ 2 = 12, pas 14" });
    (host as unknown as { table: unknown }).table = null;
    host.dispose();
  });

  it("tells guests the game's options", () => {
    peers.length = 0;
    const host = new HostSession("pairs-ghost-town", "Ana", [{ kind: "open" }], null, { hands: 3 });
    peers[0].emit("open");
    const conn = new FakeConn();
    conn.open = true;
    peers[0].emit("connection", conn);
    conn.emit("data", { type: "hello", protocol: PROTOCOL_VERSION, name: "Bo" });
    expect(host.getState().config).toEqual({ hands: 3 });
    expect(conn.sent).toContainEqual(expect.objectContaining({ type: "lobby", config: { hands: 3 } }));
    host.dispose();
  });
});
