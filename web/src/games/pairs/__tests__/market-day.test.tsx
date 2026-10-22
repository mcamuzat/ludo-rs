import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { MarketDayBoard, describeMarketDayEvent } from "../market-day/Board";
import type { MarketDayAction, MarketDayView } from "../market-day/types";

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

const NAMES = ["Ana", "Hugo", "Cléo", "Léa"];

const PLAY: MarketDayView = {
  handNo: 2,
  hands: 6,
  phase: "Play",
  dealer: 1,
  leader: 2,
  turn: 0,
  trick: [
    [2, 8],
    [3, 7],
  ],
  discard: [4, 5],
  hand: [1, 3, 3, 9],
  seats: [
    { handSize: 4, score: 12 },
    { handSize: 3, score: 3 },
    { handSize: 3, score: 9 },
    { handSize: 3, score: 0 },
  ],
  events: [{ type: "Played", seat: 3, card: 7 }],
};

const LEGAL: MarketDayAction[] = [{ Play: { card: 1 } }, { Play: { card: 3 } }, { Play: { card: 9 } }];

function board(view: MarketDayView, legal: MarketDayAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <MarketDayBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("Market Day board", () => {
  it("submits exactly the legal plays, one per control", () => {
    const sent: unknown[] = [];
    hostElements(board(PLAY, LEGAL, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toHaveLength(LEGAL.length);
    for (const action of LEGAL) expect(sent).toContainEqual(action);
  });

  it("lets the Tinker trade with a card of the round, or keep", () => {
    const tinker: MarketDayView = { ...PLAY, phase: "Tinker", trick: [[2, 8], [3, 7], [0, 1]] };
    const legal: MarketDayAction[] = [{ Swap: { with: 0 } }, { Swap: { with: 1 } }, "Keep"];
    const sent: unknown[] = [];
    hostElements(board(tinker, legal, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toEqual(legal);
    expect(renderToStaticMarkup(board(tinker, legal, 0))).toContain("Garder mon Bricoleur");
  });

  it("shows the round with who played what", () => {
    const html = renderToStaticMarkup(board(PLAY, LEGAL, 0));
    expect(html).toContain("Main 2 sur 6");
    expect(html).toContain("Cléo");
    expect(html).toContain("Le marché");
    expect(html).not.toMatch(/\d\s?%</);
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...PLAY, phase: "Over", turn: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeMarketDayEvent>[0]) => describeMarketDayEvent(e, NAMES);
    expect(say({ type: "Dealt", hand: 3, dealer: 0 })).toBe("Main 3 : Ana distribue.");
    expect(say({ type: "Played", seat: 1, card: 7 })).toBe("Hugo pose un 7 (Pluie).");
    expect(say({ type: "Played", seat: 1, card: 5 })).toBe("Hugo pose un 5.");
    expect(say({ type: "Swapped", seat: 0, with: 1, card: 8 })).toBe("Le Bricoleur d'Ana prend le 8 de Hugo.");
    expect(
      say({
        type: "TrickScored",
        cards: [
          [1, 1, "Scored"],
          [0, 8, "Scored"],
          [2, 10, "Killed"],
          [3, 6, "Spoiled"],
        ],
        gained: [8, 1, 0, 0],
        scores: [20, 4, 9, 0],
      }),
    ).toBe("Fin du tour : Ana +8 (20), Hugo +1 (4), Cléo +0 (9), Léa +0 (0). Tués : 10. Gâchés : 6.");
    expect(say({ type: "HandOver", scores: [20, 4, 9, 0] })).toBe("Fin de la main.");
  });
});
