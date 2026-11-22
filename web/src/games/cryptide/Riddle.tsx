import { useCallback, useEffect, useRef, useState } from "react";
import { EngineClient, randomSeed } from "../../engine/clients";
import { aiSeatName, errorMessage } from "../../session/names";
import { describeEvent, place } from "./Board";
import { HexMap, SEAT_FILL } from "./HexMap";
import { HintHelp } from "./HintHelp";
import { FAMILY_TITLES, family, hintLabel, type Family } from "./hints";
import type { Cell, Combination, Hint, Mode, Outcome, Riddle, RiddleLevel } from "./types";
import "./cryptide.css";

const NO_CELLS = new Set<Cell>();
const FAMILIES: Family[] = ["terrains", "one", "two", "three"];

export const LEVELS: { level: RiddleLevel; label: string }[] = [
  { level: "Warmup", label: "1 · Échauffement" },
  { level: "Hint", label: "2 · L'indice d'un joueur" },
  { level: "Easy", label: "3 · Énigme facile" },
  { level: "Full", label: "4 · Énigme" },
];

/** Two hints are the same clue with the same sign. */
export function sameHint(a: Hint, b: Hint): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** A player's attempt at one riddle, before any guess. */
export function freshAttempt() {
  return { misses: [] as Cell[], hintMisses: [] as Hint[], solved: null as "found" | "shown" | null, shaded: null as number | null };
}

/** A hint's sentence without its final full stop, to go inside a line. */
function bare(h: Hint): string {
  return hintLabel(h).replace(/\.$/, "");
}

function outcomeText(o: Outcome): string {
  if (o === "Nowhere") return "aucune case commune";
  if (o === "Repeated") return "deux joueurs auraient le même indice";
  if ("Several" in o) return `${o.Several} cases possibles`;
  return place(o.Habitat);
}

function CombinationLine({ c, names }: { c: Combination; names: string[] }) {
  return (
    <li>
      {c.hints.map((h, seat) => `${names[seat]} : ${bare(h)}`).join(" · ")} → {outcomeText(c.outcome)}
    </li>
  );
}

