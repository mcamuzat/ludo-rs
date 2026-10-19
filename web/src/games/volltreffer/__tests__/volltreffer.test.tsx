import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { VolltrefferBoard, offers, publishAction } from "../Board";
import type { Card, VolltrefferAction, VolltrefferView } from "../types";

const NAMES = ["Ana", "Hugo", "Léa"];
const c = (tribe: Card["tribe"], value: number): Card => ({ tribe, value });
const LINE = { spent: 0, melds: 0, preemptive: false, tricks: 0, score: 0 };

const VIEW: VolltrefferView = {
  me: 0,
  hand: [c("Red", 3), c("Green", 3), c("Blue", 3)],
  seats: [
    { score: 25, cards: 3, line: LINE, publication: null, published: false },
    { score: 64, cards: 2, line: LINE, publication: null, published: false },
    { score: 0, cards: 2, line: LINE, publication: null, published: false },
  ],
  phase: "Hire",
  round: 2,
  blessing: c("Green", 9),
  trump: "Green",
  display: [
    { card: c("Red", 1), cost: 0, hireable: true },
    { card: c("Blue", 4), cost: 1, hireable: true },
    { card: c("Yellow", 8), cost: 2, hireable: true },
    { card: c("Green", 2), cost: 4, hireable: false },
  ],
  deck: 30,
  bargaining: 1,
  current: 0,
  recentHires: [{ seat: 1, card: c("Purple", 7), cost: 2 }],
  preemptive: null,
  trick: [],
  lastTrick: null,
  lastRound: null,
  sanctuary: [61, 70],
  ranks: null,
  publicationPoints: [],
  legal: [],
};

const HIRES: VolltrefferAction[] = [
  { Hire: { card: c("Red", 1) } },
  { Hire: { card: c("Blue", 4) } },
  { Hire: { card: c("Yellow", 8) } },
];

const PUBLISH: VolltrefferAction[] = [
  { Publish: { groups: [], tribes: false } },
  { Publish: { groups: [], tribes: true } },
  { Publish: { groups: [3], tribes: false } },
  { Publish: { groups: [3], tribes: true } },
];

function board(view: VolltrefferView, legal: VolltrefferAction[]): string {
  return renderToStaticMarkup(
    <VolltrefferBoard view={view} legalActions={legal} pending={[]} me={0} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("publishAction", () => {
  it("finds the publication and its points", () => {
    expect(publishAction(PUBLISH, [0, 5, 5, 10], [3], true)).toEqual({ action: PUBLISH[3], points: 10 });
    expect(publishAction(PUBLISH, [0, 5, 5, 10], [], false)).toEqual({ action: PUBLISH[0], points: 0 });
    expect(publishAction(PUBLISH, [0, 5, 5, 10], [7], false)).toBeNull();
  });
  it("lists what is on offer", () => {
    expect(offers(PUBLISH)).toEqual({ groups: [3], tribes: true });
    expect(offers(HIRES)).toEqual({ groups: [], tribes: false });
  });
});

describe("VolltrefferBoard", () => {
  it("shows the display with its costs, the hires and the track", () => {
    const html = board(VIEW, HIRES);
    expect(html.match(/class="vt-place"/g)?.length).toBe(4);
    expect(html).toContain("atout vert");
    expect(html).toContain('aria-label="8 jaune"');
    expect(html).toContain("Hugo engage ");
    expect(html.match(/class="vt-space[^"]*sanctuary/g)?.length).toBe(10);
    expect(html).toContain("À 0 :");
    // The card I cannot pay for is dimmed.
    expect(html).toMatch(/vt-green[^"]*dimmed/);
  });
  it("offers the combinations to show", () => {
    const html = board({ ...VIEW, phase: "Publish", publicationPoints: [0, 5, 5, 10] }, PUBLISH);
    expect(html).toContain("Groupe de 3 × 3");
    expect(html).toContain("5 tribus");
    expect(html).toContain("Montrer (+10)");
  });
  it("ranks the players at the end", () => {
    const html = board({ ...VIEW, phase: "Over", current: null, ranks: [1, 0, 2], hand: [] }, []);
    expect(html).toContain("1. Hugo — 64 · victoire");
  });
});
