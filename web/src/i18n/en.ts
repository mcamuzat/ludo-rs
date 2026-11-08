import type { Dict } from "./fr";

export const en: Dict = {
  gameNames: {
    "cards-president": "President",
    "pairs-continuous": "Continuous Pairs",
    "pairs-auberge": "Regent",
    "tigre-euphrate": "Tigris & Euphrates",
    cryptide: "Cryptid",
    riichi: "Riichi Mahjong",
  },
  language: { label: "Language", fr: "Français", en: "English" },
  players: {
    or: "or",
    count: (label: string) => `${label} players`,
  },
  home: {
    yourName: "Your name",
    joinGame: "Join a game",
    join: "Join",
    hostedGame: "Hosted game in progress",
    code: "code",
    resume: "Resume",
    abandon: "Abandon",
    loading: "Loading…",
    shelves: {
      board: "Board and tiles",
      pairs: "Pairs deck",
      cards: "Traditional cards",
      modern: "Designer card games",
      coins: "Coin games",
      trade: "Trading games",
    },
    games: (n: number) => (n > 1 ? `${n} games` : "1 game"),
    riddles: "Riddles",
    training: "training",
    riddlesHint: "Find the habitat in an AI's place",
  },
};
