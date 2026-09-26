// Mirrors games/riichi/src (View, Action). Tiles in mjai notation.

export type Tile = string;
export type Wind = "E" | "S" | "W" | "N";
export type MeldKind = "Chi" | "Pon" | "Daiminkan" | "Ankan" | "Kakan";

export interface DiscardView {
  tile: Tile;
  tsumogiri: boolean;
  riichi: boolean;
}

export interface MeldView {
  kind: MeldKind;
  tiles: Tile[];
  from: number | null;
}

export interface PlayerView {
  seatWind: Wind;
  score: number;
  riichi: boolean;
  handSize: number;
  discards: DiscardView[];
  melds: MeldView[];
}

export interface RoundView {
  wind: Wind;
  hand: number;
  honba: number;
  riichiSticks: number;
  wallLeft: number;
  doraIndicators: Tile[];
  dealer: number;
}

export interface Help {
  shanten: number;
  waits: Tile[];
}

export interface YakuName {
  name: string;
  nameJa: string;
}

export interface WinView {
  seat: number;
  from: number | null;
  hand: Tile[];
  melds: MeldView[];
  winTile: Tile;
  yaku: YakuName[];
  han: number;
  fu: number;
  points: number;
  yakuman: boolean;
}

export type DrawKind = "Exhaustive" | "NineTerminals" | "Abortive";

export interface HandResult {
  number: number;
  wins: WinView[];
  draw: DrawKind | null;
  deltas: number[];
  uraIndicators: Tile[];
}

export interface RiichiView {
  me: number | null;
  hand: Tile[];
  drawn: Tile | null;
  players: PlayerView[];
  round: RoundView;
  help: Help | null;
  answered: boolean;
  lastHand: HandResult | null;
}

export type RiichiAction =
  | { type: "Discard"; tile: Tile }
  | { type: "Riichi" }
  | { type: "Chi"; tile: Tile; with: [Tile, Tile] }
  | { type: "Pon"; tile: Tile; with: [Tile, Tile] }
  | { type: "Daiminkan"; tile: Tile }
  | { type: "Ankan"; tile: Tile }
  | { type: "Kakan"; tile: Tile }
  | { type: "Tsumo" }
  | { type: "Ron" }
  | { type: "KyushuKyuhai" }
  | { type: "Pass" };
