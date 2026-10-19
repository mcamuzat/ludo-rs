import { CollusionBoard } from "./cards/collusion/Board";
import { COLLUSION_RULES } from "./cards/collusion/rules";
import { BantuBoard } from "./bantu/Board";
import { BANTU_RULES } from "./bantu/rules";
import { PresidentBoard } from "./cards/president/Board";
import { PRESIDENT_RULES } from "./cards/president/rules";
import type { ComponentType } from "react";
import type { Seat } from "../engine/types";
import { AubergeBoard } from "./pairs/auberge/Board";
import { BlackstoneBoard } from "./pairs/blackstone/Board";
import { CaptainsOrdersBoard } from "./pairs/captains-orders/Board";
import { ContinuousBoard } from "./pairs/continuous/Board";
import { DeadfallBoard } from "./pairs/deadfall/Board";
import { GhostTownBoard } from "./pairs/ghost-town/Board";
import { GoblinPokerBoard } from "./pairs/goblin-poker/Board";
import { KittyBoard } from "./pairs/kitty/Board";
import { MonsterBoard } from "./pairs/monster/Board";
import { OctopusBoard } from "./pairs/octopus/Board";
import { PendletonBoard } from "./pairs/pendleton/Board";
import { PortBoard } from "./pairs/port/Board";
import { RocketBoard } from "./pairs/rocket/Board";
import { StarboardBoard } from "./pairs/starboard/Board";
import { SweepBoard } from "./pairs/sweep/Board";
import { ToTheMoonBoard } from "./pairs/to-the-moon/Board";
import { TrollGutsBoard } from "./pairs/troll-guts/Board";
import {
  AUBERGE_RULES,
  BLACKSTONE_RULES,
  CAPTAINS_ORDERS_RULES,
  CONTINUOUS_RULES,
  DEADFALL_RULES,
  GHOST_TOWN_RULES,
  GOBLIN_POKER_RULES,
  KITTY_RULES,
  MONSTER_RULES,
  OCTOPUS_RULES,
  PENDLETON_RULES,
  PORT_RULES,
  ROCKET_RULES,
  STARBOARD_RULES,
  SWEEP_RULES,
  TO_THE_MOON_RULES,
  TROLL_GUTS_RULES,
  ZOMBIE_LIFE_RULES,
} from "./pairs/rules";
import { ZombieLifeBoard } from "./pairs/zombie-life/Board";
import { PennywiseBoard } from "./pennywise/Board";
import { PENNYWISE_RULES } from "./pennywise/rules";
import { MonadBoard } from "./monad/Board";
import { MONAD_RULES } from "./monad/rules";
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
export type GameFamily = "board" | "tiles" | "pairs" | "cards" | "coins" | "trade";

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
  { id: "bantu", family: "board", Board: BantuBoard, rules: BANTU_RULES },
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
  { id: "pairs-octopus", family: "pairs", Board: OctopusBoard, rules: OCTOPUS_RULES },
  { id: "pairs-port", family: "pairs", Board: PortBoard, rules: PORT_RULES },
  { id: "pairs-starboard", family: "pairs", Board: StarboardBoard, rules: STARBOARD_RULES },
  { id: "pairs-sweep", family: "pairs", Board: SweepBoard, rules: SWEEP_RULES },
  { id: "pairs-rocket", family: "pairs", Board: RocketBoard, rules: ROCKET_RULES },
  { id: "pairs-monster", family: "pairs", Board: MonsterBoard, rules: MONSTER_RULES },
  { id: "pairs-troll-guts", family: "pairs", Board: TrollGutsBoard, rules: TROLL_GUTS_RULES },
  { id: "pairs-blackstone", family: "pairs", Board: BlackstoneBoard, rules: BLACKSTONE_RULES },
  { id: "cards-president", family: "cards", Board: PresidentBoard, rules: PRESIDENT_RULES },
  { id: "cards-collusion", family: "cards", Board: CollusionBoard, rules: COLLUSION_RULES },
  { id: "pennywise", family: "coins", Board: PennywiseBoard, rules: PENNYWISE_RULES },
  { id: "monad", family: "trade", Board: MonadBoard, rules: MONAD_RULES },
];

export function gameUi(id: string): GameUi | undefined {
  return GAME_UIS.find((g) => g.id === id);
}
