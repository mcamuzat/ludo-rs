import { describe, expect, it, vi } from "vitest";
import { FakeConn, peers } from "./fakePeer";
import { PROTOCOL_VERSION } from "../protocol";

vi.mock("peerjs", async () => ({ default: (await import("./fakePeer")).FakePeer }));

const { HostSession } = await import("../host");

describe("HostSession", () => {
  it("gives one connection one seat only", () => {
    peers.length = 0;
    const host = new HostSession("yavalath", "Ana", [{ kind: "open" }, { kind: "open" }], null);
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
});