/** One riddle: the map, what the level gives, the player's guesses. */
export function RiddleBoard({
  riddle,
  misses,
  hintMisses,
  solved,
  shaded,
  onGuess,
  onGuessHint,
  onShade,
  onGiveUp,
}: {
  riddle: Riddle;
  misses: Cell[];
  /** Level Hint: the wrong hints picked. */
  hintMisses: Hint[];
  /** Found, or given up: the answer is shown. */
  solved: "found" | "shown" | null;
  /** Warm-up solution: the seat whose hint's spaces are shown. */
  shaded: number | null;
  onGuess(cell: Cell): void;
  onGuessHint(hint: Hint): void;
  onShade(seat: number | null): void;
  onGiveUp(): void;
}) {
  const names = Array.from({ length: riddle.players }, (_, seat) => aiSeatName(seat));
  const solver = names[riddle.solver];
  const level = riddle.level;
  const findsHint = level === "Hint";
  const tries = findsHint ? hintMisses.length : misses.length;
  // Any space without a cube may be guessed until the answer is out.
  const open =
    solved || findsHint
      ? NO_CELLS
      : new Set(riddle.tokens.flatMap((t, cell) => (t.cube === null && !misses.includes(cell) ? [cell] : [])));
  const shade = solved && shaded !== null ? new Set(riddle.hint_cells[shaded]) : null;
  return (
    <div className="cr-board">
      <ul className="legend">
        {names.map((name, seat) => (
          <li key={seat} className={level !== "Warmup" && seat === riddle.solver ? "active" : undefined}>
            <span className="cr-swatch" style={{ background: SEAT_FILL[seat] }} />
            {name}
          </li>
        ))}
      </ul>
      {level === "Warmup" && (
        <>
          <p className="cr-status">Voici tous les indices. Trouve la seule case qui les vérifie tous.</p>
          <ul className="cr-given">
            {riddle.hints.map((h, seat) => (
              <li key={seat}>
                <strong>{names[seat]} :</strong> {hintLabel(h)}
              </li>
            ))}
          </ul>
        </>
      )}
      {findsHint && (
        <p className="cr-status">
          Les jetons de <strong>{solver}</strong> ne laissent qu'un indice possible. Lequel ?
        </p>
      )}
      {(level === "Easy" || level === "Full") && (
        <>
          <p className="cr-status">
            La partie s'arrête : <strong>{solver}</strong> sait où se trouve l'habitat. Mets-toi à sa place.
          </p>
          <p className="cr-my-hint">
            <strong>Indice de {solver} :</strong> {hintLabel(riddle.hint)}
          </p>
        </>
      )}
      <div className="cr-table felt">
        <HexMap
          squares={riddle.squares}
          tokens={riddle.tokens}
          playable={open}
          selected={null}
          focus={null}
          habitat={solved && !findsHint ? riddle.habitat : null}
          misses={misses}
          shaded={shade}
          onPick={onGuess}
        />
      </div>
      {solved === null && (
        <div className="cr-controls">
          {tries > 0 && (
            <span className="notice">
              Raté : {tries > 1 ? `${tries} essais` : "1 essai"}
              {findsHint ? " (indices barrés)." : " (cases barrées)."}
            </span>
          )}
          <span>{findsHint ? `Choisis l'indice de ${solver} ci-dessous.` : "Clique sur la case de l'habitat."}</span>
          <button type="button" className="ghost" onClick={onGiveUp}>
            Voir la solution
          </button>
        </div>
      )}
      {findsHint && solved === null && (
        <div className="cr-picker">
          {FAMILIES.map((f) => {
            const list = riddle.mode_hints.filter((h) => family(h.clue) === f);
            return (
              <section key={f}>
                <h4>{FAMILY_TITLES[f]}</h4>
                <ul>
                  {list.map((h) => (
                    <li key={JSON.stringify(h)}>
                      <button
                        type="button"
                        className="ghost"
                        data-hint={JSON.stringify(h)}
                        disabled={hintMisses.some((m) => sameHint(m, h))}
                        onClick={() => onGuessHint(h)}
                      >
                        {hintLabel(h)}
                      </button>
                    </li>
                  ))}
                </ul>
              </section>
            );
          })}
        </div>
      )}
      {solved && (
        <div className="cr-reveal">
          <h3>
            {solved === "found"
              ? `Trouvé${tries > 0 ? ` en ${tries + 1} essais` : " du premier coup"} !`
              : "La solution"}
          </h3>
          {findsHint ? (
            <>
              <p>
                L'indice de {solver} : {hintLabel(riddle.hint)}
              </p>
              <h4>Pourquoi les autres indices tombent</h4>
              <ul>
                {riddle.eliminations.map((e, i) => (
                  <li key={i}>
                    {bare(e.hint)} : écarté par {e.mark === "Disc" ? "le disque" : "le cube"} de {solver} en{" "}
                    {place(e.cell)}
                  </li>
                ))}
              </ul>
            </>
          ) : (
            <p>L'habitat est en {place(riddle.habitat)}.</p>
          )}
          {level === "Warmup" && (
            <>
              <h4>Les cases que garde chaque indice</h4>
              <ul className="cr-shade-list">
                {riddle.hints.map((h, seat) => (
                  <li key={seat}>
                    <button
                      type="button"
                      className={`ghost${shaded === seat ? " active" : ""}`}
                      onClick={() => onShade(shaded === seat ? null : seat)}
                    >
                      {names[seat]} : {hintLabel(h)}
                    </button>
                  </li>
                ))}
              </ul>
            </>
          )}
          {(level === "Easy" || level === "Full") && (
            <>
              <h4>Ce que {solver} pouvait déduire des autres</h4>
              <ul>
                {riddle.deduction.map((list, seat) =>
                  seat === riddle.solver ? null : (
                    <li key={seat}>
                      <span className="cr-swatch" style={{ background: SEAT_FILL[seat] }} />
                      {names[seat]} : {list.length > 1 ? `un indice parmi ${list.length}` : "un seul indice possible"}
                      <ul className="cr-deduction">
                        {list.map((h, i) => (
                          <li key={i} className={sameHint(h, riddle.hints[seat]) ? "true" : undefined}>
                            {hintLabel(h)}
                          </li>
                        ))}
                      </ul>
                    </li>
                  ),
                )}
              </ul>
              <p className="cr-help-note">En gras, l'indice que chacun avait vraiment.</p>
            </>
          )}
          {level === "Full" && riddle.combinations.working.length > 0 && (
            <>
              <h4>Combinaisons qui marchent</h4>
              <ul className="cr-combos">
                {riddle.combinations.working.map((c, i) => (
                  <CombinationLine key={i} c={c} names={names} />
                ))}
              </ul>
              <p>
                Tombées : {riddle.combinations.nowhere} sans case commune, {riddle.combinations.several} qui laissent
                plusieurs cases, {riddle.combinations.repeated} où deux joueurs auraient le même indice
                {riddle.combinations.complete ? "." : " (liste incomplète)."}
              </p>
              {riddle.combinations.fallen.length > 0 && (
                <details className="cr-help">
                  <summary>Exemples de combinaisons tombées</summary>
                  <ul className="cr-combos">
                    {riddle.combinations.fallen.map((c, i) => (
                      <CombinationLine key={i} c={c} names={names} />
                    ))}
                  </ul>
                </details>
              )}
            </>
          )}
        </div>
      )}
      {!findsHint && <HintHelp hints={riddle.mode_hints} />}
      {riddle.log.length > 0 && (
        <details className="cr-help">
          <summary>Déroulement de la partie ({riddle.log.length} étapes)</summary>
          <ol className="cr-log">
            {riddle.log.map((e, i) => (
              <li key={i}>{describeEvent(e, names)}</li>
            ))}
          </ol>
        </details>
      )}
    </div>
  );
}

