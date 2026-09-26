export type RpcResponse = { id: number; ok: true; value: unknown } | { id: number; ok: false; error: string };

/** Promise-based calls to a worker; requests are answered in order. */
export class WorkerRpc<Req> {
  private nextId = 0;
  private waiting = new Map<number, { resolve: (value: unknown) => void; reject: (error: Error) => void }>();

  constructor(private worker: Worker) {
    worker.onmessage = (event: MessageEvent<RpcResponse>) => {
      const res = event.data;
      const waiter = this.waiting.get(res.id);
      if (!waiter) return;
      this.waiting.delete(res.id);
      if (res.ok) waiter.resolve(res.value);
      else waiter.reject(new Error(res.error));
    };
  }

  call<T>(req: Req): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.waiting.set(id, { resolve: (value) => resolve(value as T), reject });
      this.worker.postMessage({ id, req });
    });
  }

  dispose() {
    this.worker.terminate();
    for (const waiter of this.waiting.values()) waiter.reject(new Error("Moteur arrêté."));
    this.waiting.clear();
  }
}

/** Worker side: answers each request with `handle`'s result. */
export function serve<Req>(ready: Promise<unknown>, handle: (req: Req) => unknown) {
  self.onmessage = async (event: MessageEvent<{ id: number; req: Req }>) => {
    await ready;
    const { id, req } = event.data;
    let res: RpcResponse;
    try {
      res = { id, ok: true, value: handle(req) };
    } catch (e) {
      res = { id, ok: false, error: e instanceof Error ? e.message : String(e) };
    }
    self.postMessage(res);
  };
}
