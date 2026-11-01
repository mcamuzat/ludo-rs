import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { CorporationBoard, endText, placeName, stepsText, stepText } from "../Board";
import type { Color, CorporationAction, CorporationView } from "../types";

const NAMES = ["Ana", "Hugo"];
const TABLE: Color[] = ["Green", "Red", "Red", "Yellow", "Blue", "Green", "Green", "Blue", "Red", "Blue", "Yellow", "Yellow"];
const ROOMS: (Color | null)[] = [
  "Red", "Blue", "Green", "Yellow",
  "Red", "Blue", "Green", "Yellow", "Red", "Blue", "Green", "Yellow",
  null,
];
const stacks = (): Color[][] => Array.from({ length: 13 }, () => []);

const VIEW: CorporationView = {
  me: 0,
  colors: ["Red", "Blue"],
  table: TABLE,
  supply: [0, 0, 0, 0],
  rooms: ROOMS,
  stacks: stacks(),
  outside: [3, 3, 3, 3],
  arrived: [0, 0, 0, 0],
  phase: "Swap",
  turn: 0,
  eligible: [],
  steps: [],
  lastTurn: null,
  quietLeft: 20,
  end: null,
  legal: ["Skip", { Swap: { a: 0, b: 1 } }],
};

function render(v: CorporationView, me: number | null = 0) {
  return renderToStaticMarkup(
    <CorporationBoard view={v} legalActions={v.legal} pending={v.turn === null ? [] : [v.turn]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("Corporation board", () => {
  it("names the places and steps", () => {
    expect(placeName(2)).toBe("l'accueil vert");
    expect(placeName(4)).toBe("1er étage");
    expect(placeName(10)).toBe("7e étage");
    expect(stepText({ Promote: { color: "Red", from: 1, to: 4 } })).toBe("un rouge monte de l'accueil bleu au 1er étage");
    expect(stepText({ Promote: { color: "Red", from: 11, to: 12 } })).toBe("un rouge monte du 8e étage à la salle du conseil");
    expect(stepText({ Swap: { a: 2, b: 3 } })).toBe("retourne la promotion 2");
    expect(stepText({ Swap: { a: 0, b: 9 } })).toBe("échange deux jetons (promotions 1 et 5)");
    expect(stepText({ Enter: { color: "Green", desk: 0 } })).toBe("fait entrer un employé vert à l'accueil rouge");
    expect(
      stepsText([
        { Promote: { color: "Green", from: 0, to: 4 } },
        { Promote: { color: "Green", from: 4, to: 5 } },
        { Promote: { color: "Blue", from: 5, to: 6 } },
      ]),
    ).toBe("un vert monte de l'accueil rouge au 2e étage ; un bleu monte du 2e étage au 3e étage");
  });

  it("offers the exchange and the skip", () => {
    const html = render(VIEW);
    expect(html).toContain('aria-label="promotion 1 gauche"');
    expect(html).toContain("Ne rien échanger");
    expect(html).toContain("neutre");
  });

  it("highlights the promotions to choose from", () => {
    const s = stacks();
    s[6] = ["Blue", "Red"];
    s[7] = ["Yellow"];
    const legal: CorporationAction[] = [{ Promote: { from: 6 } }, { Promote: { from: 7 } }];
    const html = render({ ...VIEW, stacks: s, phase: "Promote", eligible: [6, 7], legal });
    expect(html).toContain('aria-label="3e étage"');
    expect(html).toContain('aria-label="4e étage"');
    expect(html).toContain("eligible");
  });

  it("names the winner", () => {
    expect(endText({ Won: { seat: 1 } }, NAMES)).toBe("Hugo gagne : ses trois employés sont à la salle du conseil.");
    expect(endText("Quiet", NAMES)).toContain("Partie nulle");
    const html = render({ ...VIEW, turn: null, legal: [], end: { Won: { seat: 0 } }, arrived: [3, 1, 0, 0] });
    expect(html).toContain("3/3 au conseil");
  });
});
