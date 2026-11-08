// French: the reference dictionary. `en.ts` has the same shape (`Dict`),
// so a missing English text fails the type check.

export const fr = {
  /** Names that differ from the engine's `GameMeta.name`, by game id. */
  gameNames: {} as Record<string, string>,
  language: { label: "Langue", fr: "Français", en: "English" },
};

export type Dict = typeof fr;
