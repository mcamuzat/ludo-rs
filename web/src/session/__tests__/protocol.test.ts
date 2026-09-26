import { describe, expect, it } from "vitest";
import { PROTOCOL_VERSION, newCode, normalizeCode, parseGuestMessage, parseHostMessage } from "../protocol";

describe("parseGuestMessage", () => {
  it("rejects garbage", () => {
    for (const bad of [
      null,
      42,
      "hello",
      {},
      { type: "x" },
      { type: "action", action: 1 },
      { type: "action", rev: 1.5, action: 1 },
      { type: "hello", name: "a" },
      { type: "hello", protocol: 1, name: 3 },
      { type: "ping" },
    ]) {
      expect(parseGuestMessage(bad)).toBeNull();
    }
  });

  it("accepts well-formed messages", () => {
    expect(parseGuestMessage({ type: "hello", protocol: PROTOCOL_VERSION, name: "Ana" })).toEqual({
      type: "hello",
      protocol: PROTOCOL_VERSION,
      name: "Ana",
      seatToken: undefined,
    });
    expect(parseGuestMessage({ type: "hello", protocol: 1, name: "Ana", seatToken: "t" })?.type).toBe("hello");
    expect(parseGuestMessage({ type: "action", rev: 3, action: { Place: { q: 0, r: 0 } } })).toEqual({
      type: "action",
      rev: 3,
      action: { Place: { q: 0, r: 0 } },
    });
    expect(parseGuestMessage({ type: "ping", t: 12 })).toEqual({ type: "ping", t: 12 });
  });

  it("reads a null token as no token (PeerJS's binary encoding turns undefined into null)", () => {
    expect(parseGuestMessage({ type: "hello", protocol: 1, name: "Ana", seatToken: null })).toEqual({
      type: "hello",
      protocol: 1,
      name: "Ana",
      seatToken: undefined,
    });
  });
});

describe("parseHostMessage", () => {
  it("needs a known type", () => {
    expect(parseHostMessage({ type: "nope" })).toBeNull();
    expect(parseHostMessage(null)).toBeNull();
    expect(parseHostMessage({ type: "pong", t: 1 })).toEqual({ type: "pong", t: 1 });
  });
});

describe("codes", () => {
  it("are 6 unambiguous characters", () => {
    for (let i = 0; i < 50; i++) expect(newCode()).toMatch(/^[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{6}$/);
  });

  it("are normalized", () => {
    expect(normalizeCode("  ab3dxy ")).toBe("AB3DXY");
  });
});
