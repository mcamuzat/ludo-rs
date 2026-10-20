import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { TheJudgeBoard, describeTheJudgeEvent } from "../the-judge/Board";
import type { TheJudgeAction, TheJudgeView } from "../the-judge/types";

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

const COOK: TheJudgeView = {
  roundNo: 3,
  target: 50,
  phase: "Cook",
  seats: [
    { recipe: [4, 9], recipePoints: 13, handSize: 5, passed: false, score: 21 },
    { recipe: [2], recipePoints: 2, handSize: 6, passed: true, score: 8 },
    { recipe: [], recipePoints: 0, handSize: 7, passed: false, score: 30 },
  ],
  hand: [3, 4, 7, 7, 10],
  addable: [3, 7, 10],
  judgeSize: 8,
  leader: 2,
  turn: 0,
  events: [{ type: "Added", seat: 2, card: 6 }],
};

const LEGAL: TheJudgeAction[] = [{ Add: { card: 3 } }, { Add: { card: 7 } }, { Add: { card: 10 } }, "Pass"];

function board(view: TheJudgeView, legal: TheJudgeAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <TheJudgeBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("The Judge board", () => {
  it("submits exactly the legal actions, one per control", () => {
    const sent: unknown[] = [];
    hostElements(board(COOK, LEGAL, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toHaveLength(LEGAL.length);
    for (const action of LEGAL) expect(sent).toContainEqual(action);
  });

  it("shows the recipes, the Judge and the scores", () => {
    const html = renderToStaticMarkup(board(COOK, LEGAL, 0));
    expect(html).toContain("Manche 3 · Partie en 50 points");
    expect(html).toContain("Recette : 13 points");
    expect(html).toContain("a fini");
    expect(html).toContain("le Juge");
    expect(html).toContain("Ajouter ce 7 à ta recette");
    expect(html).not.toContain("Ajouter ce 4");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...COOK, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(COOK, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...COOK, phase: "Over", turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeTheJudgeEvent>[0]) => describeTheJudgeEvent(e, NAMES);
    expect(say({ type: "Dealt", round: 2, leader: 1 })).toBe("Manche 2 : 7 cartes chacun, 8 pour le Juge. Hugo commence.");
    expect(say({ type: "Passed", seat: 2 })).toBe("Cléo passe.");
    expect(
      say({ type: "Contest", reveals: [{ card: 7, eliminated: [0, 1, 2] }], survivors: [], gained: [0, 0, 0], scores: [1, 2, 3] }),
    ).toBe("Le Juge retourne : 7 (élimine Ana, Hugo et Cléo). Personne ne marque.");
    expect(
      say({
        type: "Contest",
        reveals: [
          { card: 5, eliminated: [] },
          { card: 6, eliminated: [0, 2] },
        ],
        survivors: [1],
        gained: [0, 11, 0],
        scores: [21, 19, 30],
      }),
    ).toBe("Le Juge retourne : 5, 6 (élimine Ana et Cléo). Marquent : Hugo +11 (19).");
    expect(say({ type: "Contest", reveals: [], survivors: [], gained: [0, 0, 0], scores: [1, 2, 3] })).toBe(
      "Le Juge n'a rien à retourner. Personne ne marque.",
    );
  });
});
