import { afterEach, describe, expect, it } from "vitest";
import { setLang } from "../../i18n";
import { aiSeatName, rankLabel, seatName } from "../names";

describe("seat names and ranks", () => {
  afterEach(() => setLang("fr"));

  it("names seats in French", () => {
    expect(aiSeatName(1)).toBe("Hugo (IA)");
    expect(seatName({ name: "", kind: "open", connected: false }, 2)).toBe("Place libre");
    expect(rankLabel(0)).toBe("1er");
    expect(rankLabel(1)).toBe("2ᵉ");
    expect(seatName({ name: "", kind: "local", connected: true }, 0)).toBe("Hôte");
    expect(seatName({ name: "", kind: "remote", connected: true }, 1)).toBe("Invité");
  });

  it("names seats in English, keeping the humans' own names", () => {
    setLang("en");
    expect(aiSeatName(1)).toBe("Hugo (AI)");
    expect(seatName({ name: "", kind: "ai", connected: true }, 2)).toBe("Lina (AI)");
    expect(seatName({ name: "", kind: "open", connected: false }, 2)).toBe("Open seat");
    expect(seatName({ name: "Ana", kind: "remote", connected: true }, 0)).toBe("Ana");
    expect(seatName({ name: "", kind: "local", connected: true }, 0)).toBe("Host");
    expect(seatName({ name: "", kind: "remote", connected: true }, 1)).toBe("Guest");
  });

  it("writes English ordinals", () => {
    setLang("en");
    expect([0, 1, 2, 3].map(rankLabel)).toEqual(["1st", "2nd", "3rd", "4th"]);
    expect(rankLabel(10)).toBe("11th");
    expect(rankLabel(11)).toBe("12th");
    expect(rankLabel(20)).toBe("21st");
  });
});
