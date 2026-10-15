import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RocketBoard, describeRocketEvent } from "../rocket/Board";
import type { RocketAction, RocketView } from "../rocket/types";

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

const NAMES = ["Ana", "Hugo"];

const VIEW: RocketView = {
  roundNo: 4,
  roundsTotal: 10,
  phase: "Play",
  stacks: [
    [8, 4],
    [2, 2],
  ],
  status: ["In", "Busted"],
  dealer: [5, 9],
  turn: 0,
  chips: [6, -9],
  foldCost: 4,
  deck: 31,
  events: [{ type: "DealerHit", card: 9 }],
};

function board(view: RocketView, legal: RocketAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <RocketBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("Rocket board", () => {
  it("submits exactly the legal actions", () => {
    const sent: unknown[] = [];
    hostElements(board(VIEW, ["Hit", "Fold"], 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toEqual(["Hit", "Fold"]);
  });

  it("shows the stacks, the dealer, the balances and the fold cost", () => {
    const html = renderToStaticMarkup(board(VIEW, ["Hit", "Fold"], 0));
    expect(html).toContain("Manche 4 / 10");
    expect(html).toContain("Se coucher (4 jetons)");
    expect(html).toContain("+6");
    expect(html).toContain("−9");
    expect(html).toContain("Paire !");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("offers nothing out of turn or watching", () => {
    expect(renderToStaticMarkup(board({ ...VIEW, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(VIEW, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeRocketEvent>[0]) => describeRocketEvent(e, NAMES);
    expect(say({ type: "DealerLost", card: 7, seats: [0], amount: 7 })).toBe(
      "La banque tire une paire de 7 : elle paie 7 jetons à Ana.",
    );
    expect(say({ type: "DealerLost", card: 3, seats: [0, 1], amount: 3 })).toBe(
      "La banque tire un 3, carte fatale : elle paie 3 jetons à Ana, Hugo.",
    );
    expect(say({ type: "Folded", seat: 1, paid: 1 })).toBe("Hugo se couche et paie 1 jeton.");
  });
});
