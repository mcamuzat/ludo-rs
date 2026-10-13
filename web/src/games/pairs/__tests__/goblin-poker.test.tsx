import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { GoblinPokerBoard, describeGoblinPokerEvent } from "../goblin-poker/Board";
import type { GoblinPokerAction, GoblinPokerView } from "../goblin-poker/types";

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

const PLAY: GoblinPokerView = {
  handNo: 2,
  handsTotal: 5,
  round: 3,
  phase: "Play",
  seats: [
    { stack: [6, 2], handSize: 3, out: false, pitched: false, chips: -14 },
    { stack: [9, 4], handSize: 2, out: false, pitched: true, chips: 7 },
    { stack: [], handSize: 0, out: true, pitched: false, chips: -21 },
  ],
  hand: [6, 7, 7],
  placed: null,
  pitched: null,
  turn: 0,
  pot: 28,
  ante: 1,
  foldCost: 2,
  deck: 30,
  discard: 6,
  discardSeen: [3],
  events: [{ type: "Pitched", seat: 1 }],
};

const PLAY_LEGAL: GoblinPokerAction[] = [{ Pitch: { card: 6 } }, { Pitch: { card: 7 } }, "Fold"];

const SETUP: GoblinPokerView = {
  ...PLAY,
  handNo: 1,
  round: 0,
  phase: "Setup",
  seats: NAMES.map(() => ({ stack: [], handSize: 6, out: false, pitched: false, chips: -1 })),
  hand: [3, 8, 8, 9, 10, 10],
  turn: null,
  pot: 3,
  foldCost: null,
  events: [],
};

function board(view: GoblinPokerView, legal: GoblinPokerAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <GoblinPokerBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: GoblinPokerView, legal: GoblinPokerAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Goblin Poker board", () => {
  it("submits exactly the legal actions during play, one per control", () => {
    const sent = clickEverything(PLAY, PLAY_LEGAL);
    expect(sent).toHaveLength(PLAY_LEGAL.length);
    for (const action of PLAY_LEGAL) expect(sent).toContainEqual(action);
  });

  it("submits exactly the legal first cards during setup", () => {
    const legal: GoblinPokerAction[] = [3, 8, 9, 10].map((card) => ({ Place: { card } }));
    const sent = clickEverything(SETUP, legal);
    expect(sent).toHaveLength(4);
    for (const action of legal) expect(sent).toContainEqual(action);
  });

  it("shows the hand, the round, the pot, the balances and what folding costs", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Main 2 / 5");
    expect(html).toContain("Tour 3");
    expect(html).toContain("Pot : 28");
    expect(html).toContain("−14");
    expect(html).toContain("+7");
    expect(html).toContain("Jeter ce 6");
    expect(html).toContain("Jeter ce 7");
    expect(html).toContain("Se coucher (2 jetons)");
    expect(html).toContain("a jeté");
    expect(html).toContain("hors de la main");
    // No statistics: guessing what comes back is the game.
    expect(html).not.toMatch(/\d\s?%</);
    expect(html.match(/class="pairs-seat/g)).toHaveLength(3);
    const one = renderToStaticMarkup(board({ ...PLAY, foldCost: 1 }, PLAY_LEGAL, 0));
    expect(one).toContain("Se coucher (1 jeton)");
  });

  it("tells me what I pitched and what I placed", () => {
    const pitched = renderToStaticMarkup(board({ ...PLAY, turn: 1, pitched: 7, hand: [6, 7] }, [], 0));
    expect(pitched).toContain("Tu as jeté un 7");
    expect(pitched).not.toContain("<button");
    const setup = renderToStaticMarkup(board(SETUP, [{ Place: { card: 3 } }], 0));
    expect(setup).toContain("Commencer ma pile avec ce 3");
    expect(setup).toContain("Choisis la première carte de ta pile");
    const placed = renderToStaticMarkup(board({ ...SETUP, placed: 8 }, [], 0));
    expect(placed).not.toContain("<button");
    expect(placed).toContain("Tu as posé un 8");
  });

  it("offers nothing out of the hand, watching, or after the game", () => {
    const out = renderToStaticMarkup(board({ ...PLAY, turn: 1, hand: [] }, [], 2));
    expect(out).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null, pot: 0 }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeGoblinPokerEvent>[0]) => describeGoblinPokerEvent(e, NAMES);
    expect(say({ type: "Placed", seat: 1, card: 4 })).toBe("Bob commence sa pile avec un 4.");
    expect(say({ type: "TieBreak", seat: 1, card: 4, kept: true })).toBe("Départage : Bob ajoute un 4 à sa pile.");
    expect(say({ type: "TieBreak", seat: 1, card: 4, kept: false })).toBe(
      "Départage : Bob tire un 4, une paire : défaussé.",
    );
    expect(say({ type: "Starts", seat: 1 })).toBe("Bob a la pile la plus basse et commence.");
    expect(say({ type: "Pitched", seat: 0 })).toBe("Ana jette une carte.");
    expect(say({ type: "Folded", seat: 2, cost: 3 })).toBe("Cléo se couche et paie 3 jetons.");
    expect(say({ type: "Folded", seat: 2, cost: 1 })).toBe("Cléo se couche et paie 1 jeton.");
    expect(say({ type: "Caught", seat: 0, card: 9, paired: false })).toBe("Ana reçoit un 9.");
    expect(say({ type: "Caught", seat: 0, card: 6, paired: true })).toBe(
      "Ana reçoit un 6 : paire ! 6 jetons au pot, et dehors.",
    );
    expect(say({ type: "HandEnded", winners: [1], share: 30 })).toBe("Bob reste seul et prend le pot : 30 jetons.");
    expect(say({ type: "HandEnded", winners: [0, 2], share: 4 })).toBe(
      "Ana et Cléo se partagent le pot : 4 jetons chacun.",
    );
    expect(say({ type: "HandEnded", winners: [0, 1, 2], share: 1 })).toBe(
      "Ana, Bob et Cléo se partagent le pot : 1 jeton chacun.",
    );
  });
});
