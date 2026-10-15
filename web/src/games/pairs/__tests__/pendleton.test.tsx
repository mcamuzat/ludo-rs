import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PendletonBoard, describePendletonEvent } from "../pendleton/Board";
import type { PendletonAction, PendletonView } from "../pendleton/types";

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

const NAMES = ["Ana", "Hugo"];

const VIEW: PendletonView = {
  roundNo: 2,
  targetScore: 100,
  phase: "Play",
  columns: [
    { target: 2, sides: [[7], []], points: [7, 0] },
    { target: 4, sides: [[3], [3, 9]], points: [3, 12] },
    { target: 6, sides: [[], [8]], points: [0, 8] },
    { target: 8, sides: [[], []], points: [0, 0] },
    { target: 9, sides: [[], [5]], points: [0, 5] },
  ],
  drawn: 5,
  turn: 0,
  scores: [31, 44],
  deckSize: 42,
  events: [{ type: "Played", seat: 1, card: 9, column: 1, target: 4 }],
};

const LEGAL: PendletonAction[] = [0, 1, 2, 3].map((column) => ({ Play: { column } }));

function board(view: PendletonView, legal: PendletonAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <PendletonBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

describe("Pendleton board", () => {
  it("submits exactly the legal columns, one per target", () => {
    const sent: unknown[] = [];
    hostElements(board(VIEW, LEGAL, 0, (a) => sent.push(a)))
      .filter((el) => typeof el.props.onClick === "function")
      .forEach((el) => (el.props.onClick as () => void)());
    expect(sent).toHaveLength(LEGAL.length);
    for (const action of LEGAL) expect(sent).toContainEqual(action);
  });

  it("shows the columns, the drawn card and the scores", () => {
    const html = renderToStaticMarkup(board(VIEW, LEGAL, 0));
    expect(html).toContain("Manche 2");
    expect(html).toContain("31 / 100");
    expect(html).toContain("44 / 100");
    expect(html).toContain("Poser ton 5 dans la colonne du 2");
    expect(html).not.toContain("colonne du 9");
    expect(html).toContain("Ana a pioché");
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    expect(renderToStaticMarkup(board({ ...VIEW, turn: 1 }, [], 0))).not.toContain("<button");
    expect(renderToStaticMarkup(board(VIEW, [], null))).not.toContain("<button");
    const over = renderToStaticMarkup(board({ ...VIEW, phase: "Over", turn: null, drawn: null }, [], 0));
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describePendletonEvent>[0]) => describePendletonEvent(e, NAMES);
    expect(say({ type: "RoundStarted", targets: [1, 3, 5, 7, 9], first: 1 })).toBe(
      "Nouvelle manche, cibles 1, 3, 5, 7, 9. Hugo commence.",
    );
    expect(say({ type: "Played", seat: 0, card: 7, column: 0, target: 2 })).toBe(
      "Ana pose un 7 dans la colonne du 2.",
    );
    expect(
      say({ type: "RoundScored", seat: 1, card: 7, columns: [], round: [1, 18], totals: [32, 62] }),
    ).toBe("Hugo pioche un 7, injouable partout : fin de la manche. Ana 1 point, Hugo 18 points (total : Ana 32, Hugo 62).");
  });
});
