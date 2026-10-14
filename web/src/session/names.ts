import type { Level } from "../engine/types";

export const LEVEL_LABELS: Record<Level, string> = { Easy: "facile", Medium: "moyenne", Hard: "difficile" };

/** The level, as offered in the lobby menus. */
export function aiName(level: Level): string {
  return `IA ${LEVEL_LABELS[level]}`;
}

/** One first name per seat, so the AIs at a table can be told apart. */
const AI_FIRST_NAMES = ["Margot", "Hugo", "Lina", "Oscar", "Zoé", "Félix", "Inès", "Jules"];

/** An AI's name at the table: "Hugo (IA)" in seat 1. */
export function aiSeatName(seat: number): string {
  return `${AI_FIRST_NAMES[seat % AI_FIRST_NAMES.length]} (IA)`;
}

export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
