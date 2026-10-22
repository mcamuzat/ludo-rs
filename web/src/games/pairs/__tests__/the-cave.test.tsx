import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { TheCaveBoard, describeTheCaveEvent } from "../the-cave/Board";
import type { TheCaveAction, TheCaveView } from "../the-cave/types";

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

const PLAY: TheCaveView = {
  roundNo: 2,
  rounds: 3,
  phase: "Play",
  dealer: 2,
  turn: 0,
  cave: [4, 7, 7, 8, 9],
  deckSize: 21,
  hand: [2, 8, 8, 10],
  seats: [
    { captured: [7, 7, 10], handSize: 4, score: 12 },
    { captured: [], handSize: 3, score: 3 },
    { captured: [5], handSize: 3, score: 4 },
  ],
  events: [{ type: "Played", seat: 2, card: 6, captured: [5] }],
};

const LEGAL: TheCaveAction[] = [{ Play: { card: 2 } }, { Play: { card: 8 } }, { Play: { card: 10 } }];

function board(view: TheCaveView, legal: TheCaveAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <TheCaveBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("The Cave board", () => {
  it("submits exactly the legal actions, one per control", () => {
    const sent: unknown[] = [];
    hostElements(board(PLAY, LEGAL, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toHaveLength(LEGAL.length);
    for (const action of LEGAL) expect(sent).toContainEqual(action);
  });

  it("shows the cave, the deck and the captures", () => {
    const html = renderToStaticMarkup(board(PLAY, LEGAL, 0));
    expect(html).toContain("Manche 2 sur 3");
    expect(html).toContain("La grotte");
    expect(html).toContain("pioche");
    expect(html).toContain("Capturé : 3 cartes");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeTheCaveEvent>[0]) => describeTheCaveEvent(e, NAMES);
    expect(say({ type: "Dealt", round: 1, dealer: 2, first: 0 })).toBe("Manche 1 : Cléo distribue, Ana commence.");
    expect(say({ type: "Played", seat: 0, card: 8, captured: [7, 7] })).toBe("Ana pose un 8 et capture 7, 7.");
    expect(say({ type: "Played", seat: 1, card: 3, captured: [] })).toBe("Hugo pose un 3 dans la grotte vide.");
    expect(say({ type: "Refilled", cards: [2, 6] })).toBe("La grotte est complétée : 2, 6.");
    expect(say({ type: "RoundOver", points: [4, 0, 2], scores: [12, 3, 6] })).toBe(
      "Fin de la manche : Ana +4 (12), Hugo +0 (3), Cléo +2 (6).",
    );
  });
});
