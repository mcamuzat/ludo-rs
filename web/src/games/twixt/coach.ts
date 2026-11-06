// Display helpers for TwixT's coach. Every number comes from the engine
// (games/twixt/src/coach.rs); this file only sequences the calls and formats.

import type { Grade, Review, ReviewPoint, TwixtAction } from "./types";

/** What the board needs from `CoachClient` (a fake in tests). */
export interface CoachLike {
  advise<T>(moves: unknown[]): Promise<T>;
  reviewPoint<T>(moves: unknown[], ply: number): Promise<T>;
  review<T>(moves: unknown[], points: unknown[]): Promise<T>;
  position<T>(moves: unknown[]): Promise<T>;
}

export const GRADE_NAMES: Record<Grade, string> = {
  Best: "Meilleur",
  Good: "Bon",
  Inaccuracy: "Imprécision",
  Mistake: "Erreur",
  Blunder: "Gaffe",
};

export function percent(p: number): string {
  return `${Math.round(p * 100)} %`;
}

/** A loss as percentage points, with a true minus sign. */
export function lossText(loss: number): string {
  const points = Math.round(loss * 100);
  if (points === 0) return "aucune perte";
  return `−${points} ${points === 1 ? "point" : "points"}`;
}

/**
 * The engine's numbers for every position (`onProgress(done, total)` after
 * each), then its grades. `curve` is Red's win probability after each ply.
 */
export async function computeReview(
  client: CoachLike,
  moves: TwixtAction[],
  onProgress: (done: number, total: number) => void,
): Promise<{ curve: number[]; review: Review }> {
  const total = moves.length + 1;
  const points: ReviewPoint[] = [];
  for (let ply = 0; ply < total; ply++) {
    points.push(await client.reviewPoint<ReviewPoint>(moves, ply));
    onProgress(ply + 1, total);
  }
  const review = await client.review<Review>(moves, points);
  return { curve: points.map((p) => p.redWin), review };
}
