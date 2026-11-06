import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { TwixtBoard, TwixtSvg } from "../Board";
import { CoachPanel } from "../Coach";
import { ReviewPanel } from "../Review";
import { adviceFor, computeReview, positionKey, type CoachLike } from "../coach";
import type { Advice, Review, TwixtAction, TwixtView } from "../types";

const NAMES = ["Ana", "Hugo (IA)"];
const MOVES: TwixtAction[] = [{ Place: { x: 5, y: 5 } }, { Place: { x: 15, y: 15 } }];

const VIEW: TwixtView = {
  pegs: [
    [{ x: 5, y: 5 }, "Red"],
    [{ x: 15, y: 15 }, "Black"],
  ],
  links: [],
  turn: "Red",
  canSwap: false,
  moves: MOVES,
  last: { x: 15, y: 15 },
  result: null,
  chain: null,
};

const ADVICE: Advice = {
  win: 0.62,
  best: [
    { action: { Place: { x: 6, y: 7 } }, win: 0.7 },
    { action: { Place: { x: 4, y: 7 } }, win: 0.66 },
    { action: "Swap", win: 0.6 },
  ],
  heat: [
    [{ x: 6, y: 7 }, 0.5],
    [{ x: 4, y: 7 }, 0.3],
  ],
  last: { ply: 0, loss: 0.12, grade: "Mistake" },
};

function board(view: TwixtView, assist?: { vsAi: boolean }) {
  return renderToStaticMarkup(
    <TwixtBoard
      view={view}
      legalActions={view.result ? [] : [{ Place: { x: 9, y: 9 } }]}
      pending={view.result ? [] : [0]}
      me={0}
      seatNames={NAMES}
      submit={() => {}}
      assist={assist && { ...assist, takeBack: () => {} }}
    />,
  );
}

describe("TwixT coach", () => {
  it("shows nothing until clicked", () => {
    const html = board(VIEW, { vsAi: true });
    expect(html).toContain("Coach");
    expect(html).toContain("Annuler et réessayer");
    expect(html).not.toContain("Tes chances");
  });

  it("is only offered against the AI", () => {
    expect(board(VIEW, { vsAi: false })).not.toContain("Coach");
    expect(board(VIEW)).not.toContain("Coach");
    expect(board(VIEW)).not.toContain("Annuler et réessayer");
  });

  it("shows the win, the last move's grade and three candidates", () => {
    const html = renderToStaticMarkup(
      <CoachPanel advice={ADVICE} heat={false} onHeat={() => {}} onHover={() => {}} />,
    );
    expect(html).toContain("Tes chances : 62 %");
    expect(html).toContain("Ton dernier coup : Erreur (−12 points)");
    expect(html).toContain("G8 — 70 %");
    expect(html).toContain("E8 — 66 %");
    expect(html).toContain("Swap — 60 %");
    const best = renderToStaticMarkup(
      <CoachPanel advice={{ ...ADVICE, last: { ply: 0, loss: 0.004, grade: "Best" } }} heat={false} onHeat={() => {}} onHover={() => {}} />,
    );
    expect(best).toContain("Ton dernier coup : Meilleur (aucune perte)");
    const good = renderToStaticMarkup(
      <CoachPanel advice={{ ...ADVICE, last: { ply: 0, loss: 0.012, grade: "Good" } }} heat={false} onHeat={() => {}} onHover={() => {}} />,
    );
    expect(good).toContain("Ton dernier coup : Bon (−1 point)");
  });

  it("never shows advice asked for another position with as many moves", () => {
    const stored = { key: positionKey(MOVES), advice: ADVICE };
    expect(adviceFor(stored, MOVES)).toBe(ADVICE);
    const other: TwixtAction[] = [{ Place: { x: 5, y: 5 } }, { Place: { x: 16, y: 15 } }];
    expect(adviceFor(stored, other)).toBeNull();
    expect(adviceFor(null, MOVES)).toBeNull();
  });

  it("draws a ghost peg and the heat map on the board", () => {
    const html = renderToStaticMarkup(<TwixtSvg view={VIEW} ghost={{ at: { x: 6, y: 7 }, color: "Red" }} heat={ADVICE.heat} />);
    expect(html).toContain("tw-ghost");
    expect(html.match(/tw-heat/g)).toHaveLength(2);
  });
});

describe("TwixT review", () => {
  it("is offered at the end of a local game, not online", () => {
    const ended: TwixtView = { ...VIEW, result: { Won: "Red" } };
    expect(board(ended, { vsAi: true })).toContain("Revue de la partie");
    expect(board(ended, { vsAi: false })).toContain("Revue de la partie");
    expect(board(ended)).not.toContain("Revue de la partie");
    expect(board(VIEW, { vsAi: true })).not.toContain("Revue de la partie");
  });

  it("evaluates every position, reporting progress, then asks for the grades", async () => {
    const review: Review = { grades: [null, "Best", "Blunder"], key: [2] };
    const points = [
      { redWin: 0.5, best: 0.6 },
      { redWin: 0.55, best: 0.5 },
      { redWin: 0.2, best: null },
    ];
    const client = {
      advise: vi.fn(),
      reviewPoint: vi.fn(async (_moves: unknown[], ply: number) => points[ply]),
      review: vi.fn(async () => review),
      position: vi.fn(),
    } as unknown as CoachLike;
    const progress = vi.fn();
    const result = await computeReview(client, MOVES, progress);
    expect(result).toEqual({ curve: [0.5, 0.55, 0.2], review });
    expect(progress).toHaveBeenLastCalledWith(3, 3);
    expect(client.review).toHaveBeenCalledWith(MOVES, points);
  });

  it("draws the curve, the grades and the key moments", () => {
    const review: Review = { grades: [null, "Best", "Blunder"], key: [2] };
    const html = renderToStaticMarkup(
      <ReviewPanel curve={[0.5, 0.55, 0.2]} review={review} names={NAMES} selected={2} onSelect={() => {}} />,
    );
    expect(html).toContain("tw-grade-Blunder");
    expect(html).toContain("tw-grade-Best");
    expect(html).toContain("Coup 2 (Hugo (IA), Noir) : Gaffe");
    expect(html).toContain("Position après le coup 2");
  });
});
