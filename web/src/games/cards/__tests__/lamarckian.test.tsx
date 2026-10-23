import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { LamarckianBoard, cardAction, describeLamarckianEvent, handName } from "../lamarckian/Board";
import type { LamarckianAction, LamarckianView } from "../lamarckian/types";
import type { Card, Rank, Suit } from "../types";

const N = (rank: Rank, suit: Suit = "Hearts"): Card => ({ kind: "Natural", rank, suit });
const NAMES = ["Ana", "Hugo", "Cléo"];

const VIEW: LamarckianView = {
  me: 0,
  competition: 2,
  competitions: 4,
  turnNo: 3,
  turns: 7,
  lowest: "Four",
  phase: "Choose",
  pool: [N("King", "Spades"), N("Nine")],
  deckSize: 16,
  hand: [N("King"), N("Queen"), N("Five", "Clubs"), N("Four", "Diamonds")],
  best: { category: "HighCard", cards: [N("King"), N("Queen"), N("Five", "Clubs"), N("Four", "Diamonds")] },
  chosen: null,
  picking: null,
  seats: [
    { handSize: 4, known: [], score: 3, mark: null, waiting: false },
    { handSize: 6, known: [N("Ace", "Clubs"), N("Ace", "Spades")], score: 5, mark: null, waiting: false },
    { handSize: 4, known: [], score: 1, mark: null, waiting: false },
  ],
  lastShowdown: null,
  events: [],
};

const MARKS: LamarckianAction[] = VIEW.hand.map((card) => ({ Mark: { card } }));

function board(view: LamarckianView, legal: LamarckianAction[]): string {
  return renderToStaticMarkup(
    <LamarckianBoard view={view} legalActions={legal} pending={[]} me={view.me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("handName", () => {
  it("names each hand", () => {
    expect(handName({ category: "Pair", cards: [N("Ace"), N("Ace", "Spades")] })).toBe("Paire d'as");
    expect(handName({ category: "Trips", cards: [N("King"), N("King"), N("King")] })).toBe("Brelan de rois");
    expect(
      handName({ category: "TwoPair", cards: [N("Queen"), N("Queen"), N("Five"), N("Five"), N("Two")] }),
    ).toBe("Deux paires, dames et cinq");
    expect(
      handName({ category: "FullHouse", cards: [N("Nine"), N("Nine"), N("Nine"), N("Jack"), N("Jack")] }),
    ).toBe("Full aux neuf par les valets");
    expect(
      handName({ category: "Straight", cards: [N("Ten"), N("Nine"), N("Eight"), N("Seven"), N("Six")] }),
    ).toBe("Quinte au dix");
    expect(handName({ category: "HighCard", cards: [N("King")] })).toBe("Hauteur roi");
  });
});

describe("cardAction", () => {
  it("finds the legal mark or pick", () => {
    expect(cardAction(N("King"), MARKS, "Mark")).toEqual({ Mark: { card: N("King") } });
    expect(cardAction(N("King"), MARKS, "Pick")).toBeNull();
  });
});

describe("LamarckianBoard", () => {
  it("shows my hand, my best hand and the cards others were seen to take", () => {
    const html = board(VIEW, MARKS);
    expect(html).toContain("Concours 2/4 · tour 3/7");
    expect(html).toContain('aria-label="roi de cœur"');
    expect(html).toContain("Ta meilleure main : Hauteur roi");
    expect(html).toContain('title="as de trèfle"');
    expect(html).toContain("Choisis ta carte");
  });

  it("lets the picker choose a pool card", () => {
    const view: LamarckianView = { ...VIEW, phase: "Pick", picking: [0, N("Four", "Diamonds")] };
    const html = board(view, [{ Pick: { card: N("Nine") } }, { Pick: { card: N("King", "Spades") } }]);
    expect(html).toContain("ne prend rien");
    expect(html).toContain('aria-label="9 de cœur"');
  });

  it("words the events", () => {
    expect(
      describeLamarckianEvent({ type: "Captured", seat: 1, mark: N("Ace"), cards: [N("Nine")] }, NAMES),
    ).toBe("Hugo (as de cœur) prend 9 de cœur.");
  });
});
