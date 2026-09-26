import { describe, expect, it } from "vitest";
import { StaleActionError, isStale } from "../rev";

describe("rev", () => {
  it("only the current revision is fresh", () => {
    expect(isStale(3, 3)).toBe(false);
    expect(isStale(2, 3)).toBe(true);
    expect(isStale(4, 3)).toBe(true);
  });

  it("the stale error is recognisable", () => {
    expect(new StaleActionError()).toBeInstanceOf(StaleActionError);
    expect(new StaleActionError().message).toMatch(/avancé/);
  });
});
