import { describe, expect, it, vi } from "vitest";
import { TwixtNetLoader } from "../twixtNet";

describe("TwixtNetLoader", () => {
  it("fetches and installs the weights once", async () => {
    const fetcher = vi.fn(async () => new Uint8Array([1, 2]));
    const install = vi.fn();
    const loader = new TwixtNetLoader(fetcher, install);
    await loader.ensure();
    await loader.ensure();
    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(install).toHaveBeenCalledWith(new Uint8Array([1, 2]));
  });

  it("rejects a failed fetch with a clear message, then retries", async () => {
    let fail = true;
    const fetcher = vi.fn(async () => {
      if (fail) throw new Error("404");
      return new Uint8Array([3]);
    });
    const loader = new TwixtNetLoader(fetcher, vi.fn());
    await expect(loader.ensure()).rejects.toThrow(/^Poids de TwixT non chargés/);
    fail = false;
    await expect(loader.ensure()).resolves.toBeUndefined();
    expect(fetcher).toHaveBeenCalledTimes(2);
  });
});
