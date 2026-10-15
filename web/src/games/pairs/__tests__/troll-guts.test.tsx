import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { StackOrderPicker, TrollGutsBoard, describeTrollGutsEvent, matchingOrder } from "../troll-guts/Board";
import type { TrollGutsAction, TrollGutsView } from "../troll-guts/types";

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

const PLAY: TrollGutsView = {
  handNo: 2,
  handsTotal: 5,
  phase: "Play",
  seats: [
    { faceup: [3, 8], stackSize: 4, out: false, chips: -1 },
    { faceup: [2], stackSize: 5, out: false, chips: 3 },
    { faceup: [], stackSize: 0, out: true, chips: -9 },
  ],
  stack: [7, 1, 9, 9],
  placed: null,
  turn: 0,
  pot: 4,
  deck: 30,
  events: [{ type: "Took", seat: 1, card: 2, from: { kind: "Deck" } }],
};

const LEGAL: TrollGutsAction[] = ["Draw", { Steal: { from: 1 } }, "Buy"];

function board(view: TrollGutsView, legal: TrollGutsAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <TrollGutsBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function click(node: ReactNode) {
  hostElements(node)
    .filter((el) => typeof el.props.onClick === "function" && !el.props.disabled)
    .forEach((el) => (el.props.onClick as () => void)());
}

describe("Troll Guts board", () => {
  it("shows the moves, the faces and the stacks", () => {
    const html = renderToStaticMarkup(board(PLAY, LEGAL, 0));
    expect(html).toContain("Main 2 / 5 · Pot : 4");
    expect(html).toContain("Piocher");
    expect(html).toContain("Acheter mon 7 (1 jeton)");
    expect(html).toContain("Voler le dessus");
    expect(html).toContain("Éliminé");
    expect(html).toContain("Ta pile (dessus d&#x27;abord)");
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 1 }, [], 0))).not.toContain("<button");
  });

  it("matches a full pick to a legal order and submits it", () => {
    const stack = [4, 6, 6];
    const legal: TrollGutsAction[] = [
      { Arrange: { order: [4, 6, 6] } },
      { Arrange: { order: [6, 4, 6] } },
      { Arrange: { order: [6, 6, 4] } },
    ];
    expect(matchingOrder(legal, stack, [1, 0, 2])).toEqual({ Arrange: { order: [6, 4, 6] } });
    expect(matchingOrder(legal, stack, [1, 0])).toBeUndefined();
    const sent: unknown[] = [];
    const toggled: number[] = [];
    let reset = 0;
    click(
      <StackOrderPicker
        stack={stack}
        legal={legal}
        picked={[2, 1, 0]}
        onToggle={(k) => toggled.push(k)}
        onReset={() => reset++}
        submit={(a) => sent.push(a)}
      />,
    );
    expect(sent).toEqual([{ Arrange: { order: [6, 6, 4] } }]);
    expect(toggled).toEqual([]);
    expect(reset).toBe(1);
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeTrollGutsEvent>[0]) => describeTrollGutsEvent(e, NAMES);
    expect(say({ type: "Took", seat: 0, card: 5, from: { kind: "Stack", seat: 1 } })).toBe(
      "Ana vole le dessus de la pile de Hugo : un 5.",
    );
    expect(say({ type: "Took", seat: 0, card: 5, from: { kind: "Stack", seat: 0 } })).toBe(
      "Ana achète le dessus de sa pile : un 5.",
    );
    expect(say({ type: "Busted", seat: 2, card: 8, paid: 8 })).toBe("Cléo prend un 8 : paire, paie 8 jetons et sort.");
  });
});
