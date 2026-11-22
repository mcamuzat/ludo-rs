import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { actionFor, FiveByFifteenBoard } from "../Board";
import { loadStats, recordResult } from "../stats";
import type { Color, FiveByFifteenAction, FiveByFifteenView, Tile } from "../types";

const COLORS: Color[] = ["Emerald", "Crimson", "Azure", "Amber", "Violet"];

function view(patch: Partial<FiveByFifteenView> = {}): FiveByFifteenView {
  const cells: (Tile | null)[][] = COLORS.map(() => Array.from({ length: 15 }, () => null));
  cells[3][8] = { color: "Amber", number: 13 };
  cells[3][9] = { color: "Amber", number: 14 };
  cells[3][10] = { color: "Amber", number: 15 };
  cells[0][5] = { color: "Violet", number: 8 };
  return {
    route: "Short",
    minRun: 3,
    anchors: COLORS,
    cells,
    status: "Playing",
    moves: 4,
    repeated: false,
    sorted: [0, 0, 0, 0, 0],
    locomotives: [{ row: 3, col: 10, ready: true }],
    ...patch,
  };
}

const LEGAL: FiveByFifteenAction[] = [
  { Place: { from: { row: 0, col: 5 }, to: { row: 3, col: 11 } } },
  { Shift: { row: 3, to: 11 } },
  "GiveUp",
];

function render(v: FiveByFifteenView, withAssist = true) {
  return renderToStaticMarkup(
    <FiveByFifteenBoard
      view={v}
      legalActions={v.status === "Playing" ? LEGAL : []}
      pending={v.status === "Playing" ? [0] : []}
      me={0}
      seatNames={["Toi"]}
      submit={() => {}}
      assist={withAssist ? { vsAi: false, takeBack: () => {} } : undefined}
    />,
  );
}

describe("5×15 board", () => {
  it("shows each line's 1 as a large number with the line's name", () => {
    const html = render(view());
    for (const name of ["Émeraude", "Cramoisi", "Azur", "Ambre", "Violet"]) expect(html).toContain(name);
    expect(html.match(/class="fx-anchor[^"]*"/g)).toHaveLength(5);
    for (const sign of ["●", "▲", "■", "◆", "★"]) expect(html).not.toContain(sign);
  });

  it("lights the locomotive", () => {
    const html = render(view());
    expect(html.match(/loco-linked/g)).toHaveLength(3);
    expect(html).toContain("loco-ready");
    expect(render(view({ locomotives: [{ row: 3, col: 10, ready: false }] }))).not.toContain("loco-ready");
  });

  it("warns of a loop and announces the result", () => {
    expect(render(view({ repeated: true }))).toContain("déjà apparue");
    expect(render(view())).not.toContain("déjà apparue");
    expect(render(view({ status: "Won" }))).toContain("gagné");
    expect(render(view({ status: "Stuck" }))).toContain("Plus aucun coup");
  });

  it("offers take-back only on this device, and giving up while playing", () => {
    expect(render(view())).toContain("Annuler");
    expect(render(view(), false)).not.toContain("Annuler");
    expect(render(view())).toContain("Abandonner");
    expect(render(view({ status: "Won" }))).not.toContain("Abandonner");
  });

  it("finds the action that takes a tile to a cell", () => {
    expect(actionFor(LEGAL, { row: 0, col: 5 }, { row: 3, col: 11 })).toEqual(LEGAL[0]);
    expect(actionFor(LEGAL, { row: 3, col: 10 }, { row: 3, col: 11 })).toEqual(LEGAL[1]);
    expect(actionFor(LEGAL, { row: 3, col: 9 }, { row: 3, col: 11 })).toBeNull();
  });
});

describe("5×15 statistics", () => {
  function memory() {
    const data = new Map<string, string>();
    const store = { getItem: (k: string) => data.get(k) ?? null, setItem: (k: string, v: string) => void data.set(k, v) };
    return () => store;
  }

  it("counts games played and won per route", () => {
    const store = memory();
    recordResult("Short", true, store);
    recordResult("Short", false, store);
    recordResult("Long", false, store);
    expect(loadStats(store)).toEqual({
      Short: { played: 2, won: 1 },
      Medium: { played: 0, won: 0 },
      Long: { played: 1, won: 0 },
    });
  });

  it("survives a storage that throws", () => {
    const blocked = () => {
      throw new Error("blocked");
    };
    expect(() => recordResult("Medium", true, blocked)).not.toThrow();
    expect(loadStats(blocked).Medium).toEqual({ played: 0, won: 0 });
  });
});
