import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { CobrasBoard, cardName, trickSummary } from "../cobras/Board";
import type { CbAction, CbCard, CbSuit, CbView, Who } from "../cobras/types";

const C = (suit: CbSuit, value: number): CbCard => ({ suit, value });
const NAMES = ["Ana", "Hugo", "Cléo"];

const VIEW: CbView = {
  me: 0,
  phase: "Trick",
  hand: [C("Green", 9), C("Red", 3)],
  chosen: null,
  handCounts: [2, 2, 2],
  dummy: null,
  revealed: [C("Blue", 12), C("Red", 1), C("Yellow", 6)],
  baskets: [4, 0, 7],
  basketPoints: [8, 0, 21],
  center: 1,
  triples: 6,
  triplesToEnd: 10,
  kings: [true, false, true],
  trick: [{ who: { Seat: 2 }, played: { Card: C("Green", 4) } }],
  led: "Green",
  leader: { Seat: 2 },
  turn: { Seat: 0 },
  lastTrick: null,
  firstLeader: 1,
  scores: [12, 30, 5],
  target: 100,
  round: 2,
  dealer: 0,
  lastRound: null,
  legal: [{ Play: { card: C("Green", 9) } }, { King: { share: "Win" } }, { King: { share: "Loss" } }],
};

function render(v: CbView, me: number | null = 0) {
  return renderToStaticMarkup(
    <CobrasBoard
      view={v}
      legalActions={v.legal}
      pending={me === null ? [] : [me]}
      me={me}
      seatNames={NAMES.slice(0, v.handCounts.length)}
      submit={() => {}}
    />,
  );
}

describe("Cobras board", () => {
  it("names cards in French", () => {
    expect(cardName(C("Yellow", 13))).toBe("jaune 13");
  });

  it("only cards of the led suit and the King Cobra can be played", () => {
    const html = render(VIEW);
    expect(html).toContain('aria-label="vert 9"');
    expect(html).not.toContain('aria-label="rouge 3"');
    expect(html).toContain("Fournis le vert.");
    expect(html).toContain("Cobra Royal : partager la victoire");
    expect(html).toContain("Cobra Royal : partager la défaite");
  });

  it("shows baskets, the centre and the line of triples", () => {
    const html = render(VIEW);
    expect(html).toContain("Panier : 7 cobras");
    expect(html).toContain("(21 pts)");
    expect(html).toContain("Triples : 6 / 10");
    expect(html).toContain("Centre : 1 cobra");
    expect(html).toContain("Cobra Royal joué");
  });

  it("asks for the basket card in the seeding phase", () => {
    const v: CbView = {
      ...VIEW,
      phase: "Seed",
      trick: [],
      revealed: [null, null, null],
      legal: VIEW.hand.map((card): CbAction => ({ Seed: { card } })),
    };
    const html = render(v);
    expect(html).toContain("Choisis la carte qui amorce ton panier.");
    expect(html).toContain('aria-label="rouge 3"');
    expect(html).toContain("Hugo commencera le premier pli.");
  });

  it("seats the dummy and shows its top card", () => {
    const v: CbView = {
      ...VIEW,
      handCounts: [2, 2],
      dummy: { count: 5, top: C("Red", 11) },
      revealed: [null, null],
      baskets: [0, 0],
      basketPoints: [0, 0],
      kings: [true, true],
      trick: [{ who: "Dummy", played: { Card: C("Green", 4) } }],
      scores: [0, 0],
    };
    const html = render(v);
    expect(html).toContain("Fictif");
    expect(html).toContain('title="rouge 11"');
    expect(html).toContain("dans sa pile");
  });

  it("sums up a trick", () => {
    const name = (w: Who) => (w === "Dummy" ? "Fictif" : NAMES[w.Seat]);
    const text = trickSummary(
      {
        plays: [],
        winner: { Seat: 1 },
        scored: [
          { seat: 1, cobras: 4, points: 8 },
          { seat: 2, cobras: 7, points: 21 },
        ],
        losers: [{ Seat: 0 }],
        share: 6,
        center: 0,
      },
      name,
    );
    expect(text).toBe("Hugo gagne le pli. Hugo rend 4 cobras : +8 pts. Cléo rend 7 cobras : +21 pts. Ana reçoit 6 cobras.");
    expect(
      trickSummary({ plays: [], winner: null, scored: [], losers: [{ Seat: 0 }, "Dummy"], share: 2, center: 1 }, name),
    ).toBe("Personne ne gagne le pli avec une carte. Ana, Fictif reçoivent 2 cobras chacun. 1 cobra au centre.");
  });
});
