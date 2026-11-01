import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { BlackHoleBoard, finishText } from "../Board";
import type { BlackHoleAction, BlackHoleView } from "../types";

const NAMES = ["Ana", "Hugo"];
const empty = (): BlackHoleView["cells"] => Array.from({ length: 21 }, () => null);

const VIEW: BlackHoleView = {
  me: 0,
  order: "Ascending",
  cells: empty(),
  remaining: [
    [3, 4, 5, 6, 7, 8, 9, 10],
    [3, 4, 5, 6, 7, 8, 9, 10],
  ],
  next: 3,
  turn: 0,
  last: 4,
  finish: null,
  around: [],
  legal: [{ Place: { cell: 2, tile: 3 } }, { Place: { cell: 7, tile: 3 } }],
};

function render(v: BlackHoleView, me: number | null = 0) {
  return renderToStaticMarkup(
    <BlackHoleBoard view={v} legalActions={v.legal} pending={v.turn === null ? [] : [v.turn]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("Black Hole board", () => {
  it("offers the free cells for the tile to lay", () => {
    const html = render(VIEW);
    expect(html).toContain('aria-label="case 3"');
    expect(html).toContain('aria-label="case 8"');
    expect(html).toContain("Pose ton 3 sur une case vide.");
  });

  it("lets the free order pick a tile", () => {
    const legal: BlackHoleAction[] = [3, 4].map((tile) => ({ Place: { cell: 2, tile } }));
    const html = render({ ...VIEW, order: "Free", next: null, legal });
    expect(html).toContain('aria-pressed="true"');
    expect(html).toContain("Choisis une tuile, puis une case : tu poses le 3.");
  });

  it("names the winner and the totals", () => {
    expect(finishText({ hole: 14, sums: [8, 11] }, NAMES)).toBe("Ana gagne (8 contre 11).");
    expect(finishText({ hole: 14, sums: [12, 5] }, NAMES)).toBe("Hugo gagne (5 contre 12).");
    expect(finishText({ hole: 14, sums: [7, 7] }, NAMES)).toBe("Partie nulle : 7 contre 7.");
    const cells = empty();
    cells[13] = { owner: 1, value: 9 };
    const html = render({ ...VIEW, cells, turn: null, legal: [], finish: { hole: 14, sums: [8, 11] }, around: [13, 9, 19, 20] });
    expect(html).toContain("bh-hole");
    expect(html).toContain("underline");
    expect(html).toContain("11 autour du trou");
  });
});
