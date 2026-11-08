import type { GameConfig, GameMeta } from "../engine/types";
import { t } from "../i18n";

export function optionLabel(gameId: string, key: string): string {
  return t().options[`${gameId}.${key}`] ?? key;
}

export function valueLabel(gameId: string, key: string, value: string): string {
  return t().options[`${gameId}.${key}.${value}`] ?? value;
}

/** The engine's defaults, or null for a game without options. */
export function defaultConfig(meta: GameMeta): GameConfig | null {
  if (meta.options.length === 0) return null;
  return Object.fromEntries(meta.options.map((o) => [o.key, o.kind.default]));
}

/** One "Label : valeur" ("Label: value") line per option, for whoever cannot change them. */
export function optionSummary(meta: GameMeta, config: GameConfig): string[] {
  const words = t().optionWords;
  return meta.options.map(({ key }) => {
    const value = config[key];
    const shown = typeof value === "boolean" ? (value ? words.yes : words.no) : valueLabel(meta.id, key, String(value));
    return words.line(optionLabel(meta.id, key), shown);
  });
}
