// What the board previews and sends, kept apart from rendering so it can be tested.
import type { Dir, HandDomino, LegalMove, Placement } from "./types";

/** How much the board helps to find a placement. */
export type Aid = "none" | "playable" | "places";

export type Cell = [number, number];

export const DOMINO_BODY = "#1b1b1b";
/** Seat outlines, all light enough to stand out from a domino's body. */
export const SEAT_COLORS = ["#ffffff", "#4cc9f0", "#c77dff", "#ff8fab"];

export interface Preview {
  p: Placement;
  /** Values on the anchor and on the other cell. */
  sides: [number, number];
  /** The legal move it stands for; null when illegal. */
  move: LegalMove | null;
}

export function cellsOf(row: number, col: number, dir: Dir): [Cell, Cell] {
  return [[row, col], dir === "Right" ? [row, col + 1] : [row + 1, col]];
}

export const key = ([r, c]: Cell) => `${r},${c}`;

/** The legal placements of a domino of `d`'s values (the engine lists only the lower twin). */
export function placementsOf(hand: HandDomino[], legal: LegalMove[], d: HandDomino) {
  const valuesOf = new Map(hand.map((h) => [h.id, [h.a, h.b] as const]));
  return legal.flatMap((move) => {
    if (move.action === "Pass") return [];
    const p = move.action.Place;
    const v = valuesOf.get(p.domino);
    return v && v[0] === d.a && v[1] === d.b ? [{ p, move }] : [];
  });
}

const sidesOf = (d: HandDomino, flipped: boolean): [number, number] => (flipped ? [d.b, d.a] : [d.a, d.b]);

/**
 * The placement previewed on the touched cell. With the "places" aid, the
 * `choice`-th legal placement of `d` covering that cell (none if no legal one
 * does); otherwise `d` anchored there with the current direction and flip,
 * legal or not.
 */
export function previewAt(
  hand: HandDomino[],
  legal: LegalMove[],
  aid: Aid,
  d: HandDomino,
  cursor: Cell,
  dir: Dir,
  flipped: boolean,
  choice: number,
): Preview | null {
  const mine = placementsOf(hand, legal, d);
  if (aid === "places") {
    const here = mine.filter(({ p }) => cellsOf(p.row, p.col, p.dir).some((c) => key(c) === key(cursor)));
    if (here.length === 0) return null;
    const { p, move } = here[choice % here.length];
    return { p, sides: sidesOf(d, p.flipped), move };
  }
  const sides = sidesOf(d, flipped);
  const p: Placement = { domino: d.id, row: cursor[0], col: cursor[1], dir, flipped };
  const match = mine.find(({ p: q }) => {
    const s = sidesOf(d, q.flipped);
    return q.row === p.row && q.col === p.col && q.dir === p.dir && s[0] === sides[0] && s[1] === sides[1];
  });
  return { p, sides, move: match?.move ?? null };
}

/** A click on `cell` of the preview: it places from the touched cell, and moves the cursor from the other half. */
export function previewClick(_preview: Preview, cell: Cell, cursor: Cell): "confirm" | "move" {
  return key(cell) === key(cursor) ? "confirm" : "move";
}
