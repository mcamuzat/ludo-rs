/**
 * TwixT's network weights: fetched once per worker, only when a TwixT AI
 * move or coach request needs them (they are not in the bundle).
 */
export class TwixtNetLoader {
  private ready: Promise<void> | null = null;

  constructor(
    private fetcher: () => Promise<Uint8Array>,
    private install: (bytes: Uint8Array) => void,
  ) {}

  /** Resolves once the weights are installed; a failure is retried next time. */
  ensure(): Promise<void> {
    this.ready ??= this.fetcher().then(
      (bytes) => this.install(bytes),
      (e: unknown) => {
        this.ready = null;
        throw new Error(`Poids de TwixT non chargés : ${e instanceof Error ? e.message : String(e)}`);
      },
    );
    return this.ready;
  }
}

/** The weights file, served from `public/twixt/`. */
export async function fetchTwixtNet(): Promise<Uint8Array> {
  const res = await fetch(`${import.meta.env.BASE_URL}twixt/net-f16.bin`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return new Uint8Array(await res.arrayBuffer());
}
