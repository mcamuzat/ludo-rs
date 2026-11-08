import { afterEach, describe, expect, it } from "vitest";
import { setLang } from "../../i18n";
import type { GameMeta } from "../../engine/types";
import { playersCount, playersLabel } from "../players";

const meta = (playerCounts: number[]): GameMeta => ({
  id: "g",
  name: "G",
  minPlayers: playerCounts[0],
  maxPlayers: playerCounts[playerCounts.length - 1],
  playerCounts,
  hiddenInfo: false, sharedScreen: true,
  clock: false,
  options: [],
});

describe("playersLabel", () => {
  it("shows a run as a range, gaps with « ou », a single size alone", () => {
    expect(playersLabel(meta([2, 3, 4]))).toBe("2–4");
    expect(playersLabel(meta([2, 4]))).toBe("2 ou 4");
    expect(playersLabel(meta([4]))).toBe("4");
  });
});

describe("in English", () => {
  afterEach(() => setLang("fr"));

  it("says « or » and « players »", () => {
    setLang("en");
    expect(playersLabel(meta([2, 4]))).toBe("2 or 4");
    expect(playersCount(meta([2, 3, 4]))).toBe("2–4 players");
  });

  it("keeps « joueurs » in French", () => {
    expect(playersCount(meta([2, 3, 4]))).toBe("2–4 joueurs");
  });
});
