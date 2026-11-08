// Mirrors games/enemy-chocolatier (View, Action) and board.json (written by
// scripts/chocolatier-board.py).

export type Ingredient = "Chocolate" | "Sugar" | "Nuts" | "Love";
export type Ability =
  | "BuyThree"
  | "BuyClosed"
  | "BuyAnywhere"
  | "AllDollars"
  | "AllPoints"
  | "FiveDollarsPoint"
  | "TenDollarHouses";

export type ChocolatierAction = { Buy: number } | "EndTurn";

export type HoodStatus = { kind: "Open" } | { kind: "Closed"; by: number } | { kind: "Unavailable" };

export interface HoodView {
  section: number;
  index: number;
  color: Ingredient | null;
  dollars: number;
  points: number;
  ability: Ability | null;
  status: HoodStatus;
  houses: number[];
}

export interface HouseView {
  hood: number;
  price: number;
  owner: number | null;
}

export interface Offer {
  house: number;
  cost: number;
  rainCheck: boolean;
}

export interface RecipeView {
  name: string;
  /** Chocolate, sugar, nuts, love. */
  need: number[];
}

export interface SeatView {
  money: number;
  points: number;
  rainChecks: number;
  incomeDollars: number;
  incomePoints: number;
  abilities: Ability[];
  /** Chocolate, sugar, nuts, love. */
  ingredients: number[];
  recipe: RecipeView | null;
}

export type ChocolatierEvent =
  | { Income: { dollars: number; points: number } }
  | { Bought: { house: number; from: number | null; cost: number; rainCheck: boolean } }
  | { Closed: { hood: number } }
  | "RainCheck";

export type End = { Points: number } | { Recipe: number } | "RoundLimit";

export interface ChocolatierView {
  me: number | null;
  players: number;
  width: number;
  height: number;
  slots: [number, number][];
  factoryAt: [number, number];
  layout: { section: number; quarters: number }[];
  hoods: HoodView[];
  houses: HouseView[];
  seats: SeatView[];
  turn: number;
  bought: number;
  limit: number;
  offers: Offer[];
  events: ChocolatierEvent[];
  lastTurn: { seat: number; events: ChocolatierEvent[] } | null;
  targetPoints: number;
  roundsLeft: number;
  end: { end: End; ranks: number[] } | null;
}

/** One section as drawn, in its own unturned frame (825 x 550). */
export interface BoardJson {
  width: number;
  height: number;
  factory: [number, number][];
  sections: {
    hoods: {
      color: Ingredient | "Gray";
      shape: [number, number][];
      /** x, y, width, height. */
      houses: [number, number, number, number][];
      symbols: { kind: "dollars" | "points" | "ability"; value: number | Ability; at: [number, number] }[];
    }[];
  }[];
}
