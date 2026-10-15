import Peer, { type DataConnection } from "peerjs";
import type { Seat } from "../engine/types";
import { gameUi } from "../games/registry";
import { PEER_PREFIX, PROTOCOL_VERSION, normalizeCode, parseHostMessage, type GuestMessage } from "./protocol";
import { Session } from "./session";
import { loadGuestToken, saveGuestToken } from "./storage";

const RETRY_MS = 3000;
const PING_MS = 5000;

/** Joins a host: sends actions, displays the updates of its own seat. */
export class GuestSession extends Session {
  readonly canRestart = false;
  private peer: Peer | null = null;
  private conn: DataConnection | null = null;
  private code: string;
  private closed = false;
  private retry: ReturnType<typeof setTimeout> | null = null;
  private ping: ReturnType<typeof setInterval>;

  constructor(
    code: string,
    private name: string,
  ) {
    super();
    this.code = normalizeCode(code);
    this.set({ phase: "connecting", shareCode: this.code, status: "Connexion à l'hôte…" });
    this.connect();
    this.ping = setInterval(() => this.send({ type: "ping", t: performance.now() }), PING_MS);
  }

  submit(_seat: Seat, action: unknown) {
    const rev = this.getState().update?.rev;
    if (rev !== undefined) this.send({ type: "action", rev, action });
  }

  dispose() {
    this.closed = true;
    clearInterval(this.ping);
    if (this.retry) clearTimeout(this.retry);
    this.peer?.destroy();
  }

  private connect() {
    // Forget the old peer first: destroying it closes its connection, whose
    // handlers must not schedule yet another reconnection.
    const old = this.peer;
    this.peer = null;
    this.conn = null;
    old?.destroy();
    const peer = new Peer();
    this.peer = peer;
    peer.on("open", () => {
      if (this.peer !== peer) return;
      const conn = peer.connect(PEER_PREFIX + this.code, { reliable: true });
      this.conn = conn;
      conn.on("open", () => {
        if (this.conn !== conn) return;
        const seatToken = loadGuestToken(this.code);
        this.send({ type: "hello", protocol: PROTOCOL_VERSION, name: this.name, ...(seatToken ? { seatToken } : {}) });
      });
      conn.on("data", (data) => {
        if (this.conn === conn) this.onHostMessage(data);
      });
      conn.on("close", () => {
        if (this.conn === conn) this.lost();
      });
    });
    peer.on("error", (err) => {
      if (this.peer !== peer) return;
      if (err.type === "peer-unavailable") {
        this.set({ status: "Hôte introuvable : vérifie le code, ou attends son retour…" });
      }
      // Only the signaling server is gone: the connection to the host keeps working.
      if (this.conn?.open) return;
      this.lost();
    });
    peer.on("disconnected", () => {
      if (this.peer === peer && !peer.destroyed) peer.reconnect();
    });
  }

  /** Keeps trying until the host is back (it may be reloading). */
  private lost() {
    if (this.closed || this.retry) return;
    if (this.getState().phase !== "connecting") {
      this.set({ status: "Connexion perdue, reconnexion…", latencyMs: null });
    }
    this.retry = setTimeout(() => {
      this.retry = null;
      if (!this.closed) this.connect();
    }, RETRY_MS);
  }

  private onHostMessage(data: unknown) {
    const msg = parseHostMessage(data);
    if (!msg) return;
    switch (msg.type) {
      case "lobby":
        if (!gameUi(msg.gameId)) {
          this.stop("Jeu inconnu : mets la page à jour.");
          return;
        }
        this.set({
          gameId: msg.gameId,
          seats: msg.seats,
          config: msg.config ?? null,
          phase: msg.started ? "playing" : "lobby",
          status: msg.started ? null : "En attente du lancement par l'hôte.",
        });
        break;
      case "seated":
        saveGuestToken(this.code, msg.seatToken);
        this.set({ me: msg.seat });
        break;
      case "update":
        this.set({ phase: "playing", update: msg.update, status: null, notice: null });
        break;
      case "rejected":
        if (msg.fatal) this.stop(msg.reason);
        else this.set({ notice: msg.reason });
        break;
      case "pong":
        this.set({ latencyMs: Math.round(performance.now() - msg.t) });
        break;
    }
  }

  private stop(reason: string) {
    this.closed = true;
    this.fail(reason);
    this.peer?.destroy();
  }

  private send(msg: GuestMessage) {
    if (this.conn?.open) void this.conn.send(msg);
  }
}
