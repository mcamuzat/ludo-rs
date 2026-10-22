import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ManoeverBoard, describeMax, selectionAfter } from "../Board";
import type { ManoeverAction, ManoeverView, Mover } from "../types";

const NAMES = ["Ana", "Hugo"];

function emptyStacks() {
  return Array.from({ length: 84 }, () => []) as ManoeverView["stacks"];
}

const MOVER: Mover = {
  number: 1,
  square: 47,
  position: 4,
  height: 2,
  bonus: 3,
  max: 24,
  dests: [{ Square: 48 }, { Square: 49 }],
};

function march(): ManoeverView {
  const stacks = emptyStacks();
  stacks[46] = [{ seat: 0, number: 1 }];
  stacks[49] = [
    { seat: 1, number: 1 },
    { seat: 0, number: 2 },
    { seat: 1, number: 2 },
  ];
  return {
    players: 2,
    setup: false,
    stacks,
    waiting: [0, 0],
    barracks: Array(12).fill(null),
    grays: [
      { square: 25, value: 1, takenBy: null },
      { square: 40, value: 2, takenBy: 0 },
      { square: 55, value: 3, takenBy: null },
    ],
    turn: 0,
    toMove: [MOVER],
    lastRoll: null,
    turnLog: [],
    lastTurns: [[], []],
    scores: [2, 0],
    over: false,
  };
}

function render(view: ManoeverView, legal: ManoeverAction[], me: number | null = 0): string {
  return renderToStaticMarkup(
    <ManoeverBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("Das glorreiche Manöver board", () => {
  it("offers the die during the setup", () => {
    const v: ManoeverView = { ...march(), setup: true, waiting: [6, 6], toMove: [], stacks: emptyStacks() };
    expect(render(v, ["Roll"])).toContain("Lancer le dé");
    expect(render(v, [], 1)).not.toContain("Lancer le dé");
  });

  it("marks the soldiers to move", () => {
    const html = render(march(), [{ Move: { soldier: 1, to: { Square: 48 } } }]);
    expect(html).toContain("soldier movable");
  });

  it("draws every level of a stack", () => {
    const html = render(march(), []);
    expect(html.match(/data-square="50"/g)?.length).toBe(3);
  });

  it("spells out the maximum", () => {
    expect(describeMax(MOVER)).toBe("4e × étage 2 × noire = 24");
    expect(describeMax({ ...MOVER, position: 1, height: 2, bonus: 2, max: 4 })).toBe("1re × étage 2 × blanche = 4");
    expect(describeMax({ ...MOVER, position: 3, height: 1, bonus: 1, max: 3 })).toBe("3e × étage 1 = 3");
  });

  it("drops a selection the new view no longer offers", () => {
    const v = march();
    expect(selectionAfter(v, 0, 1)).toBe(1);
    expect(selectionAfter({ ...v, toMove: [] }, 0, 1)).toBeNull();
    expect(selectionAfter({ ...v, turn: 1 }, 0, 1)).toBeNull();
    expect(selectionAfter(v, 0, null)).toBeNull();
  });
});
