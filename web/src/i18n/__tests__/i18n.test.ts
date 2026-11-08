import { afterEach, describe, expect, it } from "vitest";
import { detectLang, gameName, getLang, readStoredLang, setLang, t } from "../index";
import { en } from "../en";
import { fr } from "../fr";

afterEach(() => setLang("fr"));

describe("language", () => {
  it("detects French browsers only", () => {
    expect(detectLang("fr-FR")).toBe("fr");
    expect(detectLang("fr")).toBe("fr");
    expect(detectLang("en-US")).toBe("en");
    expect(detectLang("de")).toBe("en");
    expect(detectLang(undefined)).toBe("en");
  });

  it("ignores a stored language it does not know, or storage that fails", () => {
    expect(readStoredLang(() => "de")).toBeNull();
    expect(readStoredLang(() => "")).toBeNull();
    expect(readStoredLang(() => null)).toBeNull();
    expect(
      readStoredLang(() => {
        throw new Error("blocked");
      }),
    ).toBeNull();
    expect(readStoredLang(() => "en")).toBe("en");
  });

  it("switches the dictionary", () => {
    setLang("en");
    expect(getLang()).toBe("en");
    expect(t().language.label).toBe("Language");
    setLang("fr");
    expect(t().language.label).toBe("Langue");
  });

  it("names games in the current language", () => {
    const meta = { id: "tigre-euphrate", name: "Tigre et Euphrate" };
    expect(gameName(meta)).toBe("Tigre et Euphrate");
    setLang("en");
    expect(gameName(meta)).toBe("Tigris & Euphrates");
    expect(gameName({ id: "yavalath", name: "Yavalath" })).toBe("Yavalath");
  });

  it("has an English name for the six games that differ", () => {
    expect(Object.keys(en.gameNames).sort()).toEqual([
      "cards-president",
      "cryptide",
      "pairs-auberge",
      "pairs-continuous",
      "riichi",
      "tigre-euphrate",
    ]);
  });

  it("labels every game option in English too", () => {
    expect(Object.keys(en.options).sort()).toEqual(Object.keys(fr.options).sort());
  });
});
