import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { TwixtBoard, statusText } from "../Board";
import type { TwixtAction, TwixtView } from "../types";

const NAMES = ["Ana", "Hugo"];

const VIEW: TwixtView = {
  pegs: [
    [{ x: 5, y: 5 }, "Red"],
    [{ x: 6, y: 7 }, "Red"],
    [{ x: 15, y: 15 }, "Black"],
  ],
  links: [["Red", { x: 5, y: 5 }, { x: 6, y: 7 }]],
  turn: "Black",
  canSwap: false,
  moves: [{ Place: { x: 5, y: 5 } }, { Place: { x: 15, y: 15 } }, { Place: { x: 6, y: 7 } }],
  last: { x: 6, y: 7 },
  result: null,
  chain: null,
};

function render(v: TwixtView, legal: TwixtAction[], me: number | null = 1) {
  return renderToStaticMarkup(
    <TwixtBoard view={v} legalActions={legal} pending={v.result ? [] : [v.turn === "Red" ? 0 : 1]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("TwixT board", () => {
  it("renders pegs and links from the view", () => {
    const html = render(VIEW, []);
    expect(html.match(/tw-peg tw-red/g)).toHaveLength(2);
    expect(html.match(/tw-peg tw-black/g)).toHaveLength(1);
    expect(html.match(/tw-link tw-red/g)).toHaveLength(1);
    expect(html).toContain("tw-last");
  });

  it("makes only the legal holes clickable", () => {
    const html = render(VIEW, [{ Place: { x: 0, y: 3 } }, { Place: { x: 9, y: 9 } }]);
    expect(html).toContain('aria-label="trou A4"');
    expect(html).toContain('aria-label="trou J10"');
    expect(html.match(/role="button"/g)).toHaveLength(2);
  });

  it("shows the swap button only when it is legal", () => {
    const swapView = { ...VIEW, canSwap: true };
    expect(render(swapView, ["Swap", { Place: { x: 9, y: 9 } }])).toContain("Prendre son pion");
    expect(render(VIEW, [{ Place: { x: 9, y: 9 } }])).not.toContain("Prendre son pion");
    expect(render(swapView, [], 0)).not.toContain("Prendre son pion");
  });

  it("marks the winning chain at the end", () => {
    const won: TwixtView = { ...VIEW, result: { Won: "Red" }, chain: [{ x: 5, y: 5 }, { x: 6, y: 7 }] };
    const html = render(won, []);
    expect(html.match(/tw-chain/g)).toHaveLength(2);
    expect(statusText(won, NAMES)).toBe("Ana (Rouge) gagne.");
    expect(statusText({ ...VIEW, result: "Draw" }, NAMES)).toBe("Partie nulle : plus personne ne peut relier ses bords.");
    expect(statusText(VIEW, NAMES)).toBe("À Hugo (Noir) de jouer.");
  });
});
