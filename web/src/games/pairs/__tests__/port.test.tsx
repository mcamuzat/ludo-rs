import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PortBoard, describePortEvent } from "../port/Board";
import type { PortAction, PortView } from "../port/types";

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

const NAMES = ["Ana", "Hugo", "Cléo"];

const VIEW: PortView = {
  roundNo: 3,
  target: 21,
  phase: "Play",
  seats: [
    { stack: [4, 9], total: 13, status: "Playing", score: 5 },
    { stack: [2, 6, 6], total: 14, status: "Busted", score: 8 },
    { stack: [10, 7], total: 17, status: "Locked", score: 2 },
  ],
  turn: 0,
  deck: 30,
  discard: 12,
  events: [{ type: "Locked", seat: 2 }],
};

function board(view: PortView, legal: PortAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <PortBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("Port board", () => {
  it("submits exactly the legal actions", () => {
    const sent: unknown[] = [];
    hostElements(board(VIEW, ["Hit", "Lock"], 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toEqual(["Hit", "Lock"]);
  });

  it("shows stacks, totals, statuses and scores", () => {
    const html = renderToStaticMarkup(board(VIEW, ["Hit", "Lock"], 0));
    expect(html).toContain("Manche 3");
    expect(html).toContain("8 / 21");
    expect(html).toContain("Total 17 · Arrêté");
    expect(html).toContain("Paire !");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("offers nothing out of turn or watching", () => {
    expect(renderToStaticMarkup(board({ ...VIEW, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(VIEW, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describePortEvent>[0]) => describePortEvent(e, NAMES);
    expect(say({ type: "RoundStarted", round: 4, seats: [0, 2], first: 2 })).toBe(
      "Manche 4 entre Ana et Cléo. Cléo commence.",
    );
    expect(say({ type: "Busted", seat: 1, card: 6 })).toBe("Hugo tire un 6 : paire, éliminé de la manche.");
    expect(say({ type: "RoundScored", totals: [17, null, 5], points: [3, 0, 2], scores: [3, 0, 2] })).toBe(
      "Fin de manche : Ana 17 → 3 points, Cléo 5 → 2 points.",
    );
  });
});
