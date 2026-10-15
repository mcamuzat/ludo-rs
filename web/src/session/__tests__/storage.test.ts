import { describe, expect, it } from "vitest";
import { savedConfig, type SavedHost } from "../storage";

const SAVE: SavedHost = { code: "ABCDEF", gameId: "yavalath", seats: [], log: null, clock: null, banks: [] };

describe("savedConfig", () => {
  it("resumes_a_save_without_config", () => {
    expect(savedConfig(SAVE)).toBeNull();
    expect(savedConfig({ ...SAVE, config: null })).toBeNull();
    expect(savedConfig({ ...SAVE, config: { hands: 3 } })).toEqual({ hands: 3 });
  });
});
