import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { KittyBoard, describeKittyEvent } from "../kitty/Board";
import type { KittyAction, KittyView } from "../kitty/types";

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

const NAMES = ["Ana", "Hugo", "Cléo", "Malo"];

const BID: KittyView = {
  roundNo: 3,
  roundsTotal: 7,
  soft: false,
  phase: "Bid",
  handSizes: [11, 11, 11, 12],
  hand: [1, 4, 4, 6, 8, 9, 9, 10, 10, 10, 7],
  placed: null,
  passed: 6,
  received: 7,
  target: [9],
  kittySize: 4,
  discard: [3, 5, 5, 8, 2],
  turn: null,
  scores: [],
  events: [{ type: "TargetRevealed", round: 3, cards: [9] }],
};

const BID_LEGAL: KittyAction[] = [1, 4, 6, 7, 8, 9, 10].map((card) => ({ Bid: { card } }));

function board(view: KittyView, legal: KittyAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <KittyBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: KittyView, legal: KittyAction[]): unknown[] {
  const sent: unknown[] = [];
  hostElements(board(view, legal, 0, (a) => sent.push(a)))
    .filter((el) => typeof el.props.onClick === "function")
    .forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Kitty board", () => {
  it("submits exactly the legal bids, one per rank", () => {
    const sent = clickEverything(BID, BID_LEGAL);
    expect(sent).toHaveLength(BID_LEGAL.length);
    for (const action of BID_LEGAL) expect(sent).toContainEqual(action);
  });

  it("submits exactly the legal gifts and takes", () => {
    const claim: KittyView = { ...BID, phase: "Claim", turn: 0, soft: true, target: [4, 10] };
    const gives: KittyAction[] = [4, 10].flatMap((card) => [1, 2, 3].map((to) => ({ Give: { card, to } })));
    const sent = clickEverything(claim, gives);
    expect(sent).toHaveLength(6);
    for (const action of gives) expect(sent).toContainEqual(action);
    const takes: KittyAction[] = [{ Take: { card: 4 } }, { Take: { card: 10 } }];
    expect(clickEverything(claim, takes)).toEqual(takes);
  });

  it("shows the round, the target, what I passed and received", () => {
    const html = renderToStaticMarkup(board(BID, BID_LEGAL, 0));
    expect(html).toContain("Manche 3 / 7");
    expect(html).toContain("Miser ce 4");
    expect(html).toContain("Tu as passé un 6, tu as reçu un 7.");
    const pass = renderToStaticMarkup(board({ ...BID, phase: "Pass" }, [{ Pass: { card: 4 } }], 0));
    expect(pass).toContain("Passer à gauche ce 4");
    const waiting = renderToStaticMarkup(board({ ...BID, placed: 8 }, [], 0));
    expect(waiting).not.toContain("<button");
    expect(waiting).toContain("Ton choix : 8");
  });

  it("shows every hand and the scores at the end", () => {
    const over: KittyView = {
      ...BID,
      phase: "Over",
      target: [],
      scores: [20, 13, 14, 17],
      events: [
        {
          type: "Scored",
          hands: [[10, 10], [9], [3], [2]],
          ranks: [],
          scores: [20, 13, 14, 17],
        },
      ],
    };
    const html = renderToStaticMarkup(board(over, [], 0));
    expect(html).toContain("Partie terminée");
    expect(html).toContain("20 points");
    expect(html).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeKittyEvent>[0]) => describeKittyEvent(e, NAMES);
    expect(say({ type: "TargetRevealed", round: 2, cards: [4, 10] })).toBe("Manche 2 : cibles 4 et 10.");
    expect(say({ type: "BidsRevealed", bids: [2, 2, 5, 7], winners: [2] })).toBe(
      "Mises : Ana 2, Hugo 2, Cléo 5, Malo 7 ; gagnant : Cléo.",
    );
    expect(say({ type: "BidsRevealed", bids: [3, 3, 6, 6], winners: [] })).toBe(
      "Mises : Ana 3, Hugo 3, Cléo 6, Malo 6 ; aucune mise unique.",
    );
    expect(say({ type: "Gave", seat: 0, to: 3, card: 10 })).toBe("Ana a gagné avec le 1 et donne le 10 à Malo.");
    expect(say({ type: "Discarded", cards: [10] })).toBe("Personne ne prend le 10 : défaussé.");
  });
});
