import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DeclarePicker, HornOfPlentyBoard, declarations, describeHornOfPlentyEvent } from "../horn-of-plenty/Board";
import type { HornOfPlentyAction, HornOfPlentyView } from "../horn-of-plenty/types";

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

function clicks(node: ReactNode): (() => void)[] {
  return hostElements(node)
    .filter((el) => typeof el.props.onClick === "function")
    .map((el) => el.props.onClick as () => void);
}

const NAMES = ["Ana", "Hugo", "Cléo"];

const ANSWER: HornOfPlentyView = {
  roundNo: 2,
  target: 11,
  phase: "Answer",
  seats: [
    { shown: [4, 9], cardPoints: 13, handSize: 2, score: 5 },
    { shown: [2], cardPoints: 2, handSize: 3, score: 3 },
    { shown: [7, 10], cardPoints: 17, handSize: 3, score: 6 },
  ],
  hand: [3, 8],
  table: { poser: 2, declared: 6, card: null, asked: 0 },
  turn: null,
  chooser: null,
  events: [{ type: "Declared", seat: 2, declared: 6 }],
};

const PLACE: HornOfPlentyView = {
  ...ANSWER,
  phase: "Place",
  table: { poser: 2, declared: 6, card: 5, asked: null },
  chooser: 0,
};

const ALL_RANKS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const DECLARE_LEGAL: HornOfPlentyAction[] = [3, 8].flatMap((card) =>
  ALL_RANKS.map((declared) => ({ Play: { card, declared } })),
);

function board(view: HornOfPlentyView, legal: HornOfPlentyAction[], me: number | null) {
  return (
    <HornOfPlentyBoard
      view={view}
      legalActions={legal}
      pending={[]}
      me={me}
      seatNames={NAMES}
      submit={() => {}}
    />
  );
}

describe("Horn of Plenty board", () => {
  it("shows the declaration, never the hidden card, and the answers", () => {
    const html = renderToStaticMarkup(board(ANSWER, ["Agree", "Disagree", "Pass"], 0));
    expect(html).toContain("Cléo annonce un 6");
    expect(html).toContain("D&#x27;accord, c&#x27;est un 6");
    expect(html).toContain("Pas d&#x27;accord");
    expect(html).toContain("Passer");
    expect(html).toContain("13 points de cartes");
    expect(html).toContain("Manche 2 · Partie en 11 points");
    expect(html).not.toMatch(/\d\s?%</);
    expect(renderToStaticMarkup(board(ANSWER, [], 1))).not.toContain("<button");
  });

  it("offers the legal recipients once the card is revealed", () => {
    const legal: HornOfPlentyAction[] = [{ Give: { to: 0 } }, { Give: { to: 2 } }];
    const html = renderToStaticMarkup(board(PLACE, legal, 0));
    expect(html).toContain("Prendre le 5");
    expect(html.match(/Lui donner le 5/g)).toHaveLength(1);
  });

  it("picks a card, then submits one of its declarations", () => {
    expect(declarations(DECLARE_LEGAL, 8)).toEqual(ALL_RANKS);
    const picked: (number | null)[] = [];
    const pick = (
      <DeclarePicker hand={[3, 8, 8]} legal={DECLARE_LEGAL} picked={null} onPick={(c) => picked.push(c)} submit={() => {}} />
    );
    // One control per distinct card.
    clicks(pick).forEach((click) => click());
    expect(picked).toEqual([3, 8]);
    const sent: unknown[] = [];
    const declare = (
      <DeclarePicker hand={[3, 8]} legal={DECLARE_LEGAL} picked={8} onPick={(c) => picked.push(c)} submit={(a) => sent.push(a)} />
    );
    clicks(declare).forEach((click) => click());
    expect(sent).toEqual(ALL_RANKS.map((declared) => ({ Play: { card: 8, declared } })));
    expect(picked).toEqual([3, 8, null]);
  });

  it("offers nothing after the game", () => {
    const over = renderToStaticMarkup(board({ ...ANSWER, phase: "Over", table: null }, [], 0));
    expect(over).not.toContain("<button");
    expect(over).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeHornOfPlentyEvent>[0]) => describeHornOfPlentyEvent(e, NAMES);
    expect(say({ type: "Dealt", round: 1, upcards: [3, 3, 8], tie_breaks: [[0, 6], [1, 2]], leader: 1 })).toBe(
      "Manche 1. Cartes visibles : Ana 3, Hugo 3, Cléo 8. Départage : Ana 6, Hugo 2 (défaussées). Hugo commence.",
    );
    expect(say({ type: "Declared", seat: 0, declared: 7 })).toBe("Ana pose une carte cachée : « c'est un 7 ».");
    expect(say({ type: "Answered", seat: 1, agreed: false, card: 7, right: false, chooser: 0 })).toBe(
      "Hugo n'est pas d'accord : c'est un 7, raté. Ana choisit qui le prend.",
    );
    expect(say({ type: "Unanswered", card: 4 })).toBe("Tout le monde passe : c'est un 4.");
    expect(say({ type: "Given", by: 0, to: 2, card: 9, paired: true })).toBe(
      "Ana donne le 9 à Cléo : paire, les deux 9 sont défaussés.",
    );
    expect(say({ type: "Given", by: 1, to: 1, card: 5, paired: false })).toBe("Hugo prend le 5.");
    expect(say({ type: "Scored", points: [18, 18, 5], gained: [3, 3, 1], scores: [8, 6, 4] })).toBe(
      "Fin de manche. Points de cartes : Ana 18, Hugo 18, Cléo 5. Points de partie : Ana +3 (8), Hugo +3 (6), Cléo +1 (4).",
    );
  });
});
