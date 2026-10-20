// Mirrors games/pairs/src/the_judge (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export interface TheJudgeReveal {
  card: Rank;
  eliminated: number[];
}

export type TheJudgeEvent =
  | { type: "Dealt"; round: number; leader: number }
  | { type: "Added"; seat: number; card: Rank }
  /** `forced`: no card of the hand could be added. */
  | { type: "Passed"; seat: number; forced: boolean }
  | { type: "Contest"; reveals: TheJudgeReveal[]; survivors: number[]; gained: number[]; scores: number[] };

export interface TheJudgeSeatView {
  recipe: Rank[];
  /** What the recipe would score. */
  recipePoints: number;
  handSize: number;
  passed: boolean;
  score: number;
}

export interface TheJudgeView {
  roundNo: number;
  target: number;
  phase: "Cook" | "Over";
  seats: TheJudgeSeatView[];
  hand: Rank[];
  /** The hand cards I may add now. */
  addable: Rank[];
  judgeSize: number;
  leader: number;
  turn: number | null;
  events: TheJudgeEvent[];
}

export type TheJudgeAction = { Add: { card: Rank } } | "Pass";
