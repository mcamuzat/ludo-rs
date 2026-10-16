import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { GameMeta } from "../../engine/types";
import { RulesReminder } from "../RulesReminder";
import { Home } from "../Home";

function meta(id: string, name: string): GameMeta {
  return { id, name, minPlayers: 2, maxPlayers: 6, hiddenInfo: true, clock: false, options: [] };
}

const CATALOG = [
  meta("yavalath", "Yavalath"),
  meta("riichi", "Mahjong Riichi"),
  meta("pairs-continuous", "Jour de chance"),
  meta("pairs-zombie-life", "Zombie Life"),
  meta("pairs-auberge", "L'Auberge des Pirates"),
  meta("pairs-deadfall", "Deadfall"),
  meta("pairs-ghost-town", "Ghost Town"),
  meta("cards-president", "Président"),
  meta("pennywise", "Pennywise"),
];

function home(): string {
  const noop = () => {};
  return renderToStaticMarkup(
    <Home
      catalog={CATALOG}
      error={null}
      name="Ana"
      onName={noop}
      onPick={noop}
      onJoin={noop}
      saved={null}
      onResume={noop}
      onAbandon={noop}
    />,
  );
}

describe("Home", () => {
  it("lists the games in alphabetical order and leaves Riichi out", () => {
    const html = home();
    expect(html).not.toContain("Mahjong Riichi");
    const names = [...html.matchAll(/<strong>([^<]+)<\/strong>/g)].map((m) => m[1].replace("&#x27;", "'"));
    expect(names).toEqual([
      "Yavalath",
      "Deadfall",
      "Ghost Town",
      "Jour de chance",
      "L'Auberge des Pirates",
      "Zombie Life",
      "Président",
      "Pennywise",
    ]);
  });

  it("shows a rules reminder only when there are rules", () => {
    expect(renderToStaticMarkup(<RulesReminder rules={undefined} />)).toBe("");
    const html = renderToStaticMarkup(<RulesReminder rules={["Une règle.", "Une autre."]} />);
    expect(html).toContain("Rappel des règles");
    expect(html).toContain("<li>Une autre.</li>");
  });

  it("shelves the classic card games after the Pairs deck", () => {
    const html = home();
    expect(html).toContain("Cartes traditionnelles");
    expect(html.indexOf("Deck Pairs")).toBeLessThan(html.indexOf("Cartes traditionnelles"));
    expect(html.indexOf("Cartes traditionnelles")).toBeLessThan(html.indexOf("Président"));
  });

  it("shelves the coin games after the classic cards", () => {
    const html = home();
    expect(html.indexOf("Cartes traditionnelles")).toBeLessThan(html.indexOf("Jeux de pièces"));
    expect(html.indexOf("Jeux de pièces")).toBeLessThan(html.indexOf("Pennywise"));
  });
});
