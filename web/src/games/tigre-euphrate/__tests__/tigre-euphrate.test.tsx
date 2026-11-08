import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { cellName, conflictText, eventText, TigreEuphrateBoard } from "../Board";
import type { CellView, TigreAction, TigreView } from "../types";

const NAMES = ["Ana", "Hugo", "Léa"];

function cells(): CellView[] {
  const v: CellView[] = Array.from({ length: 176 }, () => ({ river: false, piece: null, treasure: false, corner: false }));
  v[17] = { river: false, piece: { Tile: { color: "Red", flipped: false } }, treasure: true, corner: true };
  v[18] = { river: false, piece: { Leader: { seat: 0, color: "Black" } }, treasure: false, corner: false };
  v[4].river = true;
  return v;
}

const VIEW: TigreView = {
  me: 0,
  players: 3,
  reinforcements: "English",
  cells: cells(),
  monuments: [
    { colors: ["Black", "Red"], at: null },
    { colors: ["Black", "Blue"], at: 40 },
    { colors: ["Black", "Green"], at: null },
    { colors: ["Red", "Blue"], at: null },
    { colors: ["Red", "Green"], at: null },
    { colors: ["Blue", "Green"], at: null },
  ],
  junction: null,
  hand: [1, 2, 3, 0],
  handSizes: [6, 6, 6],
  supply: [["Red", "Blue", "Green"], ["Black", "Red", "Blue", "Green"], ["Black", "Red", "Blue", "Green"]],
  catastrophes: [2, 2, 1],
  myScores: [3, 0, 1, 2],
  treasures: [0, 1, 0],
  bag: 120,
  treasuresLeft: 9,
  turn: 0,
  actionsLeft: 2,
  deciding: 0,
  phase: "Action",
  steps: [],
  lastTurn: { seat: 2, steps: [{ Tile: { seat: 2, color: "Blue", at: 4, scorer: 1 } }] },
  quietLeft: 20,
  end: null,
  finalScores: null,
};

const LEGAL: TigreAction[] = [{ Tile: { color: "Black", to: 19 } }, { Leader: { color: "Red", to: 33 } }, "EndTurn"];

function render(v: TigreView, legal: TigreAction[] = LEGAL, me: number | null = 0) {
  return renderToStaticMarkup(
    <TigreEuphrateBoard view={v} legalActions={legal} pending={v.deciding === null ? [] : [v.deciding]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("Tigre et Euphrate board", () => {
  it("names cells, events and conflicts", () => {
    expect(cellName(0)).toBe("A1");
    expect(cellName(17)).toBe("B2");
    expect(cellName(175)).toBe("P11");
    expect(eventText({ Tile: { seat: 2, color: "Blue", at: 4, scorer: 1 } }, NAMES)).toBe(
      "Léa pose une ferme en E1 (1 point bleu pour Hugo)",
    );
    expect(eventText({ Leader: { seat: 0, color: "Green", from: 3, to: 18 } }, NAMES)).toBe("Ana déplace son marchand en C2");
    expect(
      eventText(
        { Conflict: { kind: "External", color: "Red", attacker: 0, defender: 1, attackerTotal: 4, defenderTotal: 3, winner: 0, removed: 2 } },
        NAMES,
      ),
    ).toBe("Ana gagne le conflit des prêtres (4 contre 3), 2 tuiles retirées");
    expect(
      conflictText(
        { kind: "Internal", color: "Blue", attacker: 0, defender: 1, attackerAt: 1, defenderAt: 2, attackerBase: 1, defenderBase: 2, attackerAdded: 2 },
        NAMES,
      ),
    ).toBe("Conflit interne des fermiers (temples) : Ana attaque avec 1 + 2, Hugo défend avec 2.");
  });

  it("shows my hand and my points, not the others'", () => {
    const html = render(VIEW);
    expect(html).toContain("Ma main");
    expect(html).toContain("Mes points");
    expect(html).toContain("Passer mon tour");
    expect(html).toContain("Tour précédent (Léa)");
    expect(html).toContain("Ana (Lion)");
  });

  it("asks for reinforcements", () => {
    const v: TigreView = {
      ...VIEW,
      phase: {
        Reinforce: {
          conflict: { kind: "Internal", color: "Black", attacker: 1, defender: 0, attackerAt: 18, defenderAt: 33, attackerBase: 2, defenderBase: 1, attackerAdded: 0 },
          options: [0, 1],
        },
      },
    };
    const html = render(v, [{ Reinforce: { count: 0 } }, { Reinforce: { count: 1 } }]);
    expect(html).toContain("Pas de renfort");
    expect(html).toContain("Ajouter 1");
  });

  it("lifts the screens at the end", () => {
    const v: TigreView = {
      ...VIEW,
      turn: null,
      deciding: null,
      phase: "Over",
      end: "Treasures",
      finalScores: [
        { scores: [5, 6, 7, 8], treasures: 1, withTreasures: [6, 6, 7, 8], weakest: 6, rank: 0 },
        { scores: [4, 4, 4, 4], treasures: 0, withTreasures: [4, 4, 4, 4], weakest: 4, rank: 2 },
        { scores: [9, 5, 5, 5], treasures: 0, withTreasures: [9, 5, 5, 5], weakest: 5, rank: 1 },
      ],
    };
    const html = render(v, []);
    expect(html).toContain("Il ne reste qu&#x27;un ou deux trésors");
    const table = html.slice(html.indexOf("<table"));
    expect(table.indexOf("Léa")).toBeLessThan(table.indexOf("Hugo"));
  });
});
