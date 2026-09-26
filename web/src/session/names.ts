import type { Level } from "../engine/types";

export const LEVEL_LABELS: Record<Level, string> = { Easy: "facile", Medium: "moyenne", Hard: "difficile" };

export function aiName(level: Level): string {
  return `IA ${LEVEL_LABELS[level]}`;
}

export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
