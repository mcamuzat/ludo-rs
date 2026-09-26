import Peer, { type DataConnection } from "peerjs";
import { randomSeed } from "../engine/clients";
import { updateFor, type Seat, type Snapshot } from "../engine/types";
import { errorMessage } from "./names";
import {
  PEER_PREFIX,
  PROTOCOL_VERSION,
  newCode,
  parseGuestMessage,
  type GuestMessage,
  type HostMessage,
} from "./protocol";
import { SeatRegistry, type HostChoice, type HostSeat } from "./seats";
import { Session } from "./session";
import { clearHost, saveHost, type SavedHost } from "./storage";
import { Table } from "./table";

const HOST_SEAT: Seat = 0;
const RETRY_MS = 3000;
const MAX_RETRIES = 10;

/**
 * Hosts a game: the engine runs here and is the authority. Each guest is a
 * PeerJS connection to this host only, and receives only its own seat's view.
 */
export class HostSession extends Session {
  readonly canRestart = true;
  private peer: Peer | null = null;
  /** Open connections and the seat each one holds (null: not seated yet). */
  private conns = new Map<DataConnection, Seat | null>();
  private registry: SeatRegistry;
  private table: Table | null = null;
  private code: string;
  private retries = 0;
  private disposed = false;

  constructor(
    private gameId: string,
    hostName: string,
    others: HostChoice[],
    saved?: SavedHost,
  ) {
    super();
    if (saved) {
      this.code = saved.code;
      this.registry = SeatRegistry.fromJSON(saved.seats);
      this.set({ phase: "playing", gameId, seats: this.registry.info() });
      this.table = this.newTable();
      this.table.resume(saved.log).catch((e) => this.fail(e));
    } else {
      this.code = newCode();
      this.registry = new SeatRegistry([{ kind: "host", name: hostName.trim() || "Hôte" }, ...others]);
      this.set({ phase: "lobby", gameId, seats: this.registry.info() });
    }
    this.set({ status: "Ouverture de la salle…" });
    this.listen();
  }

  static resume(saved: SavedHost): HostSession {
    return new HostSession(saved.gameId, "", [], saved);
  }

  hostSeats(): readonly HostSeat[] {
    return this.registry.all;
  }

  ready(): boolean {
    return this.table === null && this.registry.ready();
  }

  setSeat(seat: Seat, choice: HostChoice) {
    if (this.table || !this.registry.setSeat(seat, choice)) return;
    this.set({ seats: this.registry.info() });
    this.broadcastLobby();
  }

  start() {
    if (!this.ready()) return;
    this.table = this.newTable();
    this.set({ phase: "playing", status: null });
    this.broadcastLobby();
    this.table.start(randomSeed()).catch((e) => this.fail(e));
  }

  submit(seat: Seat, action: unknown) {
    this.table?.submit(seat, action).catch((e) => this.set({ notice: errorMessage(e) }));
  }

  restart() {
    if (!this.table) return;
    this.table.dispose();
    this.table = this.newTable();
    this.table.start(randomSeed()).catch((e) => this.fail(e));
  }

  dispose() {
    this.disposed = true;
    this.table?.dispose();
    this.peer?.destroy();
  }

  private newTable(): Table {
    return new Table(this.gameId, this.registry.tableSeats(), {
      onSnapshot: (s) => this.broadcast(s),
      onLog: (log, s) => this.persist(log, s),
      onError: (message) => this.set({ notice: message }),
    });
  }

  private listen() {
    const peer = new Peer(PEER_PREFIX + this.code);
    this.peer = peer;
    peer.on("open", () => {
      this.retries = 0;
      this.set({
        shareCode: this.code,
        status: this.table ? null : "Partage le code ou le lien avec tes invités.",
      });
    });
    peer.on("connection", (conn) => this.accept(conn));
    peer.on("error", (err) => {
      if (err.type === "unavailable-id") {
        peer.destroy();
        if (!this.table) {
          // Fresh room: just take another code.
          this.code = newCode();
          this.listen();
          return;
        }
        // Resumed room: the previous page may still hold the code for a while.
        if (++this.retries > MAX_RETRIES) {
          this.fail("Le code de la salle est encore pris. Réessaie dans une minute.");
          return;
        }
        this.set({ status: "Le code de la salle est encore réservé, nouvel essai…" });
        setTimeout(() => {
          if (!this.disposed) this.listen();
        }, RETRY_MS);
      } else if (err.type !== "peer-unavailable") {
        this.set({ notice: `Connexion : ${err.message}` });
      }
    });
    // Signaling server lost: established connections keep working.
    peer.on("disconnected", () => {
      if (!peer.destroyed) peer.reconnect();
    });
  }

