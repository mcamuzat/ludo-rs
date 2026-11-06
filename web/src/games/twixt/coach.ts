// Display helpers for TwixT's coach. Every number comes from the engine
// (games/twixt/src/coach.rs); this file only sequences the calls and formats.

import type { Grade, Review, TwixtAction } from "./types";

/** What the board needs from `CoachClient` (a fake in tests). */
export interface CoachLike {
  advise<T>(moves: unknown[]): Promise<T>;
  redWinAt(moves: unknown[], ply: number): Promise<number>;
  review<T>(moves: unknown[], curve: number[]): Promise<T>;
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
  return `−${Math.round(loss * 100)} points`;
}

/**
 * Red's win probability after each ply (one engine evaluation per position,
 * `onProgress(done, total)` after each), then the engine's grades.
 */
export async function computeReview(
  client: CoachLike,
  moves: TwixtAction[],
  onProgress: (done: number, total: number) => void,
): Promise<{ curve: number[]; review: Review }> {
  const total = moves.length + 1;
  const curve: number[] = [];
  for (let ply = 0; ply < total; ply++) {
    curve.push(await client.redWinAt(moves, ply));
    onProgress(ply + 1, total);
  }
  const review = await client.review<Review>(moves, curve);
  return { curve, review };
}
