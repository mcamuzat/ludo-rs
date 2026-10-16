import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import board from "../board.json";
import { BantuBoard, describeLast, destinations, endText, pieceClick } from "../Board";
import type { BantuAction, BantuView } from "../types";

const NAMES = ["Ana", "Hugo", "Cléo"];

const VIEW: BantuView = {
  players: 2,
  pieces: [
    [{ kind: "On", cell: 69 }, { kind: "Home" }, { kind: "Finished" }, { kind: "Finished" }],
    [{ kind: "On", cell: 62 }, { kind: "Finished" }, { kind: "Finished" }, { kind: "Finished" }],
  ],
  turn: 0,
  lengths: [1, 2, 0, 0],
  protected: [],
  last: null,
  quietLeft: 28,
  end: null,
  legal: [],
};

const LEGAL: BantuAction[] = [
  { Move: { piece: 1, to: { Cell: 62 } } },
  { Move: { piece: 2, to: { Cell: 60 } } },
  { Move: { piece: 2, to: "Finish" } },
];

function render(view: BantuView, legal: BantuAction[], me: number | null = 0): string {
  return renderToStaticMarkup(
    <BantuBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("destinations", () => {
  it("lists where one piece may go", () => {
    expect(destinations(2, LEGAL)).toEqual([{ Cell: 60 }, "Finish"]);
    expect(destinations(3, LEGAL)).toEqual([]);
  });
});

describe("pieceClick", () => {
  it("takes the piece standing on a destination of the selected piece", () => {
    // My piece 1 is selected and may land on 62, where Hugo's piece stands.
    expect(pieceClick(1, { kind: "On", cell: 62 }, 1, 0, [{ Cell: 62 }], LEGAL)).toEqual({ Cell: 62 });
  });

  it("selects my own movable piece, and ignores the others", () => {
    expect(pieceClick(0, { kind: "On", cell: 69 }, 1, 0, [], LEGAL)).toBe("select");
    expect(pieceClick(1, { kind: "On", cell: 62 }, 1, 0, [], LEGAL)).toBeNull();
    expect(pieceClick(0, { kind: "Finished" }, 3, 0, [], LEGAL)).toBeNull();
  });
});

describe("describeLast", () => {
  it("tells a capture", () => {
    const text = describeLast(
      { seat: 1, piece: 3, from: { kind: "On", cell: 69 }, to: { kind: "On", cell: 62 }, captured: [0, 1] },
      NAMES,
    );
    expect(text).toContain("Hugo");
    expect(text).toContain("renvoie le pion 1 d'Ana au départ");
  });

  it("tells a piece going in", () => {
    expect(
      describeLast({ seat: 0, piece: 4, from: { kind: "On", cell: 81 }, to: { kind: "Finished" }, captured: null }, NAMES),
    ).toBe("Ana amène son pion 4 à l'arrivée.");
  });
});

describe("endText", () => {
  it("explains each end", () => {
    expect(endText({ Won: 2 }, NAMES)).toBe("Cléo a ramené tous ses pions : partie gagnée.");
    expect(endText("QuietLimit", NAMES)).toContain("30 tours de table");
    expect(endText("Blocked", NAMES)).toContain("plus personne ne peut jouer");
  });
});

describe("BantuBoard", () => {
  it("draws every cell of the board", () => {
    const html = render(VIEW, LEGAL);
    expect(html.match(/data-cell="/g)?.length).toBe(board.cells.length);
  });

  it("makes my movable pieces clickable, and not the others", () => {
    const html = render(VIEW, LEGAL);
    expect(html).toContain('aria-label="Pion 1 d&#x27;Ana"');
    expect(html.match(/class="piece movable"/g)?.length).toBe(2);
    expect(render(VIEW, [], 1).match(/class="piece movable"/g)).toBeNull();
  });

  it("shows the quiet count and the end", () => {
    expect(render(VIEW, LEGAL)).toContain("encore 28 tours de table");
    expect(render({ ...VIEW, end: "QuietLimit", turn: null }, [])).toContain("30 tours de table");
  });
});
