import type { Level, Seat } from "../engine/types";
import { aiSeatName } from "./names";
import type { SeatInfo } from "./session";
import type { TableSeat } from "./table";

export type HostSeat =
  | { kind: "host"; name: string }
  | { kind: "ai"; level: Level }
  | { kind: "open" }
  | { kind: "guest"; name: string; token: string; connected: boolean };

/** What the host may put in a seat no guest holds. */
export type HostChoice = { kind: "ai"; level: Level } | { kind: "open" };

export type JoinResult =
  | { ok: true; seat: Seat; token: string; reconnected: boolean }
  | { ok: false; reason: string };

function cleanName(name: string): string {
  return name.trim().slice(0, 20) || "Invité";
}

/** The host's seats: who sits where, and each guest's reconnection token. */
export class SeatRegistry {
  constructor(private seats: HostSeat[]) {}

  /** After a host reload: every guest is away until they reconnect. */
  static fromJSON(seats: HostSeat[]): SeatRegistry {
    return new SeatRegistry(seats.map((s) => (s.kind === "guest" ? { ...s, connected: false } : s)));
  }

  toJSON(): HostSeat[] {
    return this.seats.map((s) => ({ ...s }));
  }

  get all(): readonly HostSeat[] {
    return this.seats;
  }

  /**
   * A guest says hello. A known token takes its seat back (even if an old
   * connection still holds it: the caller closes that one). Otherwise the
   * first open seat, before the game starts.
   */
  join(name: string, token: string | undefined, started: boolean, newToken: () => string): JoinResult {
    const back = token === undefined ? -1 : this.seats.findIndex((s) => s.kind === "guest" && s.token === token);
    if (back >= 0) {
      const seat = this.seats[back] as Extract<HostSeat, { kind: "guest" }>;
      this.seats[back] = { ...seat, connected: true };
      return { ok: true, seat: back, token: seat.token, reconnected: true };
    }
    if (started) return { ok: false, reason: "La partie a déjà commencé." };
    const open = this.seats.findIndex((s) => s.kind === "open");
    if (open < 0) return { ok: false, reason: "La table est complète." };
    const fresh = newToken();
    this.seats[open] = { kind: "guest", name: cleanName(name), token: fresh, connected: true };
    return { ok: true, seat: open, token: fresh, reconnected: false };
  }

  /** In the lobby the seat opens again; during the game it waits for the guest. */
  leave(seat: Seat, started: boolean) {
    const s = this.seats[seat];
    if (s?.kind !== "guest") return;
    this.seats[seat] = started ? { ...s, connected: false } : { kind: "open" };
  }

  /** Lobby only: returns false for the host's seat or a guest's seat. */
  setSeat(seat: Seat, choice: HostChoice): boolean {
    const s = this.seats[seat];
    if (!s || s.kind === "host" || s.kind === "guest") return false;
    this.seats[seat] = choice;
    return true;
  }

  ready(): boolean {
    return this.seats.every((s) => s.kind !== "open" && (s.kind !== "guest" || s.connected));
  }

  info(): SeatInfo[] {
    return this.seats.map((s, i): SeatInfo => {
      switch (s.kind) {
        case "host":
          return { name: s.name, kind: "local", connected: true };
        case "ai":
          return { name: aiSeatName(i), kind: "ai", connected: true };
        case "open":
          return { name: "Place libre", kind: "open", connected: false };
        case "guest":
          return { name: s.name, kind: "remote", connected: s.connected };
      }
    });
  }

  /** The table to start: throws if a seat is still open. */
  tableSeats(): TableSeat[] {
    return this.seats.map((s, i): TableSeat => {
      switch (s.kind) {
        case "host":
          return { kind: "local", name: s.name };
        case "ai":
          return { kind: "ai", level: s.level, name: aiSeatName(i) };
        case "guest":
          return { kind: "remote", name: s.name };
        case "open":
          throw new Error("Un siège est encore libre.");
      }
    });
  }
}
