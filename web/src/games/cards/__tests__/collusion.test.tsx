import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { CollusionBoard, cardAction, phraseText } from "../collusion/Board";
import type { CollusionAction, CollusionView } from "../collusion/types";
import type { Card, Rank, Suit } from "../types";

const N = (rank: Rank, suit: Suit = "Hearts"): Card => ({ kind: "Natural", rank, suit });
const NAMES = ["Ana", "Hugo", "Cléo", "Malo"];

const VIEW: CollusionView = {
  me: 0,
  hand: [N("King"), N("Three"), N("Four", "Spades")],
  handCounts: [3, 3, 3, 4],
  trick: [[3, N("Two")]],
  lastTrick: null,
  taken: [2, 3, 3, 2],
  scores: [12, 30, 8, 0],
  target: 100,
  dealer: 2,
  dealNo: 4,
  turn: 0,
  lastSaid: [null, { type: "Aim", n: 3 }, { type: "WhoAims", n: 4 }, null],
  talk: [
    [1, { type: "Aim", n: 3 }],
    [2, { type: "WhoAims", n: 4 }],
  ],
  lastDeal: null,
  legal: [],
};

const PLAYS: CollusionAction[] = [{ Play: { card: N("King") } }, { Play: { card: N("Three") } }];
const SAYS: CollusionAction[] = [
  ...[2, 3, 4, 5].map((n): CollusionAction => ({ Say: { phrase: { type: "Aim", n } } })),
  { Say: { phrase: { type: "Unsure" } } },
  { Say: { phrase: { type: "WhoAims", n: 0 } } },
];

function board(view: CollusionView, legal: CollusionAction[]): string {
  return renderToStaticMarkup(
    <CollusionBoard view={view} legalActions={legal} pending={[]} me={view.me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("cardAction", () => {
  it("plays a legal card and nothing else", () => {
    expect(cardAction(N("King"), PLAYS)).toEqual({ Play: { card: N("King") } });
    expect(cardAction(N("Four", "Spades"), PLAYS)).toBeNull();
  });
});

describe("phraseText", () => {
  it("words each phrase", () => {
    expect(phraseText({ type: "Aim", n: 1 })).toBe("Je vise 1 pli");
    expect(phraseText({ type: "Aim", n: 4 })).toBe("Je vise 4 plis");
    expect(phraseText({ type: "Unsure" })).toBe("Je ne sais pas encore");
    expect(phraseText({ type: "WhoAims", n: 3 })).toBe("Qui vise 3 ?");
  });
});

describe("CollusionBoard", () => {
  it("dims the cards I cannot play", () => {
    const html = board(VIEW, PLAYS);
    expect(html).toContain('aria-label="roi de cœur"');
    expect(html).toMatch(/class="[^"]*dimmed[^"]*" title="4 de pique"/);
  });

  it("offers to talk only when a phrase is legal", () => {
    expect(board(VIEW, PLAYS)).not.toContain("Parler");
    const html = board(VIEW, [...PLAYS, ...SAYS]);
    expect(html).toContain("Parler");
    expect(html).toContain('aria-label="Je vise 2 plis"');
    expect(html).toContain('aria-label="Je vise 5 plis"');
    expect(html).not.toContain("Je vise 6 plis");
    expect(html).toContain('aria-label="Qui vise 0 ?"');
    expect(html).toContain("Je ne sais pas encore");
  });

  it("shows each seat's latest phrase as a bubble", () => {
    const html = board(VIEW, PLAYS);
    expect(html).toContain("« Je vise 3 plis »");
    expect(html).toContain("« Qui vise 4 ? »");
    const quiet = board({ ...VIEW, lastSaid: [null, null, null, null], talk: [] }, PLAYS);
    expect(quiet).not.toContain("collusion-bubble");
  });

  it("sums up the last deal, deductions included", () => {
    const html = board(
      {
        ...VIEW,
        lastDeal: [
          { tricks: 5, bonus: 10, deducted: false, total: 40 },
          { tricks: 5, bonus: 10, deducted: false, total: 50 },
          { tricks: 2, bonus: 0, deducted: true, total: 93 },
          { tricks: 1, bonus: 0, deducted: false, total: 12 },
        ],
      },
      PLAYS,
    );
    expect(html).toContain("retranché");
    expect(html).toContain("+10");
  });
});
