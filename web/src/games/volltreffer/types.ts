// Mirrors games/volltreffer (View, Action and their parts).

export type Tribe = "Red" | "Green" | "Blue" | "Yellow" | "Purple";

export interface Card {
  tribe: Tribe;
  value: number;
}

export type Phase = "Hire" | "Publish" | "Tricks" | "Over";

export type VolltrefferAction =
  | { Hire: { card: Card } }
  | { Publish: { groups: number[]; tribes: boolean } }
  | { Play: { card: Card } };

export interface Place {
  card: Card;
  cost: number;
  hireable: boolean;
}

export interface RoundLine {
  spent: number;
  melds: number;
  preemptive: boolean;
  tricks: number;
  /** The score at the end of the round. */
  score: number;
}

export interface Publication {
  groups: number[];
  tribes: boolean;
  cards: Card[];
  points: number;
}

export interface HandInfo {
  score: number;
  cards: number;
  line: RoundLine;
  /** Shown in phase 2, until the tricks start. */
  publication: Publication | null;
  published: boolean;
}

export interface HireEvent {
  seat: number;
  card: Card;
  cost: number;
}

export interface TakenTrick {
  plays: [number, Card][];
  winner: number;
}

export interface VolltrefferView {
  me: number | null;
  hand: Card[];
  seats: HandInfo[];
  phase: Phase;
  /** From 1. */
  round: number;
  blessing: Card;
  trump: Tribe;
  /** Cheapest place first. */
  display: Place[];
  deck: number;
  bargaining: number;
  current: number | null;
  recentHires: HireEvent[];
  preemptive: number | null;
  trick: [number, Card][];
  lastTrick: TakenTrick | null;
  lastRound: RoundLine[] | null;
  sanctuary: [number, number];
  ranks: number[] | null;
  /** The points of each legal publication, in `legal` order. */
  publicationPoints: number[];
  legal: VolltrefferAction[];
}
