import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { peers } from "./fakePeer";

vi.mock("peerjs", async () => ({ default: (await import("./fakePeer")).FakePeer }));

const { GuestSession } = await import("../guest");

describe("GuestSession", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    peers.length = 0;
  });
  afterEach(() => vi.useRealTimers());

  function connected() {
    const session = new GuestSession("ABCDEF", "Bo");
    const peer = peers[0];
    peer.emit("open");
    const conn = peer.conns[0];
    conn.open = true;
    conn.emit("open");
    return { session, peer, conn };
  }

  it("keeps its connection to the host when only the signaling server drops", () => {
    const { session, conn } = connected();
    peers[0].emit("error", { type: "network" });
    vi.advanceTimersByTime(20_000);
    expect(peers.length).toBe(1);
    expect(conn.open).toBe(true);
    session.dispose();
  });

  it("reconnects once after losing the host, not in a loop", () => {
    const { session, conn } = connected();
    conn.close();
    vi.advanceTimersByTime(3_000);
    expect(peers.length).toBe(2);
    peers[1].emit("open");
    peers[1].conns[0].open = true;
    peers[1].conns[0].emit("open");
    vi.advanceTimersByTime(30_000);
    expect(peers.length).toBe(2);
    session.dispose();
  });

  it("shows the options the host chose", () => {
    const { session, conn } = connected();
    conn.emit("data", { type: "lobby", gameId: "pairs-ghost-town", seats: [], started: false, config: { hands: 3 } });
    expect(session.getState().config).toEqual({ hands: 3 });
    conn.emit("data", { type: "lobby", gameId: "yavalath", seats: [], started: false, config: null });
    expect(session.getState().config).toBeNull();
    session.dispose();
  });
});
