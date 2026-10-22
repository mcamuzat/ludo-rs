import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PickupBoard, describePickupEvent } from "../pickup/Board";
import type { PickupAction, PickupView } from "../pickup/types";

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

const CHOOSE: PickupView = {
  dealNo: 1,
  target: null,
  phase: "Choose",
  dealer: 2,
  pool: [2, 5, 8, 8, 10],
  deckSize: 12,
  hand: [3, 7, 7, 9],
  chosen: null,
  taking: null,
  seats: [
    { pile: [4, 6], pilePoints: 10, handSize: 4, score: 0, waiting: null },
    { pile: [], pilePoints: 0, handSize: 4, score: 0, waiting: null },
    { pile: [9], pilePoints: 9, handSize: 4, score: 0, waiting: null },
  ],
  events: [{ type: "PoolDealt", cards: [2, 5, 8, 8, 10] }],
};

const LEGAL: PickupAction[] = [{ Choose: { card: 3 } }, { Choose: { card: 7 } }, { Choose: { card: 9 } }];

const TAKE: PickupView = {
  ...CHOOSE,
  phase: "Take",
  hand: [3, 7, 7],
  taking: [0, 9],
  seats: CHOOSE.seats.map((seat, i) => ({ ...seat, waiting: i === 1 ? 10 : null })),
};

function board(view: PickupView, legal: PickupAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <PickupBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={submit} />
  );
}

describe("Pickup board", () => {
  it("submits exactly the legal choices, one per control", () => {
    const sent: unknown[] = [];
    hostElements(board(CHOOSE, LEGAL, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toHaveLength(LEGAL.length);
    for (const action of LEGAL) expect(sent).toContainEqual(action);
  });

  it("offers the pool cards to take, one per legal rank", () => {
    const legal: PickupAction[] = [{ Take: { card: 2 } }, { Take: { card: 8 } }];
    const sent: unknown[] = [];
    hostElements(board(TAKE, legal, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toEqual(legal);
    const html = renderToStaticMarkup(board(TAKE, legal, 0));
    expect(html).toContain("Choisis la carte à prendre avec ton 9");
    expect(html).toContain("attend : 10");
  });

  it("shows the chosen card and the piles", () => {
    const html = renderToStaticMarkup(board({ ...CHOOSE, chosen: 7 }, [], 0));
    expect(html).toContain("Ta carte : 7");
    expect(html).toContain("Pile : 10 points");
    expect(html).toContain("Donne 1");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("offers nothing to a spectator or after the game", () => {
    expect(renderToStaticMarkup(board(CHOOSE, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...CHOOSE, phase: "Over" }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describePickupEvent>[0]) => describePickupEvent(e, NAMES);
    expect(say({ type: "Dealt", deal: 2, dealer: 1 })).toBe("Donne 2 : Hugo distribue.");
    expect(say({ type: "PoolDealt", cards: [1, 4, 9] })).toBe("Nouvelle réserve : 1, 4, 9.");
    expect(say({ type: "Revealed", cards: [5, null, 8] })).toBe("On révèle : Ana 5, Cléo 8.");
    expect(say({ type: "Took", seat: 0, played: 8, card: 8, knocked: [[1, 2]], pool_knocked: 1 })).toBe(
      "Ana joue un 8 et prend un 8 ; éliminés : 2 chez Hugo, 1 dans la réserve.",
    );
    expect(say({ type: "Took", seat: 2, played: 9, card: 4, knocked: [], pool_knocked: 0 })).toBe(
      "Cléo joue un 9 et prend un 4.",
    );
    expect(say({ type: "Stacked", seat: 1, card: 3 })).toBe("Hugo n'a rien à prendre et garde son 3.");
    expect(say({ type: "DealOver", points: [12, 0, 7], scores: [12, 0, 7] })).toBe(
      "Fin de la donne : Ana +12 (12), Hugo +0 (0), Cléo +7 (7).",
    );
  });
});
