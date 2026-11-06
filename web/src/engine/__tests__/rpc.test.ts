import { afterEach, describe, expect, it, vi } from "vitest";
import { serve } from "../rpc";

const realSelf = globalThis.self;

afterEach(() => {
  globalThis.self = realSelf;
});

async function ask(handle: (req: number) => unknown, req: number) {
  const posted = vi.fn();
  const fake = { onmessage: null as unknown, postMessage: posted };
  globalThis.self = fake as unknown as typeof globalThis.self;
  serve<number>(Promise.resolve(), handle);
  await (fake.onmessage as (e: { data: unknown }) => Promise<void>)({ data: { id: 7, req } });
  return posted.mock.calls[0][0];
}

describe("serve", () => {
  it("answers with a resolved promise", async () => {
    expect(await ask((n) => Promise.resolve(n * 3), 1)).toEqual({ id: 7, ok: true, value: 3 });
  });

  it("answers a rejected promise with its message", async () => {
    expect(await ask(() => Promise.reject(new Error("non")), 1)).toEqual({ id: 7, ok: false, error: "non" });
  });
});
