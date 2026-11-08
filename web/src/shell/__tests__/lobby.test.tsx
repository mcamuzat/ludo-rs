import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, describe, expect, it } from "vitest";
import type { GameMeta } from "../../engine/types";
import { setLang } from "../../i18n";
import { Lobby } from "../Lobby";

const TIGRIS: GameMeta = {
  id: "tigre-euphrate",
  name: "Tigre et Euphrate",
  minPlayers: 2,
  maxPlayers: 4,
  playerCounts: [2, 3, 4],
  hiddenInfo: true,
  sharedScreen: false,
  clock: true,
  options: [{ key: "reinforcements", kind: { type: "choice", values: ["English", "Free"], default: "English" } }],
};

function lobby(): string {
  const noop = () => {};
  return renderToStaticMarkup(<Lobby game={TIGRIS} onLocal={noop} onHost={noop} onBack={noop} />);
}

describe("Lobby", () => {
  afterEach(() => setLang("fr"));

  it("speaks French by default", () => {
    const html = lobby();
    expect(html).toContain("IA moyenne");
    expect(html).toContain("Jouer sur cet appareil");
    expect(html).toContain("Tigre et Euphrate");
  });

  it("speaks English without a French word", () => {
    setLang("en");
    const html = lobby();
    expect(html).toContain("Tigris &amp; Euphrates");
    expect(html).toContain("Easy AI");
    expect(html).toContain("Play on this device");
    expect(html).toContain("Host online");
    expect(html).toContain("Rules reminder");
    expect(html).toContain("2–4 players");
    expect(html).toContain("Online clock");
    expect(html).toContain("None");
    // The rules reminder's lines belong to each game (translated with it, not in this lot).
    const frame = html.replace(/<ul>.*?<\/ul>/s, "");
    expect(frame).not.toMatch(/Jouer|Héberger|joueurs|Siège|Aucun|[éèêàç]/);
  });
});
