import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { AskControls, BlackViennaBoard, TrioPicker, cycleMark, describeEvent, findAsk, pickTrio } from "../Board";
import { Sheet } from "../Sheet";
import type { BlackViennaAction, BlackViennaView } from "../types";

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

function clickAll(node: ReactNode) {
  hostElements(node)
    .filter((el) => typeof el.props.onClick === "function" && !el.props.disabled)
    .forEach((el) => (el.props.onClick as () => void)());
}

const NAMES = ["Ana", "Hugo", "Cléo"];
const SUSPECTS = "ABCDEFGHIJKLMNOPQRSTUVWXYZÖ".split("");
const INVESTIGATIONS = ["ACL", "AGM", "AOS", "APQ", "BCY", "BHV", "BLM"];

const VIEW: BlackViennaView = {
  phase: "Turn",
  turn: 0,
  players: 3,
  dealer: 0,
  hand: ["A", "C", "D"],
  handSizes: [8, 8, 8],
  reserve: 37,
  piles: [
    { top: 6, left: 11 },
    { top: 1, left: 12 },
    { top: null, left: 0 },
  ],
  suspects: SUSPECTS,
  investigations: INVESTIGATIONS,
  answers: [{ by: 1, target: 0, card: 4, count: 0, placed: 0, special: false }],
  wrong: [false, false, false],
  asked: [true, false, false],
  canAccuse: true,
  specialAllowed: false,
  reusable: [],
  last: null,
  finalAsked: false,
  myGuess: null,
  trio: null,
  accuser: null,
  guesses: [],
  scores: [0, 0, 0],
  notes: [
    { suspect: "A", column: { Seat: 0 }, mark: "Plus" },
    { suspect: "A", column: "Trio", mark: "Minus" },
  ],
  events: [],
};

const ASKS: BlackViennaAction[] = [0, 1].flatMap((pile) =>
  [1, 2].map((target) => ({ Ask: { pile, target, extra: null } })),
);