const INTRO: Record<RiddleLevel, string> = {
  Warmup: "Tous les indices sont donnés : trouve la case qu'ils désignent ensemble.",
  Hint: "Une table d'IA joue une partie, arrêtée dès que les jetons d'un joueur ne laissent qu'un indice possible : retrouve-le.",
  Easy: "Une table d'IA joue une partie. Dès que l'une d'elles peut déduire l'habitat et que les jetons trahissent l'indice de chaque autre joueur, la partie s'arrête : tu reçois son indice, à toi de trouver la case.",
  Full: "Une table d'IA joue une partie. Dès que l'une d'elles peut déduire l'habitat, la partie s'arrête : tu reçois son indice, à toi de trouver la case.",
};

/** The riddles screen: four levels, from the warm-up to a full riddle. */
export function CryptideRiddles({ onBack }: { onBack(): void }) {
  const engine = useRef<EngineClient | null>(null);
  const [level, setLevel] = useState<RiddleLevel>("Warmup");
  const [players, setPlayers] = useState(4);
  const [mode, setMode] = useState<Mode>("Normal");
  const [riddle, setRiddle] = useState<Riddle | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(freshAttempt);

  useEffect(() => {
    const client = new EngineClient();
    engine.current = client;
    next("Warmup", 4, "Normal");
    return () => {
      engine.current = null;
      client.dispose();
    };
    // The first riddle comes with the engine (`next` is stable).
  }, []);

  const next = useCallback((l: RiddleLevel, n: number, m: Mode) => {
    const client = engine.current;
    if (!client) return;
    setBusy(true);
    setError(null);
    // A disposed engine (left the screen) answers nothing worth showing.
    const live = () => engine.current === client;
    client.cryptideRiddle<Riddle>(l, m, n, randomSeed()).then(
      (r) => {
        if (!live()) return;
        setRiddle(r);
        setAttempt(freshAttempt());
        setBusy(false);
      },
      (e) => {
        if (!live()) return;
        setError(errorMessage(e));
        setBusy(false);
      },
    );
  }, []);

  const guess = (cell: Cell) => {
    if (!riddle || attempt.solved) return;
    if (cell === riddle.habitat) setAttempt((a) => ({ ...a, solved: "found" }));
    else setAttempt((a) => ({ ...a, misses: [...a.misses, cell] }));
  };
  const guessHint = (hint: Hint) => {
    if (!riddle || attempt.solved) return;
    if (sameHint(hint, riddle.hint)) setAttempt((a) => ({ ...a, solved: "found" }));
    else setAttempt((a) => ({ ...a, hintMisses: [...a.hintMisses, hint] }));
  };

  return (
    <main>
      <div className="topbar">
        <button className="ghost" onClick={onBack}>
          ← Menu
        </button>
        <h2>Énigmes de Cryptide</h2>
        <span className="pill">entraînement</span>
      </div>
      <div className="panel">
        <p className="hint">{INTRO[level]}</p>
        <div className="row">
          <label htmlFor="riddle-level">Niveau</label>
          <select id="riddle-level" value={level} onChange={(e) => setLevel(e.target.value as RiddleLevel)}>
            {LEVELS.map(({ level: l, label }) => (
              <option key={l} value={l}>
                {label}
              </option>
            ))}
          </select>
          <label htmlFor="riddle-players">Joueurs</label>
          <select id="riddle-players" value={players} onChange={(e) => setPlayers(Number(e.target.value))}>
            {[3, 4, 5].map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
          <label htmlFor="riddle-mode">Mode</label>
          <select id="riddle-mode" value={mode} onChange={(e) => setMode(e.target.value as Mode)}>
            <option value="Normal">Normal</option>
            <option value="Advanced">Avancé</option>
          </select>
          <button className="primary" disabled={busy} onClick={() => next(level, players, mode)}>
            {riddle ? "Nouvelle énigme" : "Énigme"}
          </button>
        </div>
      </div>
      {error && <p className="error">{error}</p>}
      {busy && <p className="status">{level === "Warmup" ? "Préparation…" : "La table joue…"}</p>}
      {riddle && !busy && (
        <RiddleBoard
          riddle={riddle}
          misses={attempt.misses}
          hintMisses={attempt.hintMisses}
          solved={attempt.solved}
          shaded={attempt.shaded}
          onGuess={guess}
          onGuessHint={guessHint}
          onShade={(seat) => setAttempt((a) => ({ ...a, shaded: seat }))}
          onGiveUp={() => setAttempt((a) => ({ ...a, solved: "shown" }))}
        />
      )}
    </main>
  );
}
