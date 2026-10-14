import type { ComponentType } from "react";
import type { Seat } from "../engine/types";
import { AubergeBoard } from "./pairs/auberge/Board";
import { ContinuousBoard } from "./pairs/continuous/Board";
import { DeadfallBoard } from "./pairs/deadfall/Board";
import { GhostTownBoard } from "./pairs/ghost-town/Board";
import { GoblinPokerBoard } from "./pairs/goblin-poker/Board";
import { ToTheMoonBoard } from "./pairs/to-the-moon/Board";
import { ZombieLifeBoard } from "./pairs/zombie-life/Board";
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

/** How the home screen groups and illustrates a game. */
export type GameFamily = "board" | "tiles" | "pairs";

export interface GameUi {
  id: string;
  family: GameFamily;
  Board: ComponentType<BoardProps>;
}

/** Adding a game: one entry here (and one in crates/ludo-registry). */
export const GAME_UIS: GameUi[] = [
  { id: "yavalath", family: "board", Board: YavalathBoard },
  { id: "riichi", family: "tiles", Board: RiichiBoard },
  { id: "pairs-continuous", family: "pairs", Board: ContinuousBoard },
  { id: "pairs-auberge", family: "pairs", Board: AubergeBoard },
  { id: "pairs-ghost-town", family: "pairs", Board: GhostTownBoard },
  { id: "pairs-zombie-life", family: "pairs", Board: ZombieLifeBoard },
  { id: "pairs-goblin-poker", family: "pairs", Board: GoblinPokerBoard },
  { id: "pairs-to-the-moon", family: "pairs", Board: ToTheMoonBoard },
  { id: "pairs-deadfall", family: "pairs", Board: DeadfallBoard },
];

export function gameUi(id: string): GameUi | undefined {
  return GAME_UIS.find((g) => g.id === id);
}
