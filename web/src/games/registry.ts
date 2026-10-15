import type { ComponentType } from "react";
import type { Seat } from "../engine/types";
import { AubergeBoard } from "./pairs/auberge/Board";
import { CaptainsOrdersBoard } from "./pairs/captains-orders/Board";
import { ContinuousBoard } from "./pairs/continuous/Board";
import { DeadfallBoard } from "./pairs/deadfall/Board";
import { GhostTownBoard } from "./pairs/ghost-town/Board";
import { GoblinPokerBoard } from "./pairs/goblin-poker/Board";
import { KittyBoard } from "./pairs/kitty/Board";
import { PendletonBoard } from "./pairs/pendleton/Board";
import { ToTheMoonBoard } from "./pairs/to-the-moon/Board";
import {
  AUBERGE_RULES,
  CAPTAINS_ORDERS_RULES,
  CONTINUOUS_RULES,
  DEADFALL_RULES,
  GHOST_TOWN_RULES,
  GOBLIN_POKER_RULES,
  KITTY_RULES,
  PENDLETON_RULES,
  TO_THE_MOON_RULES,
  ZOMBIE_LIFE_RULES,
} from "./pairs/rules";
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
  /** Left off the home screen; a shared link still opens it. */
  hidden?: boolean;
  /** A short reminder shown at the table. */
  rules?: string[];
}

/** Adding a game: one entry here (and one in crates/ludo-registry). */
export const GAME_UIS: GameUi[] = [
  { id: "yavalath", family: "board", Board: YavalathBoard },
  { id: "riichi", family: "tiles", Board: RiichiBoard, hidden: true },
  { id: "pairs-continuous", family: "pairs", Board: ContinuousBoard, rules: CONTINUOUS_RULES },
  { id: "pairs-auberge", family: "pairs", Board: AubergeBoard, rules: AUBERGE_RULES },
  { id: "pairs-ghost-town", family: "pairs", Board: GhostTownBoard, rules: GHOST_TOWN_RULES },
  { id: "pairs-zombie-life", family: "pairs", Board: ZombieLifeBoard, rules: ZOMBIE_LIFE_RULES },
  { id: "pairs-goblin-poker", family: "pairs", Board: GoblinPokerBoard, rules: GOBLIN_POKER_RULES },
  { id: "pairs-to-the-moon", family: "pairs", Board: ToTheMoonBoard, rules: TO_THE_MOON_RULES },
  { id: "pairs-deadfall", family: "pairs", Board: DeadfallBoard, rules: DEADFALL_RULES },
  { id: "pairs-captains-orders", family: "pairs", Board: CaptainsOrdersBoard, rules: CAPTAINS_ORDERS_RULES },
  { id: "pairs-pendleton", family: "pairs", Board: PendletonBoard, rules: PENDLETON_RULES },
  { id: "pairs-kitty", family: "pairs", Board: KittyBoard, rules: KITTY_RULES },
];

export function gameUi(id: string): GameUi | undefined {
  return GAME_UIS.find((g) => g.id === id);
}
