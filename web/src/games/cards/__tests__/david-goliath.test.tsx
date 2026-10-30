import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DavidGoliathBoard, cardName } from "../david-goliath/Board";
import type { DgAction, DgCard, DgColor, DgView } from "../david-goliath/types";

const C = (color: DgColor, value: number): DgCard => ({ color, value });
const NAMES = ["Ana", "Hugo", "Cléo", "Malo"];

const VIEW: DgView = {
  me: 0,
  hand: [C("Green", 9), C("Red", 3)],
  handCounts: [2, 2, 2, 1],
  trick: [[3, C("Green", 4)]],
  lastTrick: null,
  won: [[C("Yellow", 8)], [], [C("Blue", 2), C("Blue", 3), C("Blue", 6)], [C("Violet", 10), C("Violet", 12)]],
  points: [8, 0, 3, 22],
  scores: [10, 4, 0, 7],
  round: 2,
  rounds: 4,
  dealer: 2,
  turn: 0,
  lastRound: null,
  legal: [{ Play: { card: C("Green", 9) } }],
};

function render(v: DgView, me: number | null = 0) {
  return renderToStaticMarkup(
    <DavidGoliathBoard
      view={v}
      legalActions={v.legal as DgAction[]}
      pending={v.turn === null ? [] : [v.turn]}
      me={me}
      seatNames={NAMES}
      submit={() => {}}
    />,
  );
}

describe("David & Goliath board", () => {
  it("names cards in French", () => {
    expect(cardName(C("Violet", 12))).toBe("violet 12");
  });

  it("only the cards that follow the led colour can be played", () => {
    const html = render(VIEW);
    expect(html).toContain('aria-label="vert 9"');
    expect(html).not.toContain('aria-label="rouge 3"');
    expect(html).toMatch(/dg-card red[^"]*dimmed/);
    expect(html).toContain("Fournis le vert.");
  });

  it("shows each player's won cards and their points", () => {
    const html = render(VIEW);
    expect(html).toContain("22 pts");
    expect(html).toContain('title="violet 12"');
    expect(html).toContain("Manche 2 sur 4");
  });

  it("tells who took what in the last trick", () => {
    const v: DgView = {
      ...VIEW,
      trick: [],
      lastTrick: {
        cards: [
          [0, C("Green", 4)],
          [1, C("Green", 9)],
          [2, C("Red", 2)],
          [3, C("Green", 7)],
        ],
        highest: 1,
        lowest: 2,
      },
    };
    const html = render(v);
    expect(html).toContain("Cléo prend le vert 9 ; Hugo prend le reste et commence.");
  });

  it("shows the previous round", () => {
    const v: DgView = {
      ...VIEW,
      lastRound: [
        { points: 52, total: 52 },
        { points: 3, total: 3 },
        { points: 0, total: 0 },
        { points: 9, total: 9 },
      ],
    };
    expect(render(v)).toContain("Manche précédente");
  });
});
