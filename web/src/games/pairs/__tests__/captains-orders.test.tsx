import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { CaptainsOrdersBoard, describeCaptainsOrdersEvent, describeRound } from "../captains-orders/Board";
import type { CaptainsOrdersAction, CaptainsOrdersView } from "../captains-orders/types";

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

const NAMES = ["Ana", "Bob", "Cléo", "Dan"];

const TURN: CaptainsOrdersView = {
  round: 2,
  winsNeeded: 3,
  dealer: 3,
  phase: { type: "Turn" },
  seats: [
    { handSize: 2, wins: 1 },
    { handSize: 6, wins: 0 },
    { handSize: 5, wins: 2 },
    { handSize: 6, wins: 0 },
  ],
  hand: [3, 9],
  discarded: [7],
  order: null,
  turn: 0,
  log: [
    { type: "Passed", from: 1, to: 2, card: null },
    { type: "Discarded", seat: 2, card: null },
  ],
  lastRound: null,
};

const TURN_LEGAL: CaptainsOrdersAction[] = [
  { Order: { card: 3 } },
  { Order: { card: 9 } },
  ...[1, 2, 3].flatMap((to) => [{ Pass: { card: 3, to } }, { Pass: { card: 9, to } }]),
  ...[1, 2, 3].map((from) => ({ Ask: { from } })),
  ...[1, 2, 3].map((seat) => ({ Accuse: { seat } })),
];

const RESPOND: CaptainsOrdersView = {
  ...TURN,
  phase: { type: "Respond" },
  order: { card: 5, owner: 3 },
  hand: [3, 9, 9],
};

const RESPOND_LEGAL: CaptainsOrdersAction[] = [
  { Order: { card: 3 } },
  { Follow: { card: 9 } },
  ...[1, 2, 3].map((seat) => ({ Accuse: { seat } })),
];

function board(
  view: CaptainsOrdersView,
  legal: CaptainsOrdersAction[],
  me: number | null,
  submit: (a: unknown) => void = () => {},
) {
  return (
    <CaptainsOrdersBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: CaptainsOrdersView, legal: CaptainsOrdersAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Captain's Orders board", () => {
  it("submits exactly the legal actions, one per control", () => {
    for (const [view, legal] of [
      [TURN, TURN_LEGAL],
      [RESPOND, RESPOND_LEGAL],
      [{ ...TURN, phase: { type: "Give", asker: 2 } }, [{ Give: { card: 3 } }, { Give: { card: 9 } }]],
      [{ ...TURN, phase: { type: "Discard", after: 1 } }, [{ Discard: { card: 3 } }, { Discard: { card: 9 } }]],
    ] as [CaptainsOrdersView, CaptainsOrdersAction[]][]) {
      const sent = clickEverything(view, legal);
      expect(sent).toHaveLength(legal.length);
      for (const action of legal) expect(sent).toContainEqual(action);
    }
  });

  it("shows the hand, the Order, the wins and my discards", () => {
    const html = renderToStaticMarkup(board(RESPOND, RESPOND_LEGAL, 0));
    expect(html).toContain("Manche 2");
    expect(html).toContain("Ordre en cours : 5 (Dan)");
    expect(html).toContain("2 manches");
    expect(html).toContain("Tes défausses : 7");
    expect(html).toContain("Obéir avec ce 9");
    expect(html).toContain("Annuler l&#x27;ordre avec ce 3");
    expect(html).toContain("Bob passe une carte à Cléo.");
    expect(html).not.toMatch(/\d\s?%</);
    const turn = renderToStaticMarkup(board(TURN, TURN_LEGAL, 0));
    expect(turn).toContain("Ordonner ce 3");
    expect(turn).toContain("Passer ce 9 à Bob");
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...TURN, turn: 1 }, [], 0))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...TURN, phase: { type: "Over" }, turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
    expect(renderToStaticMarkup(board(TURN, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeCaptainsOrdersEvent>[0]) => describeCaptainsOrdersEvent(e, NAMES);
    expect(say({ type: "Ordered", seat: 0, card: 7, belay: false })).toBe("Ana donne l'ordre 7.");
    expect(say({ type: "Ordered", seat: 1, card: 2, belay: true })).toBe("Bob annule l'ordre avec un 2.");
    expect(say({ type: "Followed", seat: 2, owner: 1, card: 8 })).toBe("Cléo obéit et passe un 8 à Bob.");
    expect(say({ type: "Gave", from: 3, to: 0, card: null })).toBe("Dan donne une carte à Ana.");
    expect(say({ type: "OrderDone", owner: 1, card: 2 })).toBe("L'ordre 2 de Bob a fait le tour : défaussé.");
    expect(
      describeRound(
        {
          round: 1,
          ending: { type: "Accused", accuser: 0, target: 3, right: false },
          hands: [[5, 9], [2, 6], [7, 8], [3, 6]],
          captains: [1],
          points: [14, 8, 15, 9],
          winners: [2],
        },
        NAMES,
      ),
    ).toBe(
      "Manche 1 — Ana accuse Dan : raté. Mains : Ana 5-9 (14), Bob 2-6 (8), Cléo 7-8 (15), Dan 3-6 (9). " +
        "Capitaine : Bob. Cléo gagne la manche.",
    );
    expect(
      describeRound(
        {
          round: 3,
          ending: { type: "Emptied", seat: 1, card: 2, right: true },
          hands: [[5], [], [7], [3]],
          captains: [1],
          points: [5, 0, 7, 3],
          winners: [1],
        },
        NAMES,
      ),
    ).toContain("Bob vide sa main avec un 2 et se déclare Capitaine : à raison.");
  });
});
