// Mirrors games/pennywise/src (View, Action).

export type Character = "Cody" | "Darlene" | "Sugar" | "Taylor";

export interface Stack {
  cents: number;
  count: number;
}

export interface LastMove {
  seat: number;
  coin: number;
  take: number[];
}

export interface PennywiseView {
  character: Character;
  purses: Stack[][];
  totals: number[];
  pot: Stack[];
  potTotal: number;
  current: number | null;
  last: LastMove | null;
  ruined: number | null;
  ranks: number[] | null;
}

export type PennywiseAction = { Play: { coin: number; take: number[] } };
