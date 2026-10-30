import { CollusionBoard } from "./cards/collusion/Board";
import { COLLUSION_RULES } from "./cards/collusion/rules";
import { DavidGoliathBoard } from "./cards/david-goliath/Board";
import { DAVID_GOLIATH_RULES } from "./cards/david-goliath/rules";
import { HattrickBoard } from "./cards/hattrick/Board";
import { HATTRICK_RULES } from "./cards/hattrick/rules";
import { LamarckianBoard } from "./cards/lamarckian/Board";
import { LAMARCKIAN_RULES } from "./cards/lamarckian/rules";
import { BantuBoard } from "./bantu/Board";
import { BANTU_RULES } from "./bantu/rules";
import { BlackPotBoard } from "./black-pot/Board";
import { BLACK_POT_RULES } from "./black-pot/rules";
import { ManoeverBoard } from "./manoever/Board";
import { MANOEVER_RULES } from "./manoever/rules";
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
import { HornOfPlentyBoard } from "./pairs/horn-of-plenty/Board";
import { KittyBoard } from "./pairs/kitty/Board";
import { MonsterBoard } from "./pairs/monster/Board";
import { OctopusBoard } from "./pairs/octopus/Board";
import { PendletonBoard } from "./pairs/pendleton/Board";
import { PortBoard } from "./pairs/port/Board";
import { RocketBoard } from "./pairs/rocket/Board";
import { StarboardBoard } from "./pairs/starboard/Board";
import { SweepBoard } from "./pairs/sweep/Board";
import { TheJudgeBoard } from "./pairs/the-judge/Board";
import { BreadBasketBoard } from "./pairs/bread-basket/Board";
import { TheCaveBoard } from "./pairs/the-cave/Board";
import { PickupBoard } from "./pairs/pickup/Board";
import { MarketDayBoard } from "./pairs/market-day/Board";
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
  HORN_OF_PLENTY_RULES,
  KITTY_RULES,
  MONSTER_RULES,
  OCTOPUS_RULES,
  PENDLETON_RULES,
  PORT_RULES,
  ROCKET_RULES,
  STARBOARD_RULES,
  SWEEP_RULES,
  THE_JUDGE_RULES,
  BREAD_BASKET_RULES,
  THE_CAVE_RULES,
  PICKUP_RULES,
  MARKET_DAY_RULES,
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
import { BlackViennaBoard } from "./black-vienna/Board";
import { BLACK_VIENNA_RULES } from "./black-vienna/rules";
import { CryptideBoard } from "./cryptide/Board";
import { CRYPTIDE_RULES } from "./cryptide/rules";
import { VolltrefferBoard } from "./volltreffer/Board";
import { VOLLTREFFER_RULES } from "./volltreffer/rules";
import { SiberianDiceBoard } from "./siberian-dice/Board";
import { SIBERIAN_DICE_RULES } from "./siberian-dice/rules";
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
export type GameFamily = "board" | "tiles" | "pairs" | "cards" | "modern" | "coins" | "trade";

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
  { id: "black-pot", family: "board", Board: BlackPotBoard, rules: BLACK_POT_RULES },
  { id: "manoever", family: "board", Board: ManoeverBoard, rules: MANOEVER_RULES },
  { id: "siberian-dice", family: "board", Board: SiberianDiceBoard, rules: SIBERIAN_DICE_RULES },
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
  { id: "pairs-horn-of-plenty", family: "pairs", Board: HornOfPlentyBoard, rules: HORN_OF_PLENTY_RULES },
  { id: "pairs-the-judge", family: "pairs", Board: TheJudgeBoard, rules: THE_JUDGE_RULES },
  { id: "pairs-bread-basket", family: "pairs", Board: BreadBasketBoard, rules: BREAD_BASKET_RULES },
  { id: "pairs-the-cave", family: "pairs", Board: TheCaveBoard, rules: THE_CAVE_RULES },
  { id: "pairs-pickup", family: "pairs", Board: PickupBoard, rules: PICKUP_RULES },
  { id: "pairs-market-day", family: "pairs", Board: MarketDayBoard, rules: MARKET_DAY_RULES },
  { id: "cards-president", family: "cards", Board: PresidentBoard, rules: PRESIDENT_RULES },
  { id: "cards-collusion", family: "cards", Board: CollusionBoard, rules: COLLUSION_RULES },
  { id: "pennywise", family: "coins", Board: PennywiseBoard, rules: PENNYWISE_RULES },
  { id: "monad", family: "trade", Board: MonadBoard, rules: MONAD_RULES },
  { id: "volltreffer", family: "modern", Board: VolltrefferBoard, rules: VOLLTREFFER_RULES },
  { id: "black-vienna", family: "modern", Board: BlackViennaBoard, rules: BLACK_VIENNA_RULES },
  { id: "cryptide", family: "modern", Board: CryptideBoard, rules: CRYPTIDE_RULES },
  { id: "cards-lamarckian", family: "modern", Board: LamarckianBoard, rules: LAMARCKIAN_RULES },
  { id: "cards-hattrick", family: "modern", Board: HattrickBoard, rules: HATTRICK_RULES },
  { id: "cards-david-goliath", family: "modern", Board: DavidGoliathBoard, rules: DAVID_GOLIATH_RULES },
];

export function gameUi(id: string): GameUi | undefined {
  return GAME_UIS.find((g) => g.id === id);
}
