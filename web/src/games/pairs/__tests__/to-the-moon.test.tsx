import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ToTheMoonBoard, describeToTheMoonEvent } from "../to-the-moon/Board";
import type { ToTheMoonAction, ToTheMoonView } from "../to-the-moon/types";

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

const NAMES = ["Ana", "Bob", "Cléo"];

const PLAY: ToTheMoonView = {
  roundNo: 2,
  roundsTotal: 6,
  phase: "Play",
  dealer: 1,
  leader: 2,
  seats: [
    { handSize: 4, known: [], out: false, score: 12 },
    { handSize: 6, known: [8], out: true, score: 7 },
    { handSize: 3, known: [], out: false, score: 0 },
  ],
  hand: [3, 3, 5, 9],
  trick: [4, 9, 6],
  turn: 0,
  deck: 21,
  events: [{ type: "Withdrew", seat: 1, took: 8 }],
};

const PLAY_LEGAL: ToTheMoonAction[] = [
  { Build: { card: 3 } },
  { Build: { card: 5 } },
  "Test",
  { Withdraw: { take: 4 } },
  { Withdraw: { take: 6 } },
  { Withdraw: { take: 9 } },
];

function board(view: ToTheMoonView, legal: ToTheMoonAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <ToTheMoonBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: ToTheMoonView, legal: ToTheMoonAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("To the Moon board", () => {
  it("submits exactly the legal actions, one per control", () => {
    const sent = clickEverything(PLAY, PLAY_LEGAL);
    expect(sent).toHaveLength(PLAY_LEGAL.length);
    for (const action of PLAY_LEGAL) expect(sent).toContainEqual(action);
  });

  it("offers an empty-handed withdrawal when one card is on the table", () => {
    const legal: ToTheMoonAction[] = [{ Withdraw: { take: null } }];
    const sent = clickEverything({ ...PLAY, trick: [4], hand: [4] }, legal);
    expect(sent).toEqual(legal);
  });

  it("shows the round, the countdown, the scores and the known cards", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Manche 2 / 6");
    expect(html).toContain("Bob (donneur)");
    expect(html).toContain("Poser ce 3");
    expect(html).toContain("Poser ce 5");
    expect(html).not.toContain("Poser ce 9");
    expect(html).toContain("Me retirer en prenant ce 6");
    expect(html).toContain("Tester la pioche");
    expect(html).toContain("hors du pli");
    expect(html).toContain(">12<");
    // No statistics.
    expect(html).not.toMatch(/\d\s?%</);
    expect(html.match(/class="pairs-seat/g)).toHaveLength(3);
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 2 }, [], 0))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeToTheMoonEvent>[0]) => describeToTheMoonEvent(e, NAMES);
    expect(say({ type: "Dealt", round: 3, dealer: 2 })).toBe("Manche 3 : Cléo distribue.");
    expect(say({ type: "Led", seat: 0, card: 7 })).toBe("Ana lance un compte à rebours avec un 7.");
    expect(say({ type: "Built", seat: 1, card: 2 })).toBe("Bob pose un 2.");
    expect(say({ type: "Tested", seat: 1, card: 2, ok: true })).toBe("Bob teste : un 2, ça passe.");
    expect(say({ type: "Tested", seat: 1, card: 2, ok: false })).toBe(
      "Bob teste : un 2, déjà là ! Dehors, le 2 en main.",
    );
    expect(say({ type: "Withdrew", seat: 2, took: null })).toBe("Cléo se retire.");
    expect(say({ type: "Withdrew", seat: 2, took: 6 })).toBe("Cléo se retire et prend le 6.");
    expect(say({ type: "TrickWon", seat: 0, cards: 1, perfect: false })).toBe(
      "Ana gagne le pli : 1 point.",
    );
    expect(say({ type: "TrickWon", seat: 0, cards: 10, perfect: true })).toBe(
      "Ana complète le compte à rebours : décollage ! 10 points.",
    );
    expect(say({ type: "RoundEnded", round: 4 })).toBe("La pioche est vide : fin de la manche 4.");
  });
});