describe("Black Vienna board", () => {
  it("picks three distinct letters in the sheet's order", () => {
    expect(pickTrio(["C", "A", "B"], SUSPECTS)).toEqual(["A", "B", "C"]);
    expect(pickTrio(["A", "A", "B"], SUSPECTS)).toBeNull();
    expect(pickTrio(["Ö", "A", "Z"], SUSPECTS)).toEqual(["A", "Z", "Ö"]);
    expect(pickTrio(["A", "B"], SUSPECTS)).toBeNull();
  });

  it("cycles a manual mark through + − ? and back", () => {
    expect([undefined, "+", "−", "?"].map((m) => cycleMark(m as never))).toEqual(["+", "−", "?", undefined]);
  });

  it("asks only legal questions, one control per target", () => {
    expect(findAsk(ASKS, 0, 2, null)).toEqual({ Ask: { pile: 0, target: 2, extra: null } });
    expect(findAsk(ASKS, 0, 0, null)).toBeUndefined();
    const sent: unknown[] = [];
    clickAll(
      <AskControls
        view={VIEW}
        legal={ASKS}
        me={0}
        names={NAMES}
        pile={1}
        extra={null}
        onPile={() => {}}
        onExtra={() => {}}
        submit={(a) => sent.push(a)}
      />,
    );
    // Pile buttons pick, target buttons submit: the two targets of pile 1.
    expect(sent).toEqual([
      { Ask: { pile: 1, target: 1, extra: null } },
      { Ask: { pile: 1, target: 2, extra: null } },
    ]);
  });

  it("offers a special investigation only when allowed", () => {
    const html = renderToStaticMarkup(
      <AskControls view={VIEW} legal={ASKS} me={0} names={NAMES} pile={0} extra={null} onPile={() => {}} onExtra={() => {}} submit={() => {}} />,
    );
    expect(html).not.toContain("Enquête spéciale");
    const special = renderToStaticMarkup(
      <AskControls
        view={{ ...VIEW, specialAllowed: true, reusable: [4] }}
        legal={[...ASKS, { Ask: { pile: 0, target: 1, extra: 4 } }]}
        me={0}
        names={NAMES}
        pile={0}
        extra={null}
        onPile={() => {}}
        onExtra={() => {}}
        submit={() => {}}
      />,
    );
    expect(special).toContain("Enquête spéciale");
    expect(special).toContain("BCY");
  });

  it("submits the legal accusation of the letters picked", () => {
    const legal: BlackViennaAction[] = [{ Accuse: { trio: ["A", "B", "C"] } }];
    const sent: unknown[] = [];
    clickAll(
      <TrioPicker
        suspects={SUSPECTS}
        chosen={["C", "B", "A"]}
        onToggle={() => {}}
        label="Accuser"
        action={(trio) => legal.find((a) => "Accuse" in a && a.Accuse.trio.join("") === trio.join(""))}
        submit={(a) => sent.push(a)}
      />,
    );
    expect(sent).toContainEqual({ Accuse: { trio: ["A", "B", "C"] } });
  });

  it("keeps automatic notes fixed and manual marks per seat", () => {
    const cycled: string[] = [];
    const sheet = (marks: Record<string, "+" | "−" | "?">) => (
      <Sheet
        suspects={SUSPECTS}
        players={3}
        names={NAMES}
        notes={VIEW.notes}
        marks={marks}
        onCycle={(key) => cycled.push(key)}
      />
    );
    const els = hostElements(sheet({}));
    const autoA0 = els.find((el) => el.props["data-cell"] === "A:0");
    expect(autoA0?.props.onClick).toBeUndefined();
    const b1 = els.find((el) => el.props["data-cell"] === "B:1");
    (b1?.props.onClick as () => void)();
    expect(cycled).toEqual(["B:1"]);
    expect(renderToStaticMarkup(sheet({ "B:1": "?" }))).toContain("?");
    expect(renderToStaticMarkup(sheet({}))).not.toContain(">?<");
  });

  it("offers nothing out of turn, watching, or after the game", () => {
    const board = (view: BlackViennaView, legal: BlackViennaAction[], me: number | null) =>
      renderToStaticMarkup(
        <BlackViennaBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
      );
    const playing = board(VIEW, [...ASKS, { Accuse: { trio: ["B", "E", "F"] } }], 0);
    expect(playing).toContain("Black Vienna !");
    expect(playing).toContain("Réserve : 37");
    expect(playing).not.toMatch(/\d\s?%</);
    expect(board({ ...VIEW, turn: 1, canAccuse: false }, [], 0)).not.toContain("Black Vienna !");
    expect(board({ ...VIEW, hand: [], canAccuse: false }, [], null)).not.toContain("Interroger");
    const over = board(
      { ...VIEW, phase: "Over", turn: null, canAccuse: false, trio: ["L", "M", "Ö"], scores: [111, 0, 37] },
      [],
      0,
    );
    expect(over).toContain("L-M-Ö");
    expect(over).toContain("111");
    expect(over).not.toContain("Interroger");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeEvent>[0]) => describeEvent(e, NAMES, INVESTIGATIONS);
    expect(say({ type: "Asked", by: 0, target: 1, answers: [{ by: 0, target: 1, card: 6, count: 2, placed: 2, special: false }] })).toBe(
      "Ana interroge Hugo : BLM, 2 jetons.",
    );
    expect(say({ type: "Accused", seat: 2, right: false })).toBe("Cléo accuse… et se trompe.");
    expect(
      say({ type: "Revealed", end: "Solved", trio: ["L", "M", "Ö"], accuser: 0, guesses: [], scores: [111, 0, 37] }),
    ).toBe("Black Vienna : L-M-Ö. Démasqué par Ana. Points : Ana 111, Hugo 0, Cléo 37.");
    expect(
      say({ type: "Revealed", end: "Reserve", trio: ["L", "M", "Ö"], accuser: null, guesses: [], scores: [0, 0, 0] }),
    ).toContain("Plus assez de jetons");
  });
});
