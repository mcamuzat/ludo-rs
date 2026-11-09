import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { cellName, endText, KuroteBoard, lastText } from "../Board";
import type { KuroteView, Wizard } from "../types";

const NAMES = ["Ana", "Hugo"];
const g: Wizard = { owner: 0, exhausted: false };
const y: Wizard = { owner: 1, exhausted: false };

function start(): KuroteView["cells"] {
  const cells: KuroteView["cells"] = Array.from({ length: 25 }, () => null);
  [1, 2, 3, 5, 9].forEach((c) => (cells[c] = y));
  [15, 19, 21, 22, 23].forEach((c) => (cells[c] = g));
  return cells;
}

const VIEW: KuroteView = {
  me: 0,
  cells: start(),
  reserve: [2, 2],
  captured: [0, 0],
  homeRows: [4, 0],
  turn: 0,
  quietLeft: 80,
  last: null,
  end: null,
  legal: [{ Move: { from: 15, to: 16 } }, { Move: { from: 21, to: 16 } }],
};

function render(v: KuroteView, me: number | null = 0) {
  return renderToStaticMarkup(
    <KuroteBoard view={v} legalActions={v.legal} pending={v.turn === null ? [] : [v.turn]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("Kurote board", () => {
  it("offers the wizards that can move", () => {
    const html = render(VIEW);
    expect(html).toContain('aria-label="sorcier en a2"');
    expect(html).toContain('aria-label="sorcier en b1"');
    expect(html).toContain("Choisis un sorcier à téléporter.");
    expect(html).toContain("0/3 captures");
  });

  it("offers a pass when nothing else is possible", () => {
    const html = render({ ...VIEW, legal: ["Pass"] });
    expect(html).toContain("Passer (aucun coup possible)");
  });

  it("tells the last action and the end", () => {
    expect(cellName(0)).toBe("a5");
    expect(cellName(24)).toBe("e1");
    expect(lastText({ seat: 0, action: { Move: { from: 15, to: 16 } }, captured: [11] }, NAMES)).toBe(
      "Ana téléporte un sorcier de a2 à b2 et capture un sorcier.",
    );
    expect(lastText({ seat: 1, action: { Place: { to: 2 } }, captured: [] }, NAMES)).toBe(
      "Hugo pose un sorcier de réserve en c5.",
    );
    expect(endText({ Captures: { winner: 1 } }, NAMES)).toBe("Hugo gagne : trois sorciers capturés.");
    expect(endText("Limit", NAMES)).toBe("Partie nulle : 80 tours sans capture.");
    const html = render({ ...VIEW, turn: null, legal: [], end: "Blocked" });
    expect(html).toContain("aucun des deux ne peut plus jouer");
  });
});
