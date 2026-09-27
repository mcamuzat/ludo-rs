import type { Seat, SeatClockView } from "../engine/types";

export interface ClockSettings {
  /** Time for each decision. */
  baseMs: number;
  /** Extra time for the whole game, spent once the base time is used up. */
  bankMs: number;
}

export const CLOCK_PRESETS: { id: string; label: string; settings: ClockSettings | null }[] = [
  { id: "5+20", label: "5 + 20 s", settings: { baseMs: 5_000, bankMs: 20_000 } },
  { id: "10+40", label: "10 + 40 s", settings: { baseMs: 10_000, bankMs: 40_000 } },
  { id: "none", label: "Aucun", settings: null },
];

/** Per-seat decision clocks (Mahjong Soul style). Pure: time is passed in. */
export class Clocks {
  private banks = new Map<Seat, number>();
  private started = new Map<Seat, number>();

  constructor(
    readonly settings: ClockSettings,
    banks: [Seat, number][] = [],
  ) {
    for (const [seat, bank] of banks) this.banks.set(seat, bank);
  }

  private bank(seat: Seat): number {
    return this.banks.get(seat) ?? this.settings.bankMs;
  }

  private spend(seat: Seat, used: number) {
    const over = Math.max(0, used - this.settings.baseMs);
    this.banks.set(seat, Math.max(0, this.bank(seat) - over));
  }

  /** `seat` has just decided: its time stops (a new decision starts at the next `sync`). */
  finish(seat: Seat, now: number) {
    const since = this.started.get(seat);
    if (since === undefined) return;
    this.spend(seat, now - since);
    this.started.delete(seat);
  }

  /** `pending`: the timed seats that must decide now. */
  sync(pending: Seat[], now: number) {
    for (const seat of [...this.started.keys()]) {
      if (!pending.includes(seat)) this.finish(seat, now);
    }
    for (const seat of pending) {
      if (!this.started.has(seat)) this.started.set(seat, now);
    }
  }

  view(seat: Seat, now: number): SeatClockView | null {
    const since = this.started.get(seat);
    if (since === undefined) return null;
    const used = now - since;
    const { baseMs } = this.settings;
    return {
      seat,
      baseMs: Math.max(0, baseMs - used),
      bankMs: Math.max(0, this.bank(seat) - Math.max(0, used - baseMs)),
    };
  }

  views(now: number): SeatClockView[] {
    return [...this.started.keys()]
      .sort((a, b) => a - b)
      .map((seat) => this.view(seat, now))
      .filter((v): v is SeatClockView => v !== null);
  }

  /** When the default action is due, or null if `seat` is not deciding. */
  deadline(seat: Seat): number | null {
    const since = this.started.get(seat);
    return since === undefined ? null : since + this.settings.baseMs + this.bank(seat);
  }

  toJSON(): [Seat, number][] {
    return [...this.banks.entries()].sort(([a], [b]) => a - b);
  }
}

/** Time left `elapsed` ms after the clock view was received (never negative). */
export function clockLabel(c: SeatClockView, elapsed: number): string {
  const spent = Math.max(0, elapsed);
  const base = Math.max(0, c.baseMs - spent);
  const bank = Math.max(0, c.bankMs - Math.max(0, spent - c.baseMs));
  return `${Math.ceil(base / 1000)} s + ${Math.ceil(bank / 1000)} s`;
}
