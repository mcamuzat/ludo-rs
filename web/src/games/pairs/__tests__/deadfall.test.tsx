import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DeadfallBoard, describeDeadfallEvent } from "../deadfall/Board";
import type { DeadfallAction, DeadfallView } from "../deadfall/types";

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

const PLAY: DeadfallView = {
  handNo: 2,
  handsTotal: 5,
  phase: "Play",
  seats: [
    { stack: [4, 7], handSize: 3, chips: -6 },
    { stack: [5], handSize: 5, chips: 9 },
    { stack: [5, 2, 5], handSize: 3, chips: -3 },
  ],
  hand: [3, 3, 8],
  placed: null,
  turn: 0,
  pot: 3,
  ante: 1,
  events: [{ type: "Played", seat: 2, card: 5 }],
};

const PLAY_LEGAL: DeadfallAction[] = [{ Play: { card: 3 } }, { Play: { card: 8 } }, { Call: { seat: 2 } }];

const DOOR: DeadfallView = {
  ...PLAY,
  phase: "Door",
  seats: NAMES.map(() => ({ stack: [], handSize: 6, chips: -1 })),
  hand: [2, 6, 6, 7, 9, 10],
  turn: null,
  events: [],
};

function board(view: DeadfallView, legal: DeadfallAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <DeadfallBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: DeadfallView, legal: DeadfallAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Deadfall board", () => {
  it("submits exactly the legal actions in play, one per control", () => {
    const sent = clickEverything(PLAY, PLAY_LEGAL);
    expect(sent).toHaveLength(PLAY_LEGAL.length);
    for (const action of PLAY_LEGAL) expect(sent).toContainEqual(action);
  });

  it("submits exactly the legal door cards", () => {
    const legal: DeadfallAction[] = [2, 6, 7, 9, 10].map((card) => ({ Door: { card } }));
    const sent = clickEverything(DOOR, legal);
    expect(sent).toHaveLength(5);
    for (const action of legal) expect(sent).toContainEqual(action);
  });

  it("shows the hand, the stacks, the pot and the balances", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Main 2 / 5");
    expect(html).toContain("Pot : 3");
    expect(html).toContain("−6");
    expect(html).toContain("+9");
    expect(html).toContain("Jouer ce 3");
    expect(html).toContain("Appeler ce 5 : mort !");
    // Only the latest of the two 5s on top may be called.
    expect(html.match(/Appeler/g)).toHaveLength(1);
    expect(html).not.toMatch(/\d\s?%</);
    const door = renderToStaticMarkup(board(DOOR, [{ Door: { card: 2 } }], 0));
    expect(door).toContain("Choisir ce 2 comme porte");
    const placed = renderToStaticMarkup(board({ ...DOOR, placed: 6 }, [], 0));
    expect(placed).not.toContain("<button");
    expect(placed).toContain("Ta porte : 6");
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 1 }, [], 0))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeDeadfallEvent>[0]) => describeDeadfallEvent(e, NAMES);
    expect(say({ type: "DoorsRevealed", cards: [5, 2, 9], leader: 1 })).toBe(
      "Portes : Ana 5, Bob 2, Cléo 9. Bob commence.",
    );
    expect(say({ type: "DoorsRevealed", cards: [3, 3, 6], leader: null })).toBe(
      "Portes : Ana 3, Bob 3, Cléo 6. Égalité : chacun joue une autre porte.",
    );
    expect(say({ type: "Played", seat: 0, card: 7 })).toBe("Ana joue un 7.");
    expect(
      say({
        type: "Showdown",
        caller: 0,
        target: 2,
        rank: 5,
        dead: true,
        hands: [[3], [], [8, 8]],
        winner: 0,
        paid: 5,
        pot: 3,
      }),
    ).toBe(
      "Ana appelle le 5 de Cléo. Mains : Ana 3, Bob rien, Cléo 8-8. " +
        "Verdict : les 5 sont morts, appel réussi. Cléo paie 5 jetons à Ana, qui prend le pot (3 jetons).",
    );
    expect(
      say({
        type: "Showdown",
        caller: 0,
        target: 1,
        rank: 1,
        dead: false,
        hands: [[1], [2], [3]],
        winner: 1,
        paid: 1,
        pot: 3,
      }),
    ).toContain("il reste des 1, appel raté. Ana paie 1 jeton à Bob");
  });
});
