import { randomSeed } from "../engine/clients";
import type { GameConfig, Seat, Snapshot } from "../engine/types";
import { errorMessage } from "./names";
import { Session } from "./session";
import { Table, type TableSeat } from "./table";

/** Everything on this device: local humans (hot-seat) and AIs. */
export class LocalSession extends Session {
  readonly canRestart = true;
  private table: Table;

  constructor(
    private gameId: string,
    private seats: TableSeat[],
    private config: GameConfig | null = null,
  ) {
    super();
    this.set({
      phase: "playing",
      gameId,
      config,
      seats: seats.map((s) => ({ name: s.name, kind: s.kind, connected: true })),
    });
    this.table = this.begin();
  }

  private begin(): Table {
    const table = new Table(this.gameId, this.seats, {
      onSnapshot: (s) => this.show(table, s),
      onError: (message) => this.set({ notice: message }),
    });
    table.start(randomSeed(), this.config).catch((e) => this.fail(e));
    return table;
  }

  private show(table: Table, s: Snapshot) {
    const locals = this.seats.flatMap((seat, i) => (seat.kind === "local" ? [i] : []));
    // Hot-seat: show the local seat whose decision it is.
    const me = locals.find((i) => s.pending.includes(i)) ?? this.getState().me ?? locals[0] ?? null;
    this.set({ me, update: table.update(me), notice: null });
  }

  submit(seat: Seat, action: unknown) {
    this.table.submit(seat, action).catch((e) => this.set({ notice: errorMessage(e) }));
  }

  restart() {
    this.table.dispose();
    this.set({ update: null, notice: null });
    this.table = this.begin();
  }

  dispose() {
    this.table.dispose();
  }
}
