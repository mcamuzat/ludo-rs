import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { WuHsingBoard, finishText, type Aid } from "../Board";
import type { LegalMove, WuHsingView } from "../types";

const NAMES = ["Ana", "Hugo"];

const place = (domino: number, row: number, col: number, dir: "Right" | "Down", flipped = false, points = 9): LegalMove => ({
  action: { Place: { domino, row, col, dir, flipped } },
  points,
});

/** The square 1-2 over 3-4; Ana holds 5-5 (id 14), 1-3 (id 2) and 2-2 (id 5). */
const VIEW: WuHsingView = {
  me: 0,
  placed: [
    { row: 0, col: 0, dir: "Right", first: 1, second: 2, by: null },
    { row: 1, col: 0, dir: "Right", first: 3, second: 4, by: null },
  ],
  hand: [
    { id: 2, a: 1, b: 3 },
    { id: 5, a: 2, b: 2 },
    { id: 14, a: 5, b: 5 },
  ],
  handSizes: [3, 14],
  scores: [0, 0],
  turn: 0,
  first: 0,
  lastRound: false,
  last: null,
  finish: null,
  legal: [place(14, -1, 0, "Right", false, 18), place(14, 0, -1, "Down", false, 18)],
};

function render(v: WuHsingView, aid: Aid = "places", me: number | null = 0) {
  return renderToStaticMarkup(
    <WuHsingBoard
      view={v}
      legalActions={v.legal.map((m) => m.action)}
      pending={v.turn === null ? [] : [v.turn]}
      me={me}
      seatNames={NAMES}
      submit={() => {}}
      initialAid={aid}
    />,
  );
}

describe("Wu Hsing board", () => {
  it("highlights the cells of the legal placements", () => {
    const html = render(VIEW, "places");
    // 5-5, the only playable domino, is picked: its placements cover (-1,0), (-1,1), (0,-1) and (1,-1).
    for (const cell of ["-1,0", "-1,1", "0,-1", "1,-1"]) {
      expect(html).toContain(`data-cell="${cell}" class="wh-cell target"`);
    }
    expect(html).not.toContain('data-cell="2,0" class="wh-cell target"');
  });

  it("marks playable dominoes", () => {
    const html = render(VIEW, "playable");
    expect(html.match(/wh-hand-domino playable/g)?.length).toBe(1);
    expect(html).toContain('aria-label="5-5, jouable"');
    expect(html).not.toContain("wh-cell target");
    expect(render(VIEW, "none")).not.toContain("playable");
  });

  it("draws dominoes left of and above the start", () => {
    const v: WuHsingView = {
      ...VIEW,
      placed: [...VIEW.placed, { row: -3, col: -4, dir: "Down", first: 5, second: 1, by: 1 }],
    };
    const html = render(v);
    expect(html.match(/class="wh-domino/g)?.length).toBe(3);
    // The grid starts two cells before the smallest coordinates.
    expect(html).toContain('data-cell="-5,-6"');
    expect(html).not.toContain('data-cell="-6,-6"');
    expect(html).not.toContain('data-cell="-5,-7"');
  });

  it("names the winner with penalties", () => {
    expect(finishText({ totals: [112, 87], penalties: [30, 0] }, NAMES)).toBe(
      "Ana gagne (112 contre 87, pénalités 30 et 0).",
    );
    expect(finishText({ totals: [68, 68], penalties: [30, 0] }, NAMES)).toBe(
      "Égalité entre Ana et Hugo (68 contre 68, pénalités 30 et 0).",
    );
    expect(finishText({ totals: [10, 40, 25], penalties: [5, 0, 25] }, ["Ana", "Hugo", "Lou"])).toBe(
      "Hugo gagne (10 contre 40 contre 25, pénalités 5, 0 et 25).",
    );
  });

  it("says who scored, in the second person for oneself", () => {
    const placed = [...VIEW.placed, { row: -1, col: 0, dir: "Right" as const, first: 5, second: 5, by: 0 }];
    expect(render({ ...VIEW, placed, last: { seat: 0, placed: 2, points: 18 } })).toContain("Tu as marqué 18 points.");
    expect(render({ ...VIEW, placed, last: { seat: 1, placed: 2, points: 18 } })).toContain("Hugo a marqué 18 points.");
    expect(render({ ...VIEW, last: { seat: 0, placed: null, points: 0 } })).toContain("Tu as passé.");
  });

  it("hides the hand of a spectator", () => {
    const html = render({ ...VIEW, me: null, hand: [], legal: [] }, "places", null);
    expect(html).not.toContain("wh-hand-domino");
    expect(html).not.toContain("Passer");
  });
});