  private accept(conn: DataConnection) {
    this.conns.set(conn, null);
    conn.on("data", (data) => this.onGuestMessage(conn, data));
    conn.on("close", () => this.onClose(conn));
    conn.on("error", () => this.onClose(conn));
  }

  private onClose(conn: DataConnection) {
    if (!this.conns.has(conn)) return;
    const seat = this.conns.get(conn) ?? null;
    this.conns.delete(conn);
    // Unseated, or replaced by a newer connection of the same guest.
    if (seat === null || [...this.conns.values()].includes(seat)) return;
    const name = this.registry.info()[seat].name;
    this.registry.leave(seat, this.table !== null);
    this.set({
      seats: this.registry.info(),
      status: this.table ? `${name} s'est déconnecté : on attend son retour.` : null,
    });
    this.broadcastLobby();
  }

  private onGuestMessage(conn: DataConnection, data: unknown) {
    const msg = parseGuestMessage(data);
    if (!msg) {
      this.send(conn, { type: "rejected", reason: "Message invalide.", fatal: false });
      return;
    }
    switch (msg.type) {
      case "hello":
        this.onHello(conn, msg);
        break;
      case "ping":
        this.send(conn, { type: "pong", t: msg.t });
        break;
      case "action": {
        const seat = this.conns.get(conn) ?? null;
        if (seat === null || !this.table) {
          this.send(conn, { type: "rejected", reason: "Tu n'as pas de siège dans cette partie.", fatal: false });
          return;
        }
        // The engine checks it is this seat's decision and that it is legal.
        this.table.submit(seat, msg.action, msg.rev).catch((e) => {
          this.send(conn, { type: "rejected", reason: errorMessage(e), fatal: false });
          this.sendUpdate(conn, seat);
        });
        break;
      }
    }
  }

  private onHello(conn: DataConnection, msg: Extract<GuestMessage, { type: "hello" }>) {
    if (msg.protocol !== PROTOCOL_VERSION) {
      this.refuse(conn, `Versions différentes (hôte ${PROTOCOL_VERSION}, toi ${msg.protocol}) : recharge la page.`);
      return;
    }
    const joined = this.registry.join(msg.name, msg.seatToken, this.table !== null, () => crypto.randomUUID());
    if (!joined.ok) {
      this.refuse(conn, joined.reason);
      return;
    }
    // A reconnection replaces the seat's previous connection.
    for (const [other, seat] of this.conns) {
      if (other !== conn && seat === joined.seat) {
        this.conns.delete(other);
        other.close();
      }
    }
    this.conns.set(conn, joined.seat);
    this.send(conn, { type: "seated", seat: joined.seat, seatToken: joined.token });
    this.set({ seats: this.registry.info(), status: this.table ? null : this.getState().status });
    this.broadcastLobby();
    this.sendUpdate(conn, joined.seat);
  }

  private refuse(conn: DataConnection, reason: string) {
    this.send(conn, { type: "rejected", reason, fatal: true });
    setTimeout(() => conn.close(), 500);
  }

  /** Each player gets their own seat's update, and only that. */
  private broadcast(s: Snapshot) {
    this.set({ me: HOST_SEAT, update: updateFor(s, HOST_SEAT), notice: null });
    for (const [conn, seat] of this.conns) {
      if (seat !== null) this.send(conn, { type: "update", update: updateFor(s, seat) });
    }
  }

  private sendUpdate(conn: DataConnection, seat: Seat) {
    const s = this.table?.snapshot;
    if (s) this.send(conn, { type: "update", update: updateFor(s, seat) });
  }

  private broadcastLobby() {
    const msg: HostMessage = {
      type: "lobby",
      gameId: this.gameId,
      seats: this.registry.info(),
      started: this.table !== null,
    };
    for (const conn of this.conns.keys()) this.send(conn, msg);
  }

  /** Saved after every action so the host can resume after a reload; cleared once the game is over. */
  private persist(log: unknown, s: Snapshot) {
    if (s.outcome) clearHost();
    else saveHost({ code: this.code, gameId: this.gameId, seats: this.registry.toJSON(), log });
  }

  private send(conn: DataConnection, msg: HostMessage) {
    if (conn.open) void conn.send(msg);
  }
}
