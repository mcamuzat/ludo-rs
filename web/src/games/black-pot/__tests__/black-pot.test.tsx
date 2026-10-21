import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import board from "../board.json";
import {
  assign,
  BlackPotBoard,
  clickDie,
  clickOp,
  describeTurn,
  draftKey,
  EMPTY_DRAFT,
  exprText,
  swap,
  toAction,
  usedDice,
} from "../Board";
import type { BlackPotAction, BlackPotView, Expr } from "../types";

const NAMES = ["Ana", "Hugo", "Cléo", "Léon"];
const DICE = [4, 2, 6];

const VIEW: BlackPotView = {
  players: 2,
  pawns: board.cells.map((_, i) => (i === 3 ? { team: 1, flipped: false } : null)),
  dice: DICE,
  roller: 0,
  turn: 0,
  queue: [0],
  teams: [
    { seats: [0], pawnsLeft: 40, score: 400 },
    { seats: [1], pawnsLeft: 39, score: 390 },
  ],
  alignments: [],
  last: [null, null],
  quietLeft: 10,
  turns: 0,
  end: null,
  winners: [],
};

const LEGAL: BlackPotAction[] = [{ Play: { groups: [] } }, "NotFound"];

function render(view: BlackPotView, legal: BlackPotAction[], me: number | null = 0): string {
  return renderToStaticMarkup(
    <BlackPotBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

const die = (i: number): Expr => ({ Die: i });

describe("composing a calculation", () => {
  it("builds 6 ÷ (4 − 2) on one cell", () => {
    let d = clickDie(EMPTY_DRAFT, 0);
    d = clickOp(d, "Sub");
    d = clickDie(d, 1);
    d = clickOp(d, "Div");
    d = clickDie(d, 2);
    d = swap(d);
    expect(exprText(d.current!, DICE)).toBe("6 ÷ (4 − 2)");
    expect(toAction(d)).toBeNull();
    d = assign(d, 42);
    expect(usedDice(d)).toEqual([0, 1, 2]);
    expect(toAction(d)).toEqual({
      Play: {
        groups: [
          {
            expr: { Op: { op: "Div", left: die(2), right: { Op: { op: "Sub", left: die(0), right: die(1) } } } },
            cell: 42,
          },
        ],
      },
    });
  });

  it("plays three dice apart, and waits until every die is used", () => {
    let d = assign(clickDie(EMPTY_DRAFT, 0), 5);
    d = assign(clickDie(d, 1), 6);
    expect(toAction(d)).toBeNull();
    d = assign(clickDie(d, 2), null);
    expect(toAction(d)).toEqual({
      Play: {
        groups: [
          { expr: die(0), cell: 5 },
          { expr: die(1), cell: 6 },
          { expr: die(2), cell: null },
        ],
      },
    });
  });

  it("ignores a used die, an operation without a number, a die without an operation", () => {
    const one = assign(clickDie(EMPTY_DRAFT, 0), 5);
    expect(clickDie(one, 0)).toEqual(one);
    expect(clickOp(EMPTY_DRAFT, "Add")).toEqual(EMPTY_DRAFT);
    const started = clickDie(EMPTY_DRAFT, 1);
    expect(clickDie(started, 2)).toEqual(started);
    expect(assign(clickOp(started, "Add"), 5)).toEqual(clickOp(started, "Add"));
  });

  it("writes calculations with brackets, computing nothing", () => {
    const e: Expr = { Op: { op: "Mul", left: { Op: { op: "Add", left: die(0), right: die(1) } }, right: die(2) } };
    expect(exprText(e, DICE)).toBe("(4 + 2) × 6");
    expect(exprText(die(1), DICE)).toBe("2");
  });
});

describe("the board", () => {
  it("highlights no cell and offers « Je ne trouve pas » on my turn only", () => {
    const mine = render(VIEW, LEGAL, 0);
    expect(mine).toContain("Je ne trouve pas");
    expect(mine).not.toContain("target");
    expect(render(VIEW, [], 1)).not.toContain("Je ne trouve pas");
  });

  it("keeps a draft for the same roll and turn only", () => {
    expect(draftKey(VIEW)).toBe(draftKey({ ...VIEW }));
    expect(draftKey(VIEW)).not.toBe(draftKey({ ...VIEW, dice: [1, 2, 3] }));
    expect(draftKey(VIEW)).not.toBe(draftKey({ ...VIEW, turn: 1 }));
    // The same dice rolled again later: a new turn all the same.
    expect(draftKey(VIEW)).not.toBe(draftKey({ ...VIEW, turns: VIEW.turns + 2 }));
  });

  it("tells what each player did", () => {
    expect(
      describeTurn(
        { Played: { dice: DICE, groups: [{ text: "6 × 4 = 24 ÷ 2 = 12", value: 12, cell: 9 }], points: 250 } },
        "Ana",
      ),
    ).toBe("Ana : 6 × 4 = 24 ÷ 2 = 12 (250 points)");
    expect(describeTurn({ NotFound: { dice: DICE } }, "Hugo")).toBe("Hugo ne trouve pas (4, 2, 6).");
    expect(describeTurn({ Passed: { dice: DICE } }, "Hugo")).toBe("Hugo n'a aucun coup avec 4, 2, 6.");
    expect(
      describeTurn({ Played: { dice: DICE, groups: [{ text: "2 + 4 = 6 − 6 = 0", value: 0, cell: null }], points: 0 } }, "Ana"),
    ).toBe("Ana : 2 + 4 = 6 − 6 = 0");
  });
});
