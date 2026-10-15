import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { GameMeta } from "../../engine/types";
import { defaultConfig, optionSummary } from "../../games/options";
import { GameOptions } from "../GameOptions";

const GHOST: GameMeta = {
  id: "pairs-ghost-town",
  name: "Ghost Town",
  minPlayers: 2,
  maxPlayers: 6,
  hiddenInfo: true,
  clock: false,
  options: [{ key: "hands", kind: { type: "number", min: 1, max: 15, default: 5 } }],
};

const MIXED: GameMeta = {
  ...GHOST,
  id: "test-mixed",
  options: [
    { key: "fast", kind: { type: "toggle", default: false } },
    { key: "mode", kind: { type: "choice", values: ["Off", "On"], default: "Off" } },
  ],
};

describe("game options", () => {
  it("builds the default config from the engine's options", () => {
    expect(defaultConfig(GHOST)).toEqual({ hands: 5 });
    expect(defaultConfig({ ...GHOST, options: [] })).toBeNull();
  });

  it("summarizes a config with the game's labels", () => {
    expect(optionSummary(GHOST, { hands: 3 })).toEqual(["Mains : 3"]);
    expect(optionSummary(MIXED, { fast: true, mode: "On" })).toEqual(["fast : oui", "mode : On"]);
  });

  it("offers every value of a number option, the current one selected", () => {
    const html = renderToStaticMarkup(<GameOptions meta={GHOST} config={{ hands: 3 }} onChange={() => {}} />);
    expect(html).toContain("Mains");
    expect(html.match(/<option /g)?.length).toBe(15);
    expect(html).toContain('<option value="3" selected="">3</option>');
  });

  it("renders toggles and choices", () => {
    const html = renderToStaticMarkup(<GameOptions meta={MIXED} config={{ fast: true, mode: "On" }} onChange={() => {}} />);
    expect(html).toContain('type="checkbox"');
    expect(html).toContain("checked");
    expect(html).toContain('<option value="On" selected="">On</option>');
  });

  it("is read-only without onChange", () => {
    const html = renderToStaticMarkup(<GameOptions meta={GHOST} config={{ hands: 3 }} />);
    expect(html).not.toContain("<select");
    expect(html).toContain("Mains : 3");
  });

  it("renders_nothing_without_options", () => {
    expect(renderToStaticMarkup(<GameOptions meta={{ ...GHOST, options: [] }} config={null} onChange={() => {}} />)).toBe("");
    expect(renderToStaticMarkup(<GameOptions meta={GHOST} config={null} />)).toBe("");
  });
});
