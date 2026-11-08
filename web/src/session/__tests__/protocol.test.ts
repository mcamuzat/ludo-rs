import { afterEach, describe, expect, it } from "vitest";
import { setLang } from "../../i18n";
import { PROTOCOL_VERSION, newCode, normalizeCode, parseGuestMessage, parseHostMessage, rejectText } from "../protocol";

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

describe("refusals", () => {
  afterEach(() => setLang("fr"));

  it("are told in French", () => {
    expect(rejectText({ code: "full" })).toBe("La table est complète.");
    expect(rejectText({ code: "version", host: 4, guest: 3 })).toBe("Versions différentes (hôte 4, toi 3) : recharge la page.");
    expect(rejectText({ code: "stale" })).toBe("La partie a avancé entre-temps : coup ignoré.");
  });

  it("are told in English by an English guest", () => {
    setLang("en");
    expect(rejectText({ code: "full" })).toBe("The table is full.");
    expect(rejectText({ code: "started" })).toBe("The game has already started.");
    expect(rejectText({ code: "version", host: 4, guest: 3 })).toBe("Different versions (host 4, you 3): reload the page.");
  });

  it("survive a host or guest of another version", () => {
    // A version-3 host sends its refusal as French text.
    expect(rejectText("Versions différentes (hôte 3, toi 4) : recharge la page." as never)).toBe(
      "Versions différentes (hôte 3, toi 4) : recharge la page.",
    );
    expect(rejectText({ code: "bogus" } as never)).toBe("Refus de l'hôte.");
    expect(rejectText(null as never)).toBe("Refus de l'hôte.");
    setLang("en");
    expect(rejectText({ code: "bogus" } as never)).toBe("Refused by the host.");
  });

  it("keep the engine's own words", () => {
    expect(rejectText({ code: "engine", text: "pas 14" })).toBe("pas 14");
    setLang("en");
    expect(rejectText({ code: "engine", text: "pas 14" })).toBe("pas 14");
  });
});
