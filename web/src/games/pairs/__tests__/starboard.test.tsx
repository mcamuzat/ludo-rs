import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { StarboardBoard, describeStarboardEvent } from "../starboard/Board";
import type { StarboardAction, StarboardView } from "../starboard/types";

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

const VIEW: StarboardView = {
  roundNo: 2,
  roundsTotal: 6,
  phase: "Play",
  center: [3, 8, 10, 6],
  seats: [
    { status: "In", chips: -3 },
    { status: "Locked", chips: 4 },
    { status: "Busted", chips: -1 },
  ],
  button: 1,
  turn: 0,
  pot: 6,
  events: [{ type: "Locked", seat: 1, auto: false }],
};

function board(view: StarboardView, legal: StarboardAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <StarboardBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("Starboard board", () => {
  it("submits exactly the legal actions", () => {
    const sent: unknown[] = [];
    hostElements(board(VIEW, ["Hit", "Lock"], 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toEqual(["Hit", "Lock"]);
  });

  it("shows the middle, the statuses, the pot and the balances", () => {
    const html = renderToStaticMarkup(board(VIEW, ["Hit", "Lock"], 0));
    expect(html).toContain("Manche 2 / 6");
    expect(html).toContain("Pot : 6");
    expect(html).toContain("Bloqué");
    expect(html).toContain("−3");
    expect(html).toContain("Hugo ●");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("offers nothing out of turn or watching", () => {
    expect(renderToStaticMarkup(board({ ...VIEW, turn: 2 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(VIEW, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeStarboardEvent>[0]) => describeStarboardEvent(e, NAMES);
    expect(say({ type: "Hit", seat: 2, card: 2, knocked: [0, 1] })).toBe(
      "Cléo paie 1 jeton et tire un 2 : pas de paire. Ana et Hugo sont éliminés.",
    );
    expect(say({ type: "RoundEnded", winners: [1, 2], share: 1, left: 2 })).toBe(
      "Hugo, Cléo se partagent le pot : 1 jeton chacun. 2 jetons restent au pot.",
    );
    expect(say({ type: "Locked", seat: 0, auto: true })).toBe("Ana, seul en jeu, se bloque d'office.");
  });
});
