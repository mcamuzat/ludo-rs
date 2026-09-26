type Handler = (...args: unknown[]) => void;

export class Emitter {
  private handlers = new Map<string, Handler[]>();
  on(event: string, handler: Handler) {
    this.handlers.set(event, [...(this.handlers.get(event) ?? []), handler]);
  }
  off(event: string) {
    this.handlers.delete(event);
  }
  removeAllListeners() {
    this.handlers.clear();
  }
  emit(event: string, ...args: unknown[]) {
    for (const h of this.handlers.get(event) ?? []) h(...args);
  }
}

export class FakeConn extends Emitter {
  open = false;
  sent: unknown[] = [];
  send(msg: unknown) {
    this.sent.push(msg);
  }
  close() {
    this.open = false;
    this.emit("close");
  }
}

export const peers: FakePeer[] = [];

export class FakePeer extends Emitter {
  destroyed = false;
  conns: FakeConn[] = [];
  constructor(readonly id?: string) {
    super();
    peers.push(this);
  }
  connect() {
    const conn = new FakeConn();
    this.conns.push(conn);
    return conn;
  }
  reconnect() {}
  /** Like PeerJS 1.5: destroying closes every connection, which emits "close" synchronously. */
  destroy() {
    this.destroyed = true;
    for (const conn of this.conns) conn.close();
  }
}
