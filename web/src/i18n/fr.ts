// French: the reference dictionary. `en.ts` has the same shape (`Dict`),
// so a missing English text fails the type check.

export const fr = {
  /** Names that differ from the engine's `GameMeta.name`, by game id. */
  gameNames: {} as Record<string, string>,
  language: { label: "Langue", fr: "Français", en: "English" },
  players: {
    or: "ou",
    /** "2–4" → "2–4 joueurs". */
    count: (label: string) => `${label} joueurs`,
  },
  home: {
    yourName: "Ton nom",
    joinGame: "Rejoindre une partie",
    join: "Rejoindre",
    hostedGame: "Partie hébergée en cours",
    code: "code",
    resume: "Reprendre",
    abandon: "Abandonner",
    loading: "Chargement…",
    shelves: {
      board: "Plateau et tuiles",
      pairs: "Deck Pairs",
      cards: "Cartes traditionnelles",
      modern: "Jeux de cartes d'auteur",
      coins: "Jeux de pièces",
      trade: "Jeux d'échange",
    },
    games: (n: number) => (n > 1 ? `${n} jeux` : "1 jeu"),
    riddles: "Énigmes",
    training: "entraînement",
    riddlesHint: "Trouve l'habitat à la place d'une IA",
  },
};

export type Dict = typeof fr;
