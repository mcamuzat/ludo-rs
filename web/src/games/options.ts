import type { GameConfig, GameMeta } from "../engine/types";

/** Lobby labels by `<game id>.<key>`, and `<game id>.<key>.<value>` for choices. Missing: the key itself. */
const LABELS: Record<string, string> = {
  "pairs-ghost-town.hands": "Mains",
  "cards-president.rounds": "Manches",
  "cards-president.revolution": "Révolution",
  "cards-president.no_finish_on_two": "Pas de fin sur un 2",
  "cards-president.jokers": "Jokers",
  "cards-president.jokers.Off": "Sans",
  "cards-president.jokers.High": "Forts",
  "cards-president.jokers.Wild": "Wild (remplacent une valeur)",
  "pennywise.character": "Personnage",
};

export function optionLabel(gameId: string, key: string): string {
  return LABELS[`${gameId}.${key}`] ?? key;
}

export function valueLabel(gameId: string, key: string, value: string): string {
  return LABELS[`${gameId}.${key}.${value}`] ?? value;
}

/** The engine's defaults, or null for a game without options. */
export function defaultConfig(meta: GameMeta): GameConfig | null {
  if (meta.options.length === 0) return null;
  return Object.fromEntries(meta.options.map((o) => [o.key, o.kind.default]));
}

/** One "Label : valeur" line per option, for whoever cannot change them. */
export function optionSummary(meta: GameMeta, config: GameConfig): string[] {
  return meta.options.map(({ key }) => {
    const value = config[key];
    const shown = typeof value === "boolean" ? (value ? "oui" : "non") : valueLabel(meta.id, key, String(value));
    return `${optionLabel(meta.id, key)} : ${shown}`;
  });
}
