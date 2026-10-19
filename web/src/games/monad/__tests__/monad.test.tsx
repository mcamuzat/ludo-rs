import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { MonadBoard, actionsForSelection, describeEvent } from "../Board";
import type { Card, MonadAction, MonadView } from "../types";

const NAMES = ["Ana", "Hugo"];
const c = (rank: Card["rank"], color: Card["color"]): Card => ({ rank, color });

const VIEW: MonadView = {
  players: 2,
  target: 5,
  identities: ["Yellow", "Sky"],
  monads: [1, 0],
  monadStock: 9,
  columns: [
    ["Red", "Orange", "Yellow", "Navy", "Sky", "Green"],
    ["Green", "Sky"],
    [],
    ["Navy"],
  ],
  discard: ["Red", "Green"],
  deck: 0,
  hand: [c("Unit", "Red"), c("Unit", "Navy"), c("Bi", "Red"), c("Bi", "Orange")],
  handCounts: [
    [2, 2, 0, 0, 0],
    [5, 1, 0, 0, 0],
  ],
  current: 0,
  turn: { transacted: false, prepared: false, bonusUsed: false, bonusOffer: null },
  thisTurn: [],
  lastTurn: [],
  winner: null,
  stalled: false,
  ranks: null,
};

const TRADE: MonadAction = { Trade: { cards: [c("Unit", "Red"), c("Unit", "Navy")] } };
const BUY_TRI: MonadAction = {
  Buy: { target: { Card: "Tri" }, cards: [c("Unit", "Red"), c("Bi", "Red"), c("Bi", "Orange")] },
};
const BUY_QUAD: MonadAction = {
  Buy: { target: { Card: "Quad" }, cards: [c("Unit", "Red"), c("Bi", "Red"), c("Bi", "Orange")] },
};
const LEAP: MonadAction = { Leap: { colors: ["Red", "Orange", "Navy", "Sky"] } };
const LEGAL: MonadAction[] = [TRADE, BUY_TRI, BUY_QUAD, LEAP, "Flip"];

function board(view: MonadView, legal: MonadAction[], me: number | null): string {
  return renderToStaticMarkup(
    <MonadBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("actionsForSelection", () => {
  it("finds the trade of the selected pair, in any order", () => {
    expect(actionsForSelection(LEGAL, [c("Unit", "Navy"), c("Unit", "Red")])).toEqual([TRADE]);
  });
  it("offers every target a payment can buy", () => {
    expect(actionsForSelection(LEGAL, [c("Bi", "Orange"), c("Unit", "Red"), c("Bi", "Red")])).toEqual([
      BUY_TRI,
      BUY_QUAD,
    ]);
  });
  it("finds a leap from its Units", () => {
    const units = [c("Unit", "Sky"), c("Unit", "Red"), c("Unit", "Orange"), c("Unit", "Navy")];
    expect(actionsForSelection(LEGAL, units)).toEqual([LEAP]);
  });
  it("finds nothing for a selection no move uses", () => {
    expect(actionsForSelection(LEGAL, [c("Bi", "Red")])).toEqual([]);
    expect(actionsForSelection(LEGAL, [])).toEqual([]);
  });
});

describe("MonadBoard", () => {
  it("shows the columns, the Monads, my hand and the others' counts", () => {
    const html = board(VIEW, LEGAL, 0);
    expect(html.match(/class="monad-column"/g)?.length).toBe(4);
    expect(html).toContain("Monades : 9");
    expect(html).toContain('aria-label="Bi orange"');
    expect(html).toContain("5 Unités");
    expect(html).toContain("1 Bi");
    expect(html).toContain("Retourner la défausse");
  });

  it("turns the Bonus card over once the bonus is taken", () => {
    const html = board({ ...VIEW, turn: { ...VIEW.turn, bonusUsed: true, transacted: true } }, ["EndTurn"], 0);
    expect(html).toContain("monad-bonus used");
    expect(html).toContain("Finir le tour");
  });

  it("asks about a waiting bonus", () => {
    const html = board(
      { ...VIEW, turn: { ...VIEW.turn, transacted: true, bonusOffer: "Tri" } },
      ["TakeBonus", "DeclineBonus"],
      0,
    );
    expect(html).toContain("Prendre le bonus");
    expect(html).toContain("Renoncer");
  });

  it("names the winner at the end", () => {
    const html = board({ ...VIEW, current: null, winner: 1, monads: [1, 5], ranks: [1, 0] }, [], 0);
    expect(html).toContain("Hugo gagne avec 5 Monades");
  });

  it("describes a move without showing another seat's drawn Unit", () => {
    expect(describeEvent({ seat: 1, action: "Draw", gained: [], monad: false, drew: true, drawn: null }, NAMES)).toBe(
      "Hugo pioche une Unité",
    );
    expect(
      describeEvent({ seat: 0, action: TRADE, gained: [c("Bi", "Sky")], monad: false, drew: false, drawn: null }, NAMES),
    ).toBe("Ana échange Unité rouge + Unité bleu marine contre Bi bleu ciel");
  });
});
