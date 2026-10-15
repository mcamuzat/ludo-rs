import { describe, expect, it } from "vitest";
import { GHOST_TOWN_RULES } from "../rules";

describe("Pairs rules reminders", () => {
  it("does not fix Ghost Town's number of hands, which the lobby sets", () => {
    const text = GHOST_TOWN_RULES.join(" ");
    expect(text).not.toMatch(/\b5 mains,/);
    expect(text).toContain("réglable dans le lobby");
  });
});
