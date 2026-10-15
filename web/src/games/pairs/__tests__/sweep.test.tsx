import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { SweepBoard, describeSweepEvent } from "../sweep/Board";
import type { SweepAction, SweepView } from "../sweep/types";

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

const PLAY: SweepView = {
  roundNo: 2,
  roundsTotal: 8,
  phase: "Play",
  handSizes: [5, 4, 6],
  hand: [2, 7, 7, 9, 10],
  targets: [3, 4, 4, 6, 10],
  shooters: [null, 5, null],
  leader: 1,
  turn: 0,
  scores: [],
  events: [{ type: "Shot", seat: 1, card: 5 }],
};

const PLAY_LEGAL: SweepAction[] = [2, 7, 9, 10].map((card) => ({ Shoot: { card } }));

function board(view: SweepView, legal: SweepAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <SweepBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: SweepView, legal: SweepAction[]): unknown[] {
  const sent: unknown[] = [];
  hostElements(board(view, legal, 0, (a) => sent.push(a)))
    .filter((el) => typeof el.props.onClick === "function")
    .forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Sweep board", () => {
  it("submits exactly the legal shooters and captures", () => {
    const sent = clickEverything(PLAY, PLAY_LEGAL);
    expect(sent).toHaveLength(PLAY_LEGAL.length);
    for (const action of PLAY_LEGAL) expect(sent).toContainEqual(action);
    const capture: SweepAction[] = [{ Capture: { card: 3 } }, { Capture: { card: 4 } }];
    const got = clickEverything({ ...PLAY, phase: "Capture" }, capture);
    expect(got).toEqual(capture);
  });

  it("shows the targets, the shooters and my hand", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Manche 2 / 8");
    expect(html).toContain("tireur");
    expect(html).toContain("Tirer ce 7");
    expect(html).toContain("Choisis ton tireur.");
  });

  it("offers nothing out of turn or watching, and shows the hands at the end", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(PLAY, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(
      board(
        {
          ...PLAY,
          phase: "Over",
          turn: null,
          scores: [14, 18, 12],
          events: [{ type: "Scored", hands: [[10, 10], [9], [3]], ranks: [], scores: [14, 18, 12] }],
        },
        [],
        0,
      ),
    );
    expect(over).toContain("Partie terminée");
    expect(over).toContain("18 points");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeSweepEvent>[0]) => describeSweepEvent(e, NAMES);
    expect(say({ type: "TargetsDealt", round: 1, targets: [3, 4, 4, 6, 10], leader: 2 })).toBe(
      "Manche 1 : cibles 3, 4, 4, 6, 10. Cléo tire en premier.",
    );
    expect(say({ type: "Captured", seat: 1, cards: [3, 4, 4, 5] })).toBe("Hugo ramasse 3, 4, 4, 5.");
  });
});
