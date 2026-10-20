// French wording of the clues, after the Origames rulebook (p. 9–10).
import type { Animal, Clue, Hint, Kind, StructureColor, Terrain } from "./types";

export const TERRAIN_NAMES: Record<Terrain, string> = {
  Lake: "Lac",
  Mountain: "Montagne",
  Forest: "Forêt",
  Swamp: "Marécage",
  Desert: "Désert",
};

/** "une Forêt", "un Désert"… */
const WITH_ARTICLE: Record<Terrain, string> = {
  Lake: "un Lac",
  Mountain: "une Montagne",
  Forest: "une Forêt",
  Swamp: "un Marécage",
  Desert: "un Désert",
};

const KIND_NAMES: Record<Kind, string> = { Stone: "une pierre dressée", Shack: "une cabane abandonnée" };
const ANIMAL_NAMES: Record<Animal, string> = { Bear: "d'un ours", Cougar: "d'un puma" };
const COLOR_NAMES: Record<StructureColor, string> = {
  White: "blanche",
  Green: "verte",
  Blue: "bleue",
  Black: "noire",
};

/** "à 1 case d'un Désert": "d'" before a vowel. */
function of(phrase: string): string {
  return /^[aeiouyéèêh]/i.test(phrase) ? `d'${phrase}` : `de ${phrase}`;
}

/** What follows "L'habitat se trouve" / "L'habitat ne se trouve pas". */
export function clueText(clue: Clue): string {
  if (clue === "NearAnimals") return "dans, ou à 1 case du territoire d'un animal";
  if ("Terrains" in clue) {
    const [a, b] = clue.Terrains;
    return `dans ${WITH_ARTICLE[a]} ou ${WITH_ARTICLE[b]}`;
  }
  if ("NearTerrain" in clue) return `dans, ou à 1 case ${of(WITH_ARTICLE[clue.NearTerrain])}`;
  if ("NearKind" in clue) return `dans, ou jusqu'à 2 cases ${of(KIND_NAMES[clue.NearKind])}`;
  if ("NearAnimal" in clue) return `dans, ou jusqu'à 2 cases du territoire ${ANIMAL_NAMES[clue.NearAnimal]}`;
  return `dans, ou jusqu'à 3 cases d'une structure ${COLOR_NAMES[clue.NearColor]}`;
}

export function hintLabel(hint: Hint): string {
  return `L'habitat ${hint.negated ? "ne se trouve pas" : "se trouve"} ${clueText(hint.clue)}.`;
}

export type Family = "terrains" | "one" | "two" | "three";

export const FAMILY_TITLES: Record<Family, string> = {
  terrains: "Dans un terrain parmi deux",
  one: "Dans, ou à 1 case",
  two: "Dans, ou jusqu'à 2 cases",
  three: "Dans, ou jusqu'à 3 cases",
};

export function family(clue: Clue): Family {
  if (clue === "NearAnimals" || "NearTerrain" in clue) return "one";
  if ("Terrains" in clue) return "terrains";
  if ("NearColor" in clue) return "three";
  return "two";
}
