import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PennywiseBoard, TakeChoices, describeTake, takesFor } from "../Board";
import type { PennywiseAction, PennywiseView } from "../types";

const NAMES = ["Ana", "Hugo"];
const play = (coin: number, take: number[]): PennywiseAction => ({ Play: { coin, take } });

const VIEW: PennywiseView = {
  character: "Cody",
  purses: [
    [
      { cents: 1, count: 4 },
      { cents: 5, count: 3 },
      { cents: 10, count: 2 },
      { cents: 25, count: 1 },
    ],
    [
      { cents: 1, count: 3 },
      { cents: 5, count: 2 },
    ],
  ],
  totals: [64, 13],
  pot: [
    { cents: 2, count: 2 },
    { cents: 3, count: 1 },
  ],
  potTotal: 7,
  current: 0,
  last: null,
  ruined: null,
  ranks: null,
};

const LEGAL: PennywiseAction[] = [play(1, []), play(5, [3, 1]), play(5, [2, 2]), play(10, [3, 2, 2]), play(25, [3, 2, 2])];

function board(view: PennywiseView, legal: PennywiseAction[], me: number | null): string {
  return renderToStaticMarkup(
    <PennywiseBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("Pennywise", () => {
  it("describes a take", () => {
    expect(describeTake([])).toBe("rien");
    expect(describeTake([5, 3, 1])).toBe("5¢ + 3¢ + 1¢");
  });

  it("groups legal plays by coin", () => {
    expect(takesFor(LEGAL, 5)).toEqual([play(5, [3, 1]), play(5, [2, 2])]);
    expect(takesFor(LEGAL, 20)).toEqual([]);
  });

  it("shows every purse, the pot and the totals", () => {
    const html = board(VIEW, LEGAL, 0);
    expect(html).toContain("Ana");
    expect(html).toContain("Hugo");
    expect(html).toContain("64¢");
    expect(html).toContain("13¢");
    expect(html).toContain("Pot : 7¢");
  });

  it("makes only playable coins clickable", () => {
    const mine = board(VIEW, LEGAL, 0);
    expect(mine.match(/class="coin coin-\d+ playable/g)?.length).toBe(4);
    expect(board(VIEW, [], 1)).not.toContain("playable");
  });

  it("tells the last move", () => {
    const html = board({ ...VIEW, last: { seat: 1, coin: 10, take: [5, 3, 1] } }, LEGAL, 0);
    expect(html).toContain("Hugo a posé 10¢ et repris 5¢ + 3¢ + 1¢");
  });

  it("announces the end with the ranking", () => {
    const html = board({ ...VIEW, current: null, ruined: 0, ranks: [1, 0], totals: [0, 13] }, [], 0);
    expect(html).toContain("Ana est ruiné");
    expect(html.indexOf("1. Hugo")).toBeGreaterThan(-1);
    expect(html.indexOf("1. Hugo")).toBeLessThan(html.indexOf("2. Ana"));
  });
});

describe("TakeChoices", () => {
  it("offers each assortment of the picked coin, and a way back", () => {
    const html = renderToStaticMarkup(
      <TakeChoices choices={takesFor(LEGAL, 5)} onSend={() => {}} onCancel={() => {}} />,
    );
    expect(html).toContain("Reprendre 3¢ + 1¢");
    expect(html).toContain("Reprendre 2¢ + 2¢");
    expect(html).toContain("Annuler");
  });
});
