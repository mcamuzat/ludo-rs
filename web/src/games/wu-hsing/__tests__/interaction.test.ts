import { describe, expect, it } from "vitest";
import { DOMINO_BODY, SEAT_COLORS, previewAt, previewClick } from "../interaction";
import type { HandDomino, LegalMove } from "../types";

const place = (domino: number, row: number, col: number, dir: "Right" | "Down", flipped = false, points = 9): LegalMove => ({
  action: { Place: { domino, row, col, dir, flipped } },
  points,
});

// Two 2-3 (ids 6 and 21), a 1-3 (id 2) and a 5-5 (id 14).
const HAND: HandDomino[] = [
  { id: 2, a: 1, b: 3 },
  { id: 6, a: 2, b: 3 },
  { id: 14, a: 5, b: 5 },
  { id: 21, a: 2, b: 3 },
];
const twin = HAND[3];
const oneThree = HAND[0];
const double = HAND[2];

describe("Wu Hsing interaction", () => {
  it("sends the lower twin when the higher one is picked", () => {
    const legal = [place(6, -1, 0, "Right")];
    for (const aid of ["places", "none"] as const) {
      const p = previewAt(HAND, legal, aid, twin, [-1, 0], "Right", false, 0);
      expect(p?.move?.action).toEqual({ Place: { domino: 6, row: -1, col: 0, dir: "Right", flipped: false } });
    }
  });

  it("cycles through the placements covering the touched cell", () => {
    const legal = [place(14, -1, 0, "Right", false, 18), place(14, -1, 0, "Down", false, 6)];
    const at = (choice: number) => previewAt(HAND, legal, "places", double, [-1, 0], "Right", false, choice)?.move?.points;
    expect([at(0), at(1), at(2)]).toEqual([18, 6, 18]);
    // A cell no placement covers: no preview.
    expect(previewAt(HAND, legal, "places", double, [5, 5], "Right", false, 0)).toBeNull();
  });

  it("checks the manual preview with the flip", () => {
    // 1-3 is legal only with the 3 on the anchor.
    const legal = [place(2, 0, 2, "Down", true, 12)];
    const plain = previewAt(HAND, legal, "none", oneThree, [0, 2], "Down", false, 0);
    expect(plain?.sides).toEqual([1, 3]);
    expect(plain?.move).toBeNull();
    const flipped = previewAt(HAND, legal, "playable", oneThree, [0, 2], "Down", true, 0);
    expect(flipped?.sides).toEqual([3, 1]);
    expect(flipped?.move?.points).toBe(12);
    // Wrong direction: illegal.
    expect(previewAt(HAND, legal, "none", oneThree, [0, 2], "Right", true, 0)?.move).toBeNull();
  });

  it("matches a double whichever way it is flipped", () => {
    const legal = [place(14, -1, 0, "Right", false, 18)];
    expect(previewAt(HAND, legal, "none", double, [-1, 0], "Right", true, 0)?.move?.points).toBe(18);
  });

  it("places only from the touched cell; the other half moves the cursor", () => {
    const legal = [place(14, -1, 0, "Right", false, 18)];
    const p = previewAt(HAND, legal, "places", double, [-1, 1], "Right", false, 0)!;
    expect(previewClick(p, [-1, 1], [-1, 1])).toBe("confirm");
    expect(previewClick(p, [-1, 0], [-1, 1])).toBe("move");
  });

  it("outlines every seat in a colour that stands out from the domino", () => {
    const lum = (hex: string) => {
      const n = parseInt(hex.slice(1), 16);
      return (0.299 * (n >> 16) + 0.587 * ((n >> 8) & 255) + 0.114 * (n & 255)) / 255;
    };
    for (const c of SEAT_COLORS) expect(lum(c) - lum(DOMINO_BODY)).toBeGreaterThan(0.3);
  });
});
