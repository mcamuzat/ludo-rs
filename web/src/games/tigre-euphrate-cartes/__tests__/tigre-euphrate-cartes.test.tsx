import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { conflictText, eventText, posName, TigreEuphrateCartesBoard } from "../Board";
import type { Slot, TecAction, TecView } from "../types";

const NAMES = ["Ana", "Hugo"];
const civ = (c: "Black" | "Red" | "Blue" | "Green"): Slot => ({ face: { Civ: c }, leader: null });

const VIEW: TecView = {
  me: 0,
  players: 2,
  heads: Array.from({ length: 8 }, (_, i): Slot => ({
    face: i === 7 ? "Exchanged" : "Treasure",
    leader: i === 0 ? { seat: 1, color: "Black" } : null,
  })),
  gaps: [civ("Green"), null, null, null, null, null, null],
  columns: [
    [civ("Red"), civ("Blue"), civ("Black")],
    [civ("Green"), { face: { Civ: "Red" }, leader: { seat: 0, color: "Green" } }, civ("Blue")],
    [civ("Blue"), { face: { Ship: "Black" }, leader: null }],
    [],
    [],
    [],
    [],
    [],
  ],
  join: null,
  hand: [2, 3, 1, 2],
  handSizes: [8, 8],
  supply: [["Black", "Red", "Blue"], ["Red", "Blue", "Green"]],
  catastrophes: [true, false],
  scores: [
    [1, 0, 2, 0],
    [0, 3, 0, 1],
  ],
  treasures: [0, 1],
  ships: ["Red", "Green"],
  deck: 120,
  treasuresLeft: 7,
  turn: 0,
  deciding: 0,
  actionsLeft: 2,
  phase: "Action",
  quietLeft: 40,
  steps: [],
  lastTurn: null,
  end: null,
  finalScores: null,
};

const LEGAL: TecAction[] = [{ Extend: { color: "Red", column: 3 } }, { Leader: { color: "Black", to: { Head: 3 } } }, "EndTurn"];

function render(v: TecView, legal: TecAction[] = LEGAL, me: number | null = 0) {
  return renderToStaticMarkup(
    <TigreEuphrateCartesBoard
      view={v}
      legalActions={legal}
      pending={v.deciding === null ? [] : [v.deciding]}
      me={me}
      seatNames={NAMES}
      submit={() => {}}
    />,
  );
}

describe("Tigre et Euphrate, cartes", () => {
  it("lays out treasures, junctions, columns, leaders and ships", () => {
    const html = render(VIEW);
    expect(html).toContain('aria-label="trésor, le trésor 1"');
    expect(html).toContain('aria-label="un marché, la jonction 1-2"');
    expect(html).toContain("le navire bleu-noir, la colonne 3, carte 2");
    expect(html).toContain("temple (trésor pris)");
    expect(html.match(/class="tc-leader /g)?.length).toBe(2);
    // My own leader is a button (to move it); the opponent's is not.
    expect(html).toContain('<button type="button" class="tc-card tc-Red" aria-label="un temple, la colonne 2, carte 2"');
    expect(html).not.toContain('<button type="button" class="tc-card tc-Red tc-treasure" aria-label="trésor, le trésor 1"');
    expect(html).toContain("Passer mon tour");
    expect(html).toContain("navires à bâtir : bleu-rouge, bleu-vert");
  });

  it("shows the face-down junction during its conflicts, and the commit choices to the deciding seat only", () => {
    const conflict = {
      kind: "External" as const,
      color: "Green" as const,
      attacker: 0,
      defender: 1,
      attackerBase: 3,
      defenderBase: 1,
      attackerAdded: null,
    };
    const v: TecView = {
      ...VIEW,
      gaps: [civ("Green"), civ("Blue"), null, null, null, null, null],
      join: 1,
      phase: { Conflict: { conflict, most: 2 } },
    };
    const legal: TecAction[] = [0, 1, 2].map((count) => ({ Commit: { count } }));
    const html = render(v, legal);
    expect(html).toContain("tc-card tc-facedown");
    expect(html).toContain("Ajouter 2");
    expect(html).toContain(conflictText(conflict, NAMES));
    const other = render({ ...v, me: 1, hand: [1, 1, 1, 1], phase: { Conflict: { conflict, most: null } } }, [], 1);
    expect(other).not.toContain("Ajouter");
    expect(other).toContain("À Ana de jouer.");
  });

  it("tells the turn's events", () => {
    expect(eventText({ Extend: { seat: 0, color: "Green", column: 1, scorer: 1 } }, NAMES)).toBe(
      "Ana pose un marché en colonne 2 (1 point vert pour Hugo)",
    );
    expect(
      eventText(
        {
          Conflict: {
            kind: "External",
            color: "Green",
            attacker: 0,
            defender: 1,
            attackerTotal: 5,
            defenderTotal: 5,
            winner: 1,
            scored: true,
            removed: 3,
          },
        },
        NAMES,
      ),
    ).toBe("Hugo gagne le conflit des marchands (5 contre 5), 3 cartes prises");
    expect(posName({ Card: [2, 0] })).toBe("la colonne 3, carte 1");
  });

  it("ranks the players at the end", () => {
    const html = render(
      {
        ...VIEW,
        turn: null,
        deciding: null,
        phase: "Over",
        end: "Deck",
        finalScores: [
          { scores: [1, 0, 2, 0], treasures: 0, withTreasures: [1, 0, 2, 0], weakest: 0, rank: 1 },
          { scores: [1, 3, 1, 1], treasures: 1, withTreasures: [2, 3, 1, 1], weakest: 1, rank: 0 },
        ],
      },
      [],
    );
    expect(html).toContain("La pioche ne suffit plus");
    expect(html.indexOf("Hugo</td>")).toBeLessThan(html.indexOf("Ana</td>"));
  });
});
