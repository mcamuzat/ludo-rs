import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { BlackstoneBoard, TakeControls, describeBlackstoneEvent } from "../blackstone/Board";
import type { BlackstoneAction, BlackstoneView } from "../blackstone/types";

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

const VIEW: BlackstoneView = {
  handNo: 1,
  handsTotal: 5,
  phase: "Play",
  seats: [
    { blacks: 1, status: "In", chips: -5 },
    { blacks: 0, status: "In", chips: -5 },
    { blacks: 0, status: "Folded", chips: -9 },
  ],
  whites: [2, 3, 5, 9, 9, 4, 7],
  minTake: 2,
  foldCost: 9,
  turn: 0,
  pot: 9,
  ante: 5,
  deck: 41,
  events: [{ type: "Took", seat: 1, count: 2, cards: [9, 4] }],
};

const LEGAL: BlackstoneAction[] = [{ Take: { count: 2 } }, { Take: { count: 3 } }, { Take: { count: 4 } }, "Fold"];

describe("Blackstone board", () => {
  it("submits the chosen count or the fold", () => {
    const sent: unknown[] = [];
    hostElements(
      <TakeControls legal={LEGAL} count={3} foldCost={9} onCount={() => {}} submit={(a) => sent.push(a)} />,
    )
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toEqual([{ Take: { count: 3 } }, "Fold"]);
    const fallback: unknown[] = [];
    hostElements(
      <TakeControls legal={LEGAL} count={null} foldCost={9} onCount={() => {}} submit={(a) => fallback.push(a)} />,
    )
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(fallback[0]).toEqual({ Take: { count: 2 } });
  });

  it("shows the stones, the pot and the fold cost", () => {
    const html = renderToStaticMarkup(
      <BlackstoneBoard view={VIEW} legalActions={LEGAL} pending={[0]} me={0} seatNames={NAMES} submit={() => {}} />,
    );
    expect(html).toContain("Main 1 / 5 · Pot : 9");
    expect(html).toContain("7 pierres blanches");
    expect(html).toContain("1 pierre noire");
    expect(html).toContain("Se coucher (9 jetons)");
    expect(html).toContain("Couché");
    const watching = renderToStaticMarkup(
      <BlackstoneBoard view={VIEW} legalActions={[]} pending={[0]} me={null} seatNames={NAMES} submit={() => {}} />,
    );
    expect(watching).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeBlackstoneEvent>[0]) => describeBlackstoneEvent(e, NAMES);
    expect(say({ type: "Took", seat: 0, count: 3, cards: [5, 10] })).toBe("Ana annonce 3 et tire 5, 10.");
    expect(say({ type: "Lost", seat: 0, paid: 9 })).toBe("Ana a deux pierres noires : perdu, 9 jetons au pot.");
    expect(say({ type: "PotWon", seats: [1, 2], share: 4, left: 1 })).toBe(
      "Hugo, Cléo reprennent leur mise et partagent le pot : 4 jetons chacun. 1 jeton reste au pot.",
    );
  });
});
