import type { Seat, Update } from "../engine/types";
import { errorMessage } from "./names";

export type SeatKind = "local" | "ai" | "remote" | "open";

export interface SeatInfo {
  name: string;
  kind: SeatKind;
  connected: boolean;
}

export interface SessionState {
  phase: "connecting" | "lobby" | "playing";
  gameId: string | null;
  /** What `me` sees, or null before the game starts. */
  update: Update | null;
  /** The seat this screen plays (the pending one in hot-seat). */
  me: Seat | null;
  seats: SeatInfo[];
  /** Connection state to show. */
  status: string | null;
  /** Non-fatal message (action refused…). */
  notice: string | null;
  /** Fatal error: the session cannot continue. */
  error: string | null;
  /** Room code to share (host) or joined (guest). */
  shareCode: string | null;
  latencyMs: number | null;
}

const INITIAL: SessionState = {
  phase: "connecting",
  gameId: null,
  update: null,
  me: null,
  seats: [],
  status: null,
  notice: null,
  error: null,
  shareCode: null,
  latencyMs: null,
};

/**
 * A game as seen by the page: a state to display, actions to send. Where the
 * engine runs (here or on a host) is the implementation's business.
 */
export abstract class Session {
  abstract readonly canRestart: boolean;

  private state: SessionState = INITIAL;
  private listeners = new Set<() => void>();

  abstract submit(seat: Seat, action: unknown): void;
  restart(): void {}
  dispose(): void {}

  getState = (): SessionState => this.state;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  protected set(patch: Partial<SessionState>) {
    this.state = { ...this.state, ...patch };
    this.listeners.forEach((l) => l());
  }

  protected fail(e: unknown) {
    this.set({ error: errorMessage(e) });
  }
}
