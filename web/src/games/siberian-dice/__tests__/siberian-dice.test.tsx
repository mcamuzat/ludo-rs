import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { SiberianDiceBoard } from "../Board";
import type { CellView, SiberianDiceAction, SiberianDiceView } from "../types";

const NAMES = ["Ana", "Hugo"];

/** The 37 cells in the engine's order: sorted by (a, b), |a - b| <= 3. */
const COORDS: { a: number; b: number }[] = [];
for (let a = 1; a <= 7; a++) for (let b = 1; b <= 7; b++) if (Math.abs(a - b) <= 3) COORDS.push({ a, b });
const id = (ab: number) => COORDS.findIndex((c) => c.a === Math.floor(ab / 10) && c.b === ab % 10);

/** The example figure of the rules: 44 and 54 perfect. */
const FIGURE: [number, number, number, boolean][] = [
  [43, 1, 2, false],
  [54, 0, 3, true],
  [33, 1, 1, false],
  [44, 0, 5, true],
  [55, 1, 4, false],
  [45, 0, 6, false],
];

function view(dice: [number, number, number, boolean][], over: Partial<SiberianDiceView> = {}): SiberianDiceView {
  const cells: CellView[] = COORDS.map((c) => ({ ...c, die: null }));
  for (const [ab, owner, value, perfect] of dice) cells[id(ab)].die = { owner, value, perfect };
  return {
    cells,
    reserve: [9, 9],
    perfect: [2, 0],
    turn: 0,
    step: "Choose",
    last: null,
    quietLeft: 30,
    end: null,
    winners: [],
    ...over,
  };
}

function render(v: SiberianDiceView, legal: SiberianDiceAction[], me: number | null = 0) {
  return renderToStaticMarkup(
    <SiberianDiceBoard view={v} legalActions={legal} pending={[0]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

function cellMarkup(html: string, ab: number): string {
  const m = html.match(new RegExp(`<g[^>]*data-cell="${ab}"[^>]*>`));
  if (!m) throw new Error(`no cell ${ab}`);
  return m[0];
}

describe("Siberian Dice board", () => {
  it("marks perfect dice", () => {
    const html = render(view(FIGURE), [{ Pick: "Reserve" }]);
    for (const ab of [44, 54]) expect(cellMarkup(html, ab)).toContain("perfect");
    for (const ab of [43, 33, 55, 45]) expect(cellMarkup(html, ab)).not.toContain("perfect");
    expect(html).toContain("parfaits 2/6");
  });

  it("shows the rolled die and lights the vacated cell", () => {
    const dice = FIGURE.filter(([ab]) => ab !== 44);
    const v = view(dice, { step: { Place: { from: { Cell: id(44) }, value: 3 } } });
    const html = render(v, [{ Place: id(44) }, { Place: id(11) }]);
    expect(html).toContain('aria-label="Dé lancé : 3"');
    expect(cellMarkup(html, 44)).toContain("playable");
    expect(cellMarkup(html, 11)).toContain("playable");
    expect(cellMarkup(html, 22)).not.toContain("playable");
  });

  it("the reserve is clickable only when picking from it is legal", () => {
    expect(render(view(FIGURE), [{ Pick: "Reserve" }])).toMatch(/<button[^>]*class="sd-reserve[^"]*"(?![^>]*disabled)/);
    expect(render(view(FIGURE), [{ Pick: { Cell: id(44) } }])).toMatch(/<button[^>]*class="sd-reserve[^"]*"[^>]*disabled/);
    expect(render(view(FIGURE), [{ Pick: { Cell: id(44) } }])).toMatch(/data-cell="44"[^>]*class="[^"]*pickable/);
  });

  it("names the winner and the draws", () => {
    expect(render(view(FIGURE, { end: { Won: { seat: 1 } }, winners: [1] }), [])).toContain("Hugo gagne");
    expect(render(view(FIGURE, { end: "BothSix", winners: [0, 1] }), [])).toContain("Partie nulle");
    expect(render(view(FIGURE, { end: "QuietLimit", winners: [0, 1] }), [])).toContain("Partie nulle");
  });
});
