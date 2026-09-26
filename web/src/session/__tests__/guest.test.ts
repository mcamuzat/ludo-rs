import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

type Handler = (...args: unknown[]) => void;

class Emitter {
  private handlers = new Map<string, Handler[]>();
  on(event: string, handler: Handler) {
    this.handlers.set(event, [...(this.handlers.get(event) ?? []), handler]);
  }
  off(event: string) {
    this.handlers.delete(event);
  }
  removeAllListeners() {
    this.handlers.clear();
  }
  emit(event: string, ...args: unknown[]) {
    for (const h of this.handlers.get(event) ?? []) h(...args);
  }
}

class FakeConn extends Emitter {
  open = false;
  sent: unknown[] = [];
  send(msg: unknown) {
    this.sent.push(msg);
  }
  close() {
    this.open = false;
    this.emit("close");
  }
}

const peers: FakePeer[] = [];

class FakePeer extends Emitter {
  destroyed = false;
  conns: FakeConn[] = [];
  constructor() {
    super();
    peers.push(this);
  }
  connect() {
    const conn = new FakeConn();
    this.conns.push(conn);
    return conn;
  }
  reconnect() {}
  /** Like PeerJS 1.5: destroying closes every connection, which emits "close" synchronously. */
  destroy() {
    this.destroyed = true;
    for (const conn of this.conns) conn.close();
  }
}

vi.mock("peerjs", () => ({ default: FakePeer }));

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
});
