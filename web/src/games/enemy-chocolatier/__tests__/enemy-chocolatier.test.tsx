import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import board from "../board.json";
import { endText, EnemyChocolatierBoard, eventText, place } from "../Board";
import type { BoardJson, ChocolatierAction, ChocolatierView, HoodView, HouseView } from "../types";

const NAMES = ["Ana", "Hugo", "Léa"];
const B = board as BoardJson;

function view(): ChocolatierView {
  const hoods: HoodView[] = [];
  const houses: HouseView[] = [];
  B.sections.forEach((section, s) =>
    section.hoods.forEach((h, index) => {
      const id = hoods.length;
      const ids = h.houses.map((_, k) => {
        houses.push({ hood: id, price: k + 1, owner: null });
        return houses.length - 1;
      });
      hoods.push({
        section: s,
        index,
        color: h.color === "Gray" ? null : h.color,
        dollars: 1,
        points: 0,
        ability: null,
        status: { kind: id === 1 ? "Unavailable" : "Open" },
        houses: ids,
      });
    }),
  );
  houses[0].owner = 1;
  hoods[2].status = { kind: "Closed", by: 2 };
  return {
    me: 0,
    players: 3,
    width: 1925,
    height: 1650,
    slots: [
      [0, 0],
      [0, 825],
      [550, 0],
      [550, 1100],
      [1375, 0],
      [1375, 825],
    ],
    factoryAt: [550, 550],
    layout: [0, 1, 2, 3, 4, 5].map((section) => ({ section, quarters: section === 2 || section === 3 ? 0 : 1 })),
    hoods,
    houses,
    seats: NAMES.map((_, i) => ({
      money: 4 + i,
      points: i,
      rainChecks: 0,
      incomeDollars: 2,
      incomePoints: 0,
      abilities: i === 1 ? ["BuyThree"] : [],
      ingredients: [1, 0, 2, 0],
      recipe: i === 0 ? { name: "Alice Corwin's High-Fat Toffee Sludge", need: [3, 8, 6, 2] } : null,
    })),
    turn: 0,
    bought: 0,
    limit: 2,
    offers: [
      { house: 3, cost: 4, rainCheck: false },
      { house: 0, cost: 6, rainCheck: false },
    ],
    events: [{ Income: { dollars: 2, points: 0 } }],
    lastTurn: { seat: 2, events: [{ Bought: { house: 0, from: null, cost: 1, rainCheck: false } }, { Closed: { hood: 2 } }] },
    targetPoints: 20,
    roundsLeft: 60,
    end: null,
  };
}

function render(v: ChocolatierView, legal: ChocolatierAction[] = [{ Buy: 3 }, { Buy: 0 }, "EndTurn"]) {
  return renderToStaticMarkup(
    <EnemyChocolatierBoard view={v} legalActions={legal} pending={[v.turn]} me={0} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("Enemy Chocolatier board", () => {
  it("turns points of a section with it", () => {
    expect(place([0, 0], 0, [10, 20])).toEqual([10, 20]);
    expect(place([0, 0], 1, [10, 20])).toEqual([530, 10]);
    expect(place([0, 0], 2, [10, 20])).toEqual([815, 530]);
    expect(place([0, 0], 3, [10, 20])).toEqual([20, 815]);
  });

  it("draws every house, the offers and the recipe", () => {
    const v = view();
    const html = render(v);
    expect(html.match(/class="ec-house-tile"/g)?.length).toBe(199);
    expect(html.match(/Acheter cette maison/g)?.length).toBe(2);
    expect(html).toContain("6 $");
    expect(html).toContain("Alice Corwin");
    expect(html).toContain("Ne rien acheter (prendre un bon)");
    expect(html).toContain("Achat ×3");
  });

  it("tells the turns and the end", () => {
    const v = view();
    expect(eventText({ Bought: { house: 0, from: 1, cost: 6, rainCheck: true } }, v, NAMES)).toBe(
      "achète une maison (quartier gris) 6 $ à Hugo avec un bon",
    );
    expect(eventText("RainCheck", v, NAMES)).toBe("prend un bon");
    expect(endText({ Recipe: 1 }, NAMES)).toBe("Hugo complète sa recette secrète et gagne !");
    const over = render({ ...v, end: { end: { Points: 2 }, ranks: [1, 1, 0] }, offers: [] }, []);
    expect(over).toContain("Léa atteint 20 points et gagne !");
    expect(over).not.toContain("Acheter cette maison");
  });
});
