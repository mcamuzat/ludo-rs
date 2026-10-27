import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { HattrickBoard, cardAction, cardName } from "../hattrick/Board";
import type { Color, HattrickAction, HattrickCard, HattrickView } from "../hattrick/types";

const C = (color: Color, value: number): HattrickCard => ({ color, value });
const NAMES = ["Ana", "Hugo", "Cléo", "Malo"];

const VIEW: HattrickView = {
  me: 0,
  hand: [C("Red", 12), C("Blue", 4), C("Green", 7)],
  handCounts: [3, 2, 2, 3],
  piles: [
    { color: "Red", cards: [[2, C("Red", 9)]] },
    { color: "Blue", cards: [[3, C("Blue", 15)]] },
  ],
  passes: [[1, C("Green", 3)]],
  lastTrick: null,
  won: [
    [3, 0, 1],
    [0, 2, 0],
    [1, 1, 1],
    [0, 0, 0],
  ],
  passed: [0, 1, 0, 0],
  scores: [4, -1, 0, 2],
  round: 2,
  rounds: 8,
  dealer: 3,
  turn: 0,
  lastRound: null,
  legal: [],
};

const LEGAL: HattrickAction[] = [
  { Play: { card: C("Red", 12) } },
  { Play: { card: C("Blue", 4) } },
  { Pass: { card: C("Green", 7) } },
];

function board(view: HattrickView, legal: HattrickAction[]): string {
  return renderToStaticMarkup(
    <HattrickBoard view={view} legalActions={legal} pending={[]} me={view.me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("cardAction", () => {
  it("finds the play or the pass with a card", () => {
    expect(cardAction(C("Red", 12), LEGAL)).toEqual({
      Play: { card: C("Red", 12) },
    });
    expect(cardAction(C("Green", 7), LEGAL)).toEqual({
      Pass: { card: C("Green", 7) },
    });
    expect(cardAction(C("Green", 8), LEGAL)).toBeNull();
  });

  it("names a card", () => {
    expect(cardName(C("Blue", 20))).toBe("bleu 20");
  });
});

describe("HattrickBoard", () => {
  it("shows both piles and the passes", () => {
    const html = board(VIEW, LEGAL);
    expect(html).toContain("Pile rouge");
    expect(html).toContain("Pile bleue");
    expect(html).toContain("Hugo passe (vert 3)");
    expect(html).toContain("Manche 2 sur 8");
  });

  it("marks the third colour as a pass", () => {
    const html = board(VIEW, LEGAL);
    expect(html).toContain('aria-label="Passer avec le vert 7 (−2)"');
    expect(html).toContain('aria-label="rouge 12"');
  });

  it("sums up the last round", () => {
    const html = board(
      {
        ...VIEW,
        lastRound: [
          { won: [9, 2, 4], passed: 1, points: 1, total: 1 },
          { won: [0, 0, 0], passed: 0, points: 0, total: 0 },
          { won: [1, 1, 0], passed: 2, points: -4, total: -4 },
          { won: [0, 3, 0], passed: 0, points: 3, total: 3 },
        ],
      },
      LEGAL,
    );
    expect(html).toContain("Manche précédente");
    expect(html).toContain("+3");
    expect(html).toContain("-4");
  });
});
