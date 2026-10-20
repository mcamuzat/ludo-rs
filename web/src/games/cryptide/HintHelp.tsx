import { FAMILY_TITLES, family, hintLabel, type Family } from "./hints";
import type { Hint } from "./types";

const ORDER: Family[] = ["terrains", "one", "two", "three"];

/** The player aid: every clue of the mode, by family. No marks, no counts. */
export function HintHelp({ hints }: { hints: Hint[] }) {
  return (
    <details className="cr-help">
      <summary>Aide de jeu : les indices possibles</summary>
      <p className="cr-help-note">« Dans, ou à N cases » comprend la case de l'élément lui-même.</p>
      {ORDER.map((f) => {
        const list = hints.filter((h) => family(h.clue) === f);
        return (
          <section key={f}>
            <h4>{FAMILY_TITLES[f]}</h4>
            <ul>
              {list.map((h, i) => (
                <li key={i}>{hintLabel(h)}</li>
              ))}
            </ul>
          </section>
        );
      })}
    </details>
  );
}
