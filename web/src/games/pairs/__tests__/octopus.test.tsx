import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { OctopusBoard, describeOctopusEvent } from "../octopus/Board";
import type { OctopusAction, OctopusView } from "../octopus/types";

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

const PLAY: OctopusView = {
  handNo: 2,
  handsTotal: 5,
  phase: "Play",
  seats: [
    { handSize: 6, folded: false, chips: 4 },
    { handSize: 0, folded: true, chips: -3 },
    { handSize: 6, folded: false, chips: -6 },
    { handSize: 6, folded: false, chips: 1 },
  ],
  hand: [2, 2, 5, 7, 9, 9],
  placed: null,
  passed: 8,
  received: 9,
  played: [3, 6, 6],
  turn: null,
  leader: 0,
  pot: 9,
  events: [{ type: "Passed" }],
};

const PLAY_LEGAL: OctopusAction[] = [2, 5, 7, 9].map((card) => ({ Play: { card } }));

function board(view: OctopusView, legal: OctopusAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <OctopusBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: OctopusView, legal: OctopusAction[]): unknown[] {
  const sent: unknown[] = [];
  hostElements(board(view, legal, 0, (a) => sent.push(a)))
    .filter((el) => typeof el.props.onClick === "function")
    .forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Octopus board", () => {
  it("submits exactly the legal actions, one per control", () => {
    const cases: [OctopusView, OctopusAction[]][] = [
      [PLAY, PLAY_LEGAL],
      [{ ...PLAY, phase: "Pass" }, [2, 5, 7, 9].map((card) => ({ Pass: { card } }))],
      [{ ...PLAY, phase: "Declare", turn: 0 }, ["Stay", "Fold"]],
      [
        { ...PLAY, phase: "Call", turn: 0 },
        ["CallPass", { Show: { best: "Set" } }, { Show: { best: "Clean" } }],
      ],
    ];
    for (const [view, legal] of cases) {
      const sent = clickEverything(view, legal);
      expect(sent).toHaveLength(legal.length);
      for (const action of legal) expect(sent).toContainEqual(action);
    }
  });

  it("shows the hand, the pot, the balances and the passes", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Main 2 / 5");
    expect(html).toContain("Pot : 9");
    expect(html).toContain("+4");
    expect(html).toContain("−6");
    expect(html).toContain("Couché");
    expect(html).toContain("Jouer ce 5");
    expect(html).toContain("Tu as passé un 8, tu as reçu un 9.");
    const call = renderToStaticMarkup(board({ ...PLAY, phase: "Call", turn: 0 }, [{ Show: { best: "Set" } }, "CallPass"], 0));
    expect(call).toContain("Abattage, mains de séries gagnantes");
  });

  it("offers nothing while waiting, watching, or after the game", () => {
    const waiting = renderToStaticMarkup(board({ ...PLAY, placed: 7 }, [], 0));
    expect(waiting).not.toContain("<button");
    expect(waiting).toContain("Ton choix : 7");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over" }, [], 0));
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeOctopusEvent>[0]) => describeOctopusEvent(e, NAMES);
    expect(say({ type: "HandStarted", hand: 2, antes: [1, 5, 1, 1], leader: 0 })).toBe(
      "Main 2. Mises : Ana 1, Hugo 5, Cléo 1, Malo 1. Ana parle en premier.",
    );
    expect(say({ type: "Declared", seat: 1, fold: true })).toBe("Hugo se couche (1 jeton).");
    expect(say({ type: "CardsPlayed", cards: [5, null, 6, 5], caller: 2 })).toBe(
      "Cartes jouées : Ana 5, Cléo 6, Malo 5 ; Cléo a la plus basse carte unique.",
    );
    expect(
      say({
        type: "Showdown",
        caller: 0,
        best: "Set",
        hands: [[2, 2, 3, 3], null, [2, 4, 5, 6], [7, 7, 8, 9]],
        winners: [0],
        losers: [3],
        share: 8,
      }),
    ).toBe(
      "Abattage. Ana choisit les mains de séries. Mains : Ana 2-2-3-3, Cléo 2-4-5-6, Malo 7-7-8-9. " +
        "Ana prend le pot (8 jetons). Malo paie 5 jetons.",
    );
  });
});
