import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PresidentBoard, describePresidentEvent, selectionAction } from "../president/Board";
import type { PresidentAction, PresidentView } from "../president/types";
import type { Card, Rank, Suit } from "../types";

const N = (rank: Rank, suit: Suit = "Spades"): Card => ({ kind: "Natural", rank, suit });
const JOKER: Card = { kind: "Joker" };
const NAMES = ["Ana", "Hugo", "Cléo", "Malo"];

const PLAY: PresidentView = {
  roundNo: 2,
  roundsTotal: 5,
  phase: "Play",
  reversed: false,
  handSizes: [3, 5, 4, 0],
  hand: [N("Seven"), N("Seven", "Hearts"), JOKER],
  trick: { seat: 1, rank: "Six", cards: [N("Six"), N("Six", "Hearts")] },
  passes: [2],
  turn: 0,
  titles: ["President", "Scum", "Secretary", "VicePresident"],
  scores: [3, 0, 1, 2],
  out: [3],
  fouled: [],
  setAside: [],
  received: [N("Two", "Clubs"), N("King")],
  given: [N("Three"), N("Four")],
  toGive: [],
  events: [
    { type: "Played", seat: 1, rank: "Six", cards: [N("Six"), N("Six", "Hearts")] },
    { type: "Passed", seat: 2 },
  ],
};

const PLAY_LEGAL: PresidentAction[] = [
  { Play: { rank: "Seven", naturals: 2, jokers: 0 } },
  { Play: { rank: "Seven", naturals: 1, jokers: 1 } },
  "Pass",
];

const EXCHANGE: PresidentView = { ...PLAY, phase: "Exchange", trick: null, passes: [], turn: null, out: [], toGive: [0, 3] };
const GIVE: PresidentAction = {
  Give: { faces: [{ kind: "Natural", rank: "Three" }, { kind: "Natural", rank: "Five" }] },
};

function board(view: PresidentView, legal: PresidentAction[], me: number | null): string {
  return renderToStaticMarkup(
    <PresidentBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("selectionAction", () => {
  it("finds the play made of exactly the selected cards", () => {
    expect(selectionAction([N("Seven"), N("Seven", "Hearts")], PLAY_LEGAL)).toEqual(PLAY_LEGAL[0]);
    expect(selectionAction([N("Seven"), JOKER], PLAY_LEGAL)).toEqual(PLAY_LEGAL[1]);
    expect(selectionAction([N("Seven")], PLAY_LEGAL)).toBeNull();
    expect(selectionAction([], PLAY_LEGAL)).toBeNull();
  });

  it("finds nothing for mixed ranks or a card too many", () => {
    expect(selectionAction([N("Seven"), N("Eight")], PLAY_LEGAL)).toBeNull();
    expect(selectionAction([N("Seven"), N("Seven", "Hearts"), JOKER], PLAY_LEGAL)).toBeNull();
  });

  it("matches jokers alone, and handed-back cards in any order", () => {
    const pair: PresidentAction = { Play: { rank: null, naturals: 0, jokers: 2 } };
    expect(selectionAction([JOKER, JOKER], [pair])).toEqual(pair);
    expect(selectionAction([N("Five", "Hearts"), N("Three", "Clubs")], [GIVE])).toEqual(GIVE);
    expect(selectionAction([N("Five")], [GIVE])).toBeNull();
  });
});

describe("PresidentBoard", () => {
  it("shows the trick, my hand, the titles, and Jouer off until a play is selected", () => {
    const html = board(PLAY, PLAY_LEGAL, 0);
    expect(html).toContain("Trou du cul");
    expect(html).toContain("Président");
    expect(html).toContain('disabled="">Jouer</button>');
    expect(html).toContain("Passer");
    expect(html).toContain("reçu 2 de trèfle, roi de pique");
    expect(html).toContain('aria-label="7 de cœur"');
    expect(html).not.toContain("Révolution");
  });

  it("shows the revolution", () => {
    expect(board({ ...PLAY, reversed: true }, PLAY_LEGAL, 0)).toContain("Révolution");
  });

  it("asks the President to hand cards back", () => {
    const html = board(EXCHANGE, [GIVE], 0);
    expect(html).toContain("Rendre");
    expect(html).toContain("Rends 2 cartes");
    expect(html).not.toContain("Passer");
  });

  it("shows no hand to a spectator", () => {
    const html = board({ ...PLAY, hand: [] }, [], null);
    expect(html).not.toContain("aria-pressed");
    expect(html).not.toContain(">Jouer<");
  });
});

describe("describePresidentEvent", () => {
  it("tells what happened", () => {
    expect(describePresidentEvent({ type: "Passed", seat: 2 }, NAMES)).toBe("Cléo passe.");
    expect(describePresidentEvent({ type: "Exchanged", from: 1, to: 0, count: 2 }, NAMES)).toBe(
      "Hugo donne 2 cartes à Ana.",
    );
    expect(describePresidentEvent({ type: "Revolution", reversed: true }, NAMES)).toContain("Révolution");
    expect(describePresidentEvent({ type: "Finished", seat: 3, fouled: true }, NAMES)).toContain("interdite");
    expect(describePresidentEvent({ type: "Played", seat: 0, rank: null, cards: [JOKER] }, NAMES)).toBe(
      "Ana joue joker.",
    );
  });
});
