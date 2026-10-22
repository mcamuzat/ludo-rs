import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { BreadBasketBoard, describeBreadBasketEvent } from "../bread-basket/Board";
import type { BreadBasketAction, BreadBasketView } from "../bread-basket/types";

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

const PLAY: BreadBasketView = {
  roundNo: 2,
  rounds: 6,
  phase: "Play",
  dealer: 2,
  turn: 0,
  line: [10, 9, 4],
  deckSize: 21,
  discard: [6, 6],
  hand: [-5, 4, 4, 8],
  seats: [
    { taken: [3, -5], takenPoints: -2, handSize: 4, score: 12, out: false },
    { taken: [], takenPoints: 0, handSize: 0, score: -3, out: true },
    { taken: [7], takenPoints: 7, handSize: 5, score: 4, out: false },
  ],
  events: [{ type: "Played", seat: 2, card: 4 }],
};

const LEGAL: BreadBasketAction[] = [{ Play: { card: -5 } }, { Play: { card: 4 } }, { Play: { card: 8 } }, "Draw"];

function board(view: BreadBasketView, legal: BreadBasketAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <BreadBasketBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("Bread Basket board", () => {
  it("submits exactly the legal actions, one per control", () => {
    const sent: unknown[] = [];
    hostElements(board(PLAY, LEGAL, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toHaveLength(LEGAL.length);
    for (const action of LEGAL) expect(sent).toContainEqual(action);
  });

  it("shows the line, the piles, the picked-up cards and the scores", () => {
    const html = renderToStaticMarkup(board(PLAY, LEGAL, 0));
    expect(html).toContain("Main 2 sur 6");
    expect(html).toContain("pioche");
    expect(html).toContain("Ramassé : −2 points");
    expect(html).toContain("a plus de cartes");
    expect(html).toContain("Piocher");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("draws an Onion as −5", () => {
    const html = renderToStaticMarkup(board(PLAY, LEGAL, 0));
    expect(html).toMatch(/class="pairs-card onion[^"]*"[^>]*>−5</);
    expect(html).toContain('title="Oignon"');
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeBreadBasketEvent>[0]) => describeBreadBasketEvent(e, NAMES);
    expect(say({ type: "Dealt", round: 3, dealer: 1, first: 2 })).toBe("Main 3 : Hugo distribue, Cléo commence.");
    expect(say({ type: "Played", seat: 0, card: -5 })).toBe("Ana pose un Oignon.");
    expect(say({ type: "Sandwich", seat: 0, bread: 9, filling: [8, 6], points: 14 })).toBe(
      "Ana fait un sandwich au 9 : 8, 6 (+14).",
    );
    expect(say({ type: "Air", seat: 1, card: 7 })).toBe("Hugo fait un sandwich d'air de 7 et doit piocher.");
    expect(say({ type: "WentOut", seat: 2 })).toBe("Cléo n'a plus de cartes : un dernier tour pour les autres.");
    expect(
      say({ type: "HandOver", end: "DeckEmpty", gained: [14, 0, -5], penalties: [3, 0, 10], scores: [11, 0, -15] }),
    ).toBe("Fin de la main (pioche vide) : Ana +14 − 3 (11), Hugo +0 − 0 (0), Cléo −5 − 10 (−15).");
  });
});
