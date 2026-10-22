import { useCallback, useEffect, useRef, useState } from "react";
import { EngineClient, randomSeed } from "../../engine/clients";
import { aiSeatName, errorMessage } from "../../session/names";
import { describeEvent, place } from "./Board";
import { HexMap, SEAT_FILL } from "./HexMap";
import { HintHelp } from "./HintHelp";
import { hintLabel } from "./hints";
import type { Cell, Mode, Riddle } from "./types";
import "./cryptide.css";

const NO_CELLS = new Set<Cell>();

/** One riddle: the map, the solver's hint, the player's guesses. */
export function RiddleBoard({
  riddle,
  misses,
  solved,
  onGuess,
  onGiveUp,
}: {
  riddle: Riddle;
  misses: Cell[];
  /** Found, or given up: the answer is shown. */
  solved: "found" | "shown" | null;
  onGuess(cell: Cell): void;
  onGiveUp(): void;
}) {
  const names = Array.from({ length: riddle.players }, (_, seat) => aiSeatName(seat));
  const solver = names[riddle.solver];
  // Any space without a cube may be guessed until the answer is out.
  const open = solved
    ? NO_CELLS
    : new Set(riddle.tokens.flatMap((t, cell) => (t.cube === null && !misses.includes(cell) ? [cell] : [])));
  return (
    <div className="cr-board">
      <ul className="legend">
        {names.map((name, seat) => (
          <li key={seat} className={seat === riddle.solver ? "active" : undefined}>
            <span className="cr-swatch" style={{ background: SEAT_FILL[seat] }} />
            {name}
          </li>
        ))}
      </ul>
      <p className="cr-status">
        La partie s'arrête : <strong>{solver}</strong> sait où se trouve l'habitat. Mets-toi à sa place.
      </p>
      <p className="cr-my-hint">
        <strong>Indice de {solver} :</strong> {hintLabel(riddle.hint)}
      </p>
      <div className="cr-table felt">
        <HexMap
          squares={riddle.squares}
          tokens={riddle.tokens}
          playable={open}
          selected={null}
          focus={null}
          habitat={solved ? riddle.habitat : null}
          misses={misses}
          onPick={onGuess}
        />
      </div>
      {solved === null && (
        <div className="cr-controls">
          {misses.length > 0 && (
            <span className="notice">
              Raté : {misses.length > 1 ? `${misses.length} essais` : "1 essai"} (cases barrées).
            </span>
          )}
          <span>Clique sur la case de l'habitat.</span>
          <button type="button" className="ghost" onClick={onGiveUp}>
            Voir la solution
          </button>
        </div>
      )}
      {solved && (
        <div className="cr-reveal">
          <h3>
            {solved === "found"
              ? `Trouvé${misses.length > 0 ? ` en ${misses.length + 1} essais` : " du premier coup"} !`
              : "La solution"}
          </h3>
          <p>L'habitat est en {place(riddle.habitat)}.</p>
          <h4>Ce que {solver} pouvait déduire des autres</h4>
          <ul>
            {riddle.deduction.map((list, seat) =>
              seat === riddle.solver ? null : (
                <li key={seat}>
                  <span className="cr-swatch" style={{ background: SEAT_FILL[seat] }} />
                  {names[seat]} : {list.length > 1 ? `un indice parmi ${list.length}` : "un seul indice possible"}
                  <ul className="cr-deduction">
                    {list.map((h, i) => (
                      <li key={i} className={JSON.stringify(h) === JSON.stringify(riddle.hints[seat]) ? "true" : undefined}>
                        {hintLabel(h)}
                      </li>
                    ))}
                  </ul>
                </li>
              ),
            )}
          </ul>
          <p className="cr-help-note">En gras, l'indice que chacun avait vraiment.</p>
        </div>
      )}
      <HintHelp hints={riddle.mode_hints} />
      <details className="cr-help">
        <summary>Déroulement de la partie ({riddle.log.length} étapes)</summary>
        <ol className="cr-log">
          {riddle.log.map((e, i) => (
            <li key={i}>{describeEvent(e, names)}</li>
          ))}
        </ol>
      </details>
    </div>
  );
}

/** The riddles screen: a table of AIs plays until one of them knows. */
export function CryptideRiddles({ onBack }: { onBack(): void }) {
  const engine = useRef<EngineClient | null>(null);
  const [players, setPlayers] = useState(4);
  const [mode, setMode] = useState<Mode>("Normal");
  const [riddle, setRiddle] = useState<Riddle | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [misses, setMisses] = useState<Cell[]>([]);
  const [solved, setSolved] = useState<"found" | "shown" | null>(null);

  useEffect(() => {
    const client = new EngineClient();
    engine.current = client;
    next(4, "Normal");
    return () => {
      engine.current = null;
      client.dispose();
    };
    // The first riddle comes with the engine (`next` is stable).
  }, []);

  const next = useCallback(
    (n: number, m: Mode) => {
      const client = engine.current;
      if (!client) return;
      setBusy(true);
      setError(null);
      // A disposed engine (left the screen) answers nothing worth showing.
      const live = () => engine.current === client;
      client.cryptideRiddle<Riddle>(m, n, randomSeed()).then(
        (r) => {
          if (!live()) return;
          setRiddle(r);
          setMisses([]);
          setSolved(null);
          setBusy(false);
        },
        (e) => {
          if (!live()) return;
          setError(errorMessage(e));
          setBusy(false);
        },
      );
    },
    [],
  );

  const guess = (cell: Cell) => {
    if (!riddle || solved) return;
    if (cell === riddle.habitat) setSolved("found");
    else setMisses((m) => [...m, cell]);
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
        <p className="hint">
          Une table d'IA joue une partie. Dès que l'une d'elles peut déduire l'habitat, la partie s'arrête : tu reçois son
          indice, à toi de trouver la case.
        </p>
        <div className="row">
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
          <button className="primary" disabled={busy} onClick={() => next(players, mode)}>
            {riddle ? "Nouvelle énigme" : "Énigme"}
          </button>
        </div>
      </div>
      {error && <p className="error">{error}</p>}
      {busy && <p className="status">La table joue…</p>}
      {riddle && !busy && (
        <RiddleBoard riddle={riddle} misses={misses} solved={solved} onGuess={guess} onGiveUp={() => setSolved("shown")} />
      )}
    </main>
  );
}
