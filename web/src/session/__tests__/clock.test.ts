import { describe, expect, it } from "vitest";
import { CLOCK_PRESETS, Clocks, clockLabel } from "../clock";

const settings = { baseMs: 5000, bankMs: 20000 };

describe("Clocks", () => {
  it("counts the base time first, then the bank", () => {
    const c = new Clocks(settings);
    c.sync([1], 0);
    expect(c.view(1, 3000)).toEqual({ seat: 1, baseMs: 2000, bankMs: 20000 });
    expect(c.view(1, 8000)).toEqual({ seat: 1, baseMs: 0, bankMs: 17000 });
    expect(c.deadline(1)).toBe(25000);
  });

  it("spends the bank only beyond the base, and keeps it for the next decision", () => {
    const c = new Clocks(settings);
    c.sync([1], 0);
    c.finish(1, 7000);
    c.sync([1], 7000);
    expect(c.view(1, 7000)).toEqual({ seat: 1, baseMs: 5000, bankMs: 18000 });
    expect(c.deadline(1)).toBe(7000 + 5000 + 18000);
  });

  it("stops a seat that is no longer pending", () => {
    const c = new Clocks(settings);
    c.sync([0, 1], 0);
    c.sync([1], 2000);
    expect(c.view(0, 2000)).toBeNull();
    expect(c.views(2000).map((v) => v.seat)).toEqual([1]);
  });

  it("never goes below zero", () => {
    const c = new Clocks(settings);
    c.sync([2], 0);
    expect(c.view(2, 60000)).toEqual({ seat: 2, baseMs: 0, bankMs: 0 });
    c.finish(2, 60000);
    expect(c.toJSON()).toEqual([[2, 0]]);
  });

  it("restores saved banks", () => {
    const c = new Clocks(settings, [[1, 3000]]);
    c.sync([1], 0);
    expect(c.deadline(1)).toBe(8000);
  });

  it("offers 5 + 20 s first", () => {
    expect(CLOCK_PRESETS[0].settings).toEqual(settings);
    expect(CLOCK_PRESETS.at(-1)?.settings).toBeNull();
  });

  it("labels time left from when the update arrived, never more than was sent", () => {
    const c = { seat: 1, baseMs: 5000, bankMs: 20000 };
    expect(clockLabel(c, 3000)).toBe("2 s + 20 s");
    expect(clockLabel(c, 8000)).toBe("0 s + 17 s");
    expect(clockLabel(c, -30000)).toBe("5 s + 20 s");
  });
});
