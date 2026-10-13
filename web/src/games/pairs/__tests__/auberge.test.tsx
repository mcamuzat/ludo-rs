import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { AubergeBoard, describeAubergeEvent } from "../auberge/Board";
import type { AubergeAction, AubergeView } from "../auberge/types";

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

const PLAY: AubergeView = {
  round: 2,
  rounds: 5,
  phase: "Play",
  seats: [
    { front: [5], handSize: 2, out: false, wins: 1 },
    { front: [2, 9], handSize: 3, out: false, wins: 0 },
    { front: [], handSize: 0, out: true, wins: 0 },
  ],
  hand: [7, 7],
  placed: null,
  ownPair: 7,
  turn: 0,
  deck: 30,
  discard: 4,
  discardSeen: [4, 8],
  events: [{ type: "Out", seat: 2 }],
};

const PLAY_LEGAL: AubergeAction[] = [
  "Draw",
  { Dare: { target: 1 } },
  { Attack: { target: 1, card: 7 } },
  { Accuse: { target: 1 } },
];

const SETUP: AubergeView = {
  ...PLAY,
  round: 1,
  phase: "Setup",
  seats: NAMES.map(() => ({ front: [], handSize: 3, out: false, wins: 0 })),
  hand: [3, 8, 8],
  ownPair: 8,
  turn: null,
  events: [],
};

function board(view: AubergeView, legal: AubergeAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <AubergeBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: AubergeView, legal: AubergeAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Auberge des Pirates board", () => {
  it("submits exactly the legal actions during play, one per control", () => {
    const sent = clickEverything(PLAY, PLAY_LEGAL);
    expect(sent).toHaveLength(PLAY_LEGAL.length);
    for (const action of PLAY_LEGAL) expect(sent).toContainEqual(action);
  });

  it("submits exactly the legal placements during setup", () => {
    const legal: AubergeAction[] = [{ Place: { card: 3 } }, { Place: { card: 8 } }];
    const sent = clickEverything(SETUP, legal);
    expect(sent).toHaveLength(2);
    for (const action of legal) expect(sent).toContainEqual(action);
  });

  it("shows the table, the hand and the warning about a held pair", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Manche 2 / 5");
    expect(html).toContain("Tu as une paire de 7");
    // No statistics on the board: who holds what is for the players to work out.
    expect(html).not.toContain("Paire estimée");
    expect(html).not.toContain("pairs-unlocated");
    expect(html).toContain("Piocher");
    expect(html).toContain("Accuser");
    expect(html).toContain("éliminé");
    expect(html.match(/class="pairs-seat/g)).toHaveLength(3);
  });

  it("offers nothing once my card is placed", () => {
    const placed: AubergeView = { ...SETUP, placed: 8 };
    const html = renderToStaticMarkup(board(placed, [], 0));
    expect(html).not.toContain("<button");
    expect(html).toContain("En attente des autres joueurs");
    expect(html).toContain("Tu as posé un 8");
  });

  it("offers nothing to a spectator or once the game is over", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, hand: [], ownPair: null }, [], null))).not.toContain("<button");
    const over: AubergeView = { ...PLAY, phase: "Over", turn: null, events: [{ type: "RoundWon", seat: 0 }] };
    const html = renderToStaticMarkup(board(over, [], 0));
    expect(html).not.toContain("<button");
    expect(html).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeAubergeEvent>[0]) => describeAubergeEvent(e, NAMES);
    expect(say({ type: "Placed", seat: 1, card: 4 })).toBe("Bob pose un 4.");
    expect(say({ type: "Starts", seat: 1 })).toBe("Bob commence.");
    expect(say({ type: "Drew", seat: 0 })).toBe("Ana pioche.");
    expect(say({ type: "Attacked", seat: 0, target: 1, card: 7 })).toBe("Ana attaque Bob avec un 7.");
    expect(say({ type: "Dared", seat: 0, target: 1, card: 6, paired: false })).toBe("Ana défie Bob : un 6, pas de paire.");
    expect(say({ type: "Dared", seat: 0, target: 1, card: 9, paired: true })).toBe(
      "Ana défie Bob : un 9, qui fait une paire.",
    );
    expect(say({ type: "Accused", seat: 0, target: 1, guilty: true, hand: [] })).toBe(
      "Ana accuse Bob, à raison.",
    );
    expect(say({ type: "Accused", seat: 0, target: 1, guilty: false, hand: [3, 8] })).toBe(
      "Ana accuse Bob, à tort. Main montrée : 3, 8.",
    );
    expect(say({ type: "Out", seat: 2 })).toBe("Cléo est éliminé·e.");
    expect(say({ type: "RoundWon", seat: 0 })).toBe("Ana gagne la manche.");
    expect(say({ type: "RoundWon", seat: null })).toBe("Manche bloquée : personne ne la gagne.");
    expect(say({ type: "Reshuffled" })).toBe("La défausse est remélangée.");
  });
});
