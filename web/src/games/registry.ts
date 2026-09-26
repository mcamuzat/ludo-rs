import type { ComponentType } from "react";
import type { Seat } from "../engine/types";
import { RiichiBoard } from "./riichi/Board";
import { YavalathBoard } from "./yavalath/Board";

/** What every game board receives. It displays and submits; it computes nothing. */
export interface BoardProps {
  view: unknown;
  legalActions: unknown[];
  pending: Seat[];
  me: Seat | null;
  seatNames: string[];
  submit(action: unknown): void;
}

export interface GameUi {
  id: string;
  Board: ComponentType<BoardProps>;
}

/** Adding a game: one entry here (and one in crates/ludo-registry). */
export const GAME_UIS: GameUi[] = [
  { id: "yavalath", Board: YavalathBoard },
  { id: "riichi", Board: RiichiBoard },
];

export function gameUi(id: string): GameUi | undefined {
  return GAME_UIS.find((g) => g.id === id);
}
