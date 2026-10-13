import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ZombieLifeBoard, describeZombieLifeEvent } from "../zombie-life/Board";
import type { ZombieLifeAction, ZombieLifeView } from "../zombie-life/types";

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

const PLAY: ZombieLifeView = {
  round: 2,
  target: 21,
  phase: "Play",
  seats: [
    { faceUp: [6, 3], faceDown: 0, status: "In", score: 4 },
    { faceUp: [9], faceDown: 2, status: "Passed", score: 12 },
    { faceUp: [], faceDown: 0, status: "Out", score: 7 },
  ],
  horde: [3, 8, 5],
  turn: 0,
  drawn: null,
  dealing: [],
  dealTo: null,
  deck: 30,
  discard: 6,
  events: [{ type: "Overrun", seat: 2, card: 4 }],
};

const PLAY_LEGAL: ZombieLifeAction[] = ["Draw", { Take: { card: 5 } }, { Take: { card: 8 } }, "Pass"];

const PLACING: ZombieLifeView = { ...PLAY, phase: "Placing", drawn: 7, events: [{ type: "Drew", seat: 0, card: 7 }] };
const PLACING_LEGAL: ZombieLifeAction[] = [{ Place: { zone: "Refuge" } }, { Place: { zone: "Horde" } }];

const DEALING: ZombieLifeView = {
  ...PLAY,
  phase: "Dealing",
  seats: [
    { faceUp: [], faceDown: 0, status: "In", score: 4 },
    { faceUp: [2], faceDown: 0, status: "In", score: 12 },
    { faceUp: [], faceDown: 0, status: "In", score: 7 },
  ],
  horde: [],
  dealing: [4, 4, 9],
  dealTo: 2,
  events: [{ type: "Dealer", seat: 0 }],
};
const DEALING_LEGAL: ZombieLifeAction[] = [{ Give: { card: 4 } }, { Give: { card: 9 } }];

function board(view: ZombieLifeView, legal: ZombieLifeAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <ZombieLifeBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: ZombieLifeView, legal: ZombieLifeAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

function expectExactly(view: ZombieLifeView, legal: ZombieLifeAction[]) {
  const sent = clickEverything(view, legal);
  expect(sent).toHaveLength(legal.length);
  for (const action of legal) expect(sent).toContainEqual(action);
}

describe("Zombie Life board", () => {
  it("submits exactly the legal actions on my turn, one per control", () => {
    expectExactly(PLAY, PLAY_LEGAL);
  });

  it("submits exactly the legal placements of a drawn card", () => {
    expectExactly(PLACING, PLACING_LEGAL);
    expectExactly(PLACING, [{ Place: { zone: "Horde" } }]);
  });

  it("submits one gift per rank the dealer holds", () => {
    expectExactly(DEALING, DEALING_LEGAL);
  });

  it("shows the round, the scores, the horde and the hidden cards", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Manche 2");
    expect(html).toContain("4 / 21");
    expect(html).toContain("12 / 21");
    expect(html).toContain("Horde");
    expect(html).toContain("Prendre ce 8");
    expect(html).not.toContain("Prendre ce 3");
    expect(html).toContain("cachées");
    expect(html).toContain("caché");
    expect(html).toContain("submergé");
    expect(html).toContain("Piocher");
    expect(html).toContain("Passer");
    // No statistics: weighing a draw is the game.
    expect(html).not.toMatch(/\d\s?%</);
    expect(html.match(/class="pairs-seat/g)).toHaveLength(3);
  });

  it("shows the drawn card and where it may go", () => {
    const mine = renderToStaticMarkup(board(PLACING, PLACING_LEGAL, 0));
    expect(mine).toContain("Ana a pioché");
    expect(mine).toContain("Dans mon Refuge");
    expect(mine).toContain("Dans la Horde");
    expect(mine).not.toContain("Piocher");
    const theirs = renderToStaticMarkup(board(PLACING, [], 1));
    expect(theirs).toContain("Ana a pioché");
    expect(theirs).not.toContain("<button");
  });

  it("shows the dealer's cards and who gets the next one", () => {
    const mine = renderToStaticMarkup(board(DEALING, DEALING_LEGAL, 0));
    expect(mine).toContain("Donner ce 4 à Cléo");
    expect(mine).toContain("Donner ce 9 à Cléo");
    expect(mine).toContain("Choisis la carte de Cléo");
    const theirs = renderToStaticMarkup(board(DEALING, [], 1));
    expect(theirs).toContain("Ana distribue");
    expect(theirs).not.toContain("<button");
  });

  it("offers nothing out of turn, hidden, overrun, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 0 }, [], 1))).not.toContain("<button");
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 0 }, [], 2))).not.toContain("<button");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeZombieLifeEvent>[0]) => describeZombieLifeEvent(e, NAMES);
    expect(say({ type: "Dealt", seat: 1, card: 4 })).toBe("Bob reçoit un 4.");
    expect(say({ type: "Starts", seat: 1 })).toBe("Bob commence.");
    expect(say({ type: "Drew", seat: 0, card: 9 })).toBe("Ana pioche un 9.");
    expect(say({ type: "Placed", seat: 0, card: 9, zone: "Refuge" })).toBe("Ana pose le 9 dans son Refuge.");
    expect(say({ type: "Placed", seat: 0, card: 9, zone: "Horde" })).toBe("Ana pose le 9 dans la Horde.");
    expect(say({ type: "Took", seat: 2, card: 5 })).toBe("Cléo prend le 5 de la Horde.");
    expect(say({ type: "Passed", seat: 2, hidden: 0 })).toBe("Cléo se cache.");
    expect(say({ type: "Passed", seat: 2, hidden: 1 })).toBe("Cléo se cache et retourne 1 carte.");
    expect(say({ type: "Passed", seat: 2, hidden: 3 })).toBe("Cléo se cache et retourne 3 cartes.");
    expect(say({ type: "Overrun", seat: 0, card: 6 })).toBe("Ana pioche un 6 : submergé par les zombies !");
    expect(say({ type: "Scored", seat: 1, points: 1 })).toBe("Bob marque 1 point.");
    expect(say({ type: "Scored", seat: 1, points: 4 })).toBe("Bob marque 4 points.");
    expect(say({ type: "Scored", seat: 1, points: 0 })).toBe("Bob ne marque rien.");
    expect(say({ type: "Dealer", seat: 1 })).toBe("Bob distribue la nouvelle manche.");
    expect(say({ type: "Reshuffled" })).toBe("La défausse est remélangée.");
  });
});
