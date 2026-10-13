import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ContinuousBoard, canHit, describeEvent, foldTargets } from "../continuous/Board";
import type { ContinuousAction, ContinuousView } from "../continuous/types";

const NAMES = ["Ana", "Bob", "Cléo"];

const VIEW: ContinuousView = {
  seats: [
    { stack: [10, 7], scored: [3], score: 3 },
    { stack: [4], scored: [], score: 0 },
    { stack: [], scored: [8, 2], score: 10 },
  ],
  target: 21,
  turn: 0,
  loser: null,
  deck: 31,
  discard: 12,
  unlocated: [1, 1, 2, 3, 4, 5, 5, 6, 8, 8],
  lowest: 4,
  events: [
    { type: "Paired", seat: 2, card: 8 },
    { type: "Reshuffled" },
  ],
};

const LEGAL: ContinuousAction[] = [
  "Hit",
  { Fold: { seat: 0, card: 10 } },
  { Fold: { seat: 0, card: 7 } },
  { Fold: { seat: 1, card: 4 } },
];

function render(view: ContinuousView, legal: ContinuousAction[], me: number | null) {
  return renderToStaticMarkup(
    <ContinuousBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={() => {}}
    />,
  );
}

type AnyElement = ReactElement<Record<string, unknown>>;

/** Every host element of a tree, resolving the function components (none uses hooks). */
function hostElements(node: ReactNode, out: AnyElement[] = []): AnyElement[] {
  if (Array.isArray(node)) {
    node.forEach((child) => hostElements(child, out));
    return out;
  }
  if (!isValidElement(node)) return out;
  const el = node as AnyElement;
  if (typeof el.type === "function") {
    return hostElements((el.type as (props: unknown) => ReactNode)(el.props), out);
  }
  out.push(el);
  return hostElements(el.props.children as ReactNode, out);
}

describe("Continuous Pairs board", () => {
  it("submits exactly the legal actions, one per control", () => {
    const sent: unknown[] = [];
    const board = (
      <ContinuousBoard
        view={VIEW}
        legalActions={LEGAL}
        pending={[0]}
        me={0}
        seatNames={NAMES}
        submit={(action) => sent.push(action)}
      />
    );
    const controls = hostElements(board).filter((el) => typeof el.props.onClick === "function");
    controls.forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toHaveLength(LEGAL.length);
    for (const action of LEGAL) expect(sent).toContainEqual(action);
  });

  it("reads the moves from the legal actions and nowhere else", () => {
    expect(canHit(LEGAL)).toBe(true);
    expect(canHit([{ Fold: { seat: 1, card: 4 } }])).toBe(false);
    expect(foldTargets(LEGAL)).toEqual([
      { seat: 0, card: 10 },
      { seat: 0, card: 7 },
      { seat: 1, card: 4 },
    ]);
    expect(foldTargets([])).toEqual([]);
  });

  it("offers a hit and one button per card that can be taken", () => {
    const html = render(VIEW, LEGAL, 0);
    expect(html).toContain("Prendre une carte");
    expect(html.match(/<button[^>]*pairs-card/g)).toHaveLength(3);
    expect(html).toContain("3 / 21");
    expect(html).not.toContain("Risque de paire");
    expect(html).toContain("Se coucher coûte au moins 4");
  });

  it("offers nothing to a player who is not to move", () => {
    const html = render(VIEW, [], 1);
    expect(html).not.toContain("<button");
    expect(html).toContain("Ana");
  });

  it("shows the loser and offers nothing once the game is over", () => {
    const over: ContinuousView = { ...VIEW, turn: null, loser: 2, events: [] };
    const html = render(over, [], 0);
    expect(html).toContain("Cléo a perdu");
    expect(html).not.toContain("<button");
  });

  it("seats eight players", () => {
    const names = ["A", "B", "C", "D", "E", "F", "G", "H"];
    const eight: ContinuousView = {
      ...VIEW,
      seats: names.map(() => ({ stack: [5], scored: [], score: 0 })),
      target: 11,
    };
    const html = renderToStaticMarkup(
      <ContinuousBoard view={eight} legalActions={[]} pending={[0]} me={null} seatNames={names} submit={() => {}} />,
    );
    expect(html.match(/class="pairs-seat/g)).toHaveLength(8);
  });

  it("tells what happened in words", () => {
    expect(describeEvent({ type: "Dealt", seat: 1, card: 6 }, NAMES)).toBe("Bob reçoit un 6.");
    expect(describeEvent({ type: "Paired", seat: 2, card: 8 }, NAMES)).toBe("Cléo fait une paire de 8 : 8 points.");
    expect(describeEvent({ type: "Folded", seat: 0, from: 1, card: 4 }, NAMES)).toBe(
      "Ana se couche et prend le 4 de Bob : 4 points.",
    );
    expect(describeEvent({ type: "Folded", seat: 0, from: 0, card: 7 }, NAMES)).toBe(
      "Ana se couche et garde son 7 : 7 points.",
    );
    expect(describeEvent({ type: "Replaced", seat: 0, card: 4 }, NAMES)).toBe(
      "Départage : le 4 d'Ana ferait une paire, il est remplacé.",
    );
    expect(describeEvent({ type: "Reshuffled" }, NAMES)).toBe("La défausse est remélangée, 5 cartes sont brûlées.");
  });
});
