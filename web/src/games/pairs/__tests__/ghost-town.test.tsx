import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { GhostTownBoard, describeGhostTownEvent } from "../ghost-town/Board";
import type { GhostTownAction, GhostTownView } from "../ghost-town/types";

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

const PLAY: GhostTownView = {
  handNo: 2,
  handsTotal: 5,
  phase: "Play",
  seats: [
    { house: [6], handSize: 3, out: false, chips: -14 },
    { house: [9, 4], handSize: 2, out: false, chips: 7 },
    { house: [], handSize: 0, out: true, chips: -21 },
  ],
  street: [2, 5],
  hand: [6, 7, 7],
  placed: null,
  neighbour: 1,
  turn: 0,
  pot: 28,
  ante: 10,
  playCost: 2,
  deck: 30,
  discard: 6,
  discardSeen: [3],
  events: [{ type: "Folded", seat: 2 }],
};

const PLAY_LEGAL: GhostTownAction[] = [
  "Draw",
  { Play: { card: 6, to: 1 } },
  { Play: { card: 7, to: 0 } },
  "Fold",
];

const SETUP: GhostTownView = {
  ...PLAY,
  handNo: 1,
  phase: "Setup",
  seats: NAMES.map(() => ({ house: [], handSize: 5, out: false, chips: -10 })),
  street: [],
  hand: [3, 8, 8, 9, 10],
  neighbour: null,
  turn: null,
  pot: 30,
  events: [],
};

function board(view: GhostTownView, legal: GhostTownAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <GhostTownBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: GhostTownView, legal: GhostTownAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Ghost Town board", () => {
  it("submits exactly the legal actions during play, one per control", () => {
    const sent = clickEverything(PLAY, PLAY_LEGAL);
    expect(sent).toHaveLength(PLAY_LEGAL.length);
    for (const action of PLAY_LEGAL) expect(sent).toContainEqual(action);
  });

  it("submits exactly the legal placements during setup", () => {
    const legal: GhostTownAction[] = [3, 8, 9, 10].map((card) => ({ Place: { card } }));
    const sent = clickEverything(SETUP, legal);
    expect(sent).toHaveLength(4);
    for (const action of legal) expect(sent).toContainEqual(action);
  });

  it("shows the pot, the street, the balances and where a card would go", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Main 2 / 5");
    expect(html).toContain("Pot : 28");
    expect(html).toContain("−14");
    expect(html).toContain("+7");
    expect(html).toContain("Poser ce 6 chez Bob (2 jetons)");
    expect(html).toContain("Poser ce 7 chez toi (2 jetons)");
    expect(html).toContain("Piocher");
    expect(html).toContain("Se coucher");
    expect(html).toContain("hors de la main");
    // No statistics: weighing a draw is the game.
    expect(html).not.toMatch(/\d\s?%</);
    expect(html.match(/class="pairs-seat/g)).toHaveLength(3);
  });

  it("offers nothing once my card is placed, out of the hand, or after the game", () => {
    const placed = renderToStaticMarkup(board({ ...SETUP, placed: 8 }, [], 0));
    expect(placed).not.toContain("<button");
    expect(placed).toContain("Tu as posé un 8");
    const out = renderToStaticMarkup(board({ ...PLAY, turn: 1, hand: [] }, [], 2));
    expect(out).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null, pot: 0 }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeGhostTownEvent>[0]) => describeGhostTownEvent(e, NAMES);
    expect(say({ type: "Placed", seat: 1, card: 4 })).toBe("Bob pose un 4 dans la rue.");
    expect(say({ type: "TieBreak", seat: 1, card: 4 })).toBe("Départage : Bob tire un 4.");
    expect(say({ type: "Starts", seat: 1 })).toBe("Bob commence.");
    expect(say({ type: "Played", seat: 0, to: 0, card: 7 })).toBe("Ana pose un 7 dans sa maison.");
    expect(say({ type: "Played", seat: 0, to: 1, card: 6 })).toBe("Ana pose un 6 chez Bob.");
    expect(say({ type: "Drew", seat: 0, card: 9, won: 9 })).toBe("Ana pioche un 9 et gagne 9 jetons.");
    expect(say({ type: "Drew", seat: 0, card: 9, won: 4 })).toBe("Ana pioche un 9 et gagne 4 jetons : le pot est vide.");
    expect(say({ type: "Busted", seat: 0, card: 6 })).toBe("Ana pioche un 6 : paire ! 6 jetons au pot, et dehors.");
    expect(say({ type: "Folded", seat: 2 })).toBe("Cléo se couche.");
    expect(say({ type: "HandEnded", winner: 1, pot: 30 })).toBe("Bob reste seul et prend le pot : 30 jetons.");
    expect(say({ type: "HandEnded", winner: null, pot: 0 })).toBe("Le pot est vide : la main est finie.");
    expect(say({ type: "Reshuffled" })).toBe("La défausse est remélangée.");
  });
});
