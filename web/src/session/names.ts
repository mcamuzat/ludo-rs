import type { Level } from "../engine/types";
import { t } from "../i18n";
import type { SeatInfo } from "./session";

/** The level, as offered in the lobby menus. */
export function aiName(level: Level): string {
  return t().aiLevels[level];
}

/** One first name per seat, so the AIs at a table can be told apart. */
const AI_FIRST_NAMES = ["Margot", "Hugo", "Lina", "Oscar", "Zoé", "Félix", "Inès", "Jules"];

/** An AI's name at the table: "Hugo (IA)" ("Hugo (AI)") in seat 1. */
export function aiSeatName(seat: number): string {
  return t().seats.ai(AI_FIRST_NAMES[seat % AI_FIRST_NAMES.length]);
}

/** A seat's name as shown here: AIs and open seats are named in the viewer's language. */
export function seatName(info: SeatInfo, seat: number): string {
  switch (info.kind) {
    case "ai":
      return aiSeatName(seat);
    case "open":
      return t().seats.open;
    default:
      return info.name;
  }
}

/** "1er", "2ᵉ" / "1st", "2nd". */
export function rankLabel(rank: number): string {
  return t().seats.rank(rank);
}

export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
