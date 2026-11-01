import type { GameConfig, GameMeta } from "../engine/types";

/** Lobby labels by `<game id>.<key>`, and `<game id>.<key>.<value>` for choices. Missing: the key itself. */
const LABELS: Record<string, string> = {
  "pairs-ghost-town.hands": "Mains",
  "pairs-bread-basket.rounds": "Mains",
  "pairs-bread-basket.rounds.Six": "6",
  "pairs-bread-basket.rounds.PerPlayer": "Une donne par joueur",
  "pairs-pickup.length": "Durée",
  "pairs-pickup.length.OneDeal": "Une donne",
  "pairs-pickup.length.Target": "Partie longue (150 / 100 / 75 points)",
  "cards-collusion.target": "Points à atteindre",
  "cards-lamarckian.competitions": "Concours",
  "black-hole.order": "Ordre des tuiles",
  "black-hole.order.Ascending": "Croissant (règle)",
  "black-hole.order.Free": "Libre (variante 1)",
  "black-hole.order.Random": "Au hasard (variante 2)",
  "corporation.table": "Table des promotions",
  "corporation.table.Draft": "Posée par les joueurs (règle)",
  "corporation.table.Random": "Au hasard",
  "cards-cobras.dummy": "Pile du fictif (à 2 joueurs)",
  "cards-cobras.dummy.Hidden": "Cachée (haute surprise)",
  "cards-cobras.dummy.Shown": "Retournée (haute stratégie)",
  "bantu.short": "Partie courte (pions 1 à 3)",
  "cards-president.rounds": "Manches",
  "cards-president.revolution": "Révolution",
  "cards-president.no_finish_on_two": "Pas de fin sur un 2",
  "cards-president.jokers": "Jokers",
  "cards-president.jokers.Off": "Sans",
  "cards-president.jokers.High": "Forts",
  "cards-president.jokers.Wild": "Wild (remplacent une valeur)",
  "pennywise.character": "Personnage",
  "volltreffer.narrow_sanctuary": "Sanctuaire réduit (63–69)",
  "cryptide.mode": "Mode",
  "cryptide.mode.Normal": "Normal",
  "cryptide.mode.Advanced": "Avancé (indices négatifs, structures noires)",
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
