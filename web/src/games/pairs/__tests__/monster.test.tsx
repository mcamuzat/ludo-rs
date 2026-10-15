import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DiscardPicker, MonsterBoard, describeMonsterEvent, matchingDiscard } from "../monster/Board";
import type { MonsterAction, MonsterView } from "../monster/types";

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

const VIEW: MonsterView = {
  gameNo: 1,
  gamesTotal: 3,
  roundNo: 2,
  phase: "Act",
  seats: [
    { inGame: true, inRound: true, handSize: 6, acted: false, folded: false, discards: 0, chips: -2 },
    { inGame: true, inRound: true, handSize: 4, acted: true, folded: false, discards: 2, chips: -2 },
    { inGame: false, inRound: false, handSize: 0, acted: false, folded: false, discards: 0, chips: -7 },
    { inGame: true, inRound: true, handSize: 0, acted: true, folded: true, discards: 0, chips: -2 },
  ],
  hand: [9, 9, 4, 6, 7, 10],
  discarded: [],
  monsterSize: 2,
  button: 3,
  turn: 0,
  pot: 13,
  events: [{ type: "Discarded", seat: 1, count: 2 }],
};

const LEGAL: MonsterAction[] = [
  { Discard: { cards: [9] } },
  { Discard: { cards: [4, 9] } },
  "Fold",
];

function picker(picked: number[], legal: MonsterAction[], sent: unknown[], toggled: number[] = []) {
  return (
    <DiscardPicker
      hand={VIEW.hand}
      legal={legal}
      picked={picked}
      onToggle={(k) => toggled.push(k)}
      submit={(a) => sent.push(a)}
    />
  );
}

function click(node: ReactNode) {
  hostElements(node)
    .filter((el) => typeof el.props.onClick === "function" && !el.props.disabled)
    .forEach((el) => (el.props.onClick as () => void)());
}

describe("Monster board", () => {
  it("matches the picked cards to a legal discard", () => {
    expect(matchingDiscard(LEGAL, VIEW.hand, [1])).toEqual({ Discard: { cards: [9] } });
    expect(matchingDiscard(LEGAL, VIEW.hand, [2, 0])).toEqual({ Discard: { cards: [4, 9] } });
    expect(matchingDiscard(LEGAL, VIEW.hand, [5])).toBeUndefined();
  });

  it("submits the picked discard or the fold, and toggles cards", () => {
    const sent: unknown[] = [];
    const toggled: number[] = [];
    click(picker([0], LEGAL, sent, toggled));
    expect(sent).toEqual([{ Discard: { cards: [9] } }, "Fold"]);
    expect(toggled).toEqual([0, 1, 2, 3, 4, 5]);
    const none: unknown[] = [];
    click(picker([5], LEGAL, none));
    expect(none).toEqual(["Fold"]);
  });

  it("shows the table, the Monster and the statuses", () => {
    const html = renderToStaticMarkup(
      <MonsterBoard view={VIEW} legalActions={LEGAL} pending={[0]} me={0} seatNames={NAMES} submit={() => {}} />,
    );
    expect(html).toContain("Partie 1 / 3 · Manche 2 · Pot : 13");
    expect(html).toContain("Monstre");
    expect(html).toContain("2 cartes au Monstre");
    expect(html).toContain("Hors jeu");
    expect(html).toContain("Couché");
    const watching = renderToStaticMarkup(
      <MonsterBoard view={VIEW} legalActions={[]} pending={[0]} me={null} seatNames={NAMES} submit={() => {}} />,
    );
    expect(watching).not.toContain("<button");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeMonsterEvent>[0]) => describeMonsterEvent(e, NAMES);
    expect(
      say({ type: "Showdown", hands: [[10, 9, 8], null, null, [7, 6]], monster: [10, 9, 2], champions: [0], won: true }),
    ).toBe("Abattage. Ana 10-9-8, Malo 7-6. Monstre : 10-9-2. Ana bat le Monstre.");
    expect(say({ type: "Penalty", seats: [1] })).toBe("Hugo paie 5 jetons et sort.");
    expect(say({ type: "Discarded", seat: 2, count: 0 })).toBe("Cléo garde sa main.");
  });
});
