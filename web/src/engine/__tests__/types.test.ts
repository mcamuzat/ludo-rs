import { describe, expect, it } from "vitest";
import { updateFor, type Snapshot } from "../types";

const snapshot: Snapshot = {
  rev: 4,
  pending: [1, 3],
  outcome: null,
  views: [
    { viewer: 1, view: "v1", legalActions: ["Ron"], pending: [1] },
    { viewer: 2, view: "v2", legalActions: [], pending: [] },
    { viewer: null, view: "vs", legalActions: [], pending: [] },
  ],
};
const clocks = [
  { seat: 1, baseMs: 4000, bankMs: 20000 },
  { seat: 3, baseMs: 2000, bankMs: 5000 },
];

describe("updateFor", () => {
  it("gives each viewer only the pending seats it may know about", () => {
    expect(updateFor(snapshot, 1).pending).toEqual([1]);
    expect(updateFor(snapshot, 2).pending).toEqual([]);
  });

  it("keeps only the clocks of visible pending seats", () => {
    expect(updateFor(snapshot, 1, clocks).clocks).toEqual([clocks[0]]);
    expect(updateFor(snapshot, 2, clocks).clocks).toEqual([]);
    expect(updateFor(snapshot, null, clocks).clocks).toEqual([]);
  });
});
