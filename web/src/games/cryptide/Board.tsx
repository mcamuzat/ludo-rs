import { useState } from "react";
import type { BoardProps } from "../registry";
import { HexMap, SEAT_FILL } from "./HexMap";
import { HintHelp } from "./HintHelp";
import { hintLabel } from "./hints";
import type { Cell, CryptideAction, CryptideEvent, CryptideView } from "./types";
import "./cryptide.css";

/** "ligne 3, colonne 5", counted from 1. */
export function place(cell: Cell): string {
  return `ligne ${Math.floor(cell / 12) + 1}, colonne ${(cell % 12) + 1}`;
}

/** The space an action is about. */
export function cellOf(a: CryptideAction): Cell {
  if ("Cube" in a) return a.Cube;
  if ("Ask" in a) return a.Ask.cell;
  if ("Search" in a) return a.Search;
  return a.Disc;
}

/** The legal actions on one space. */
export function actionsAt(legal: CryptideAction[], cell: Cell): CryptideAction[] {
  return legal.filter((a) => cellOf(a) === cell);
}

export function describeEvent(e: CryptideEvent, names: string[]): string {
  switch (e.type) {
    case "Asked":
      return `${names[e.by]} demande à ${names[e.target]} : ${place(e.cell)} ?`;
    case "Searched":
      return `${names[e.by]} fouille ${place(e.cell)}.`;
    case "Won":
      return `${names[e.seat]} a trouvé l'habitat !`;
    case "Placed": {
      const token = e.answer === "Disc" ? "un disque" : "un cube";
      const why = { Setup: " (partage initial)", Answer: "", Search: "", Penalty: " (pénalité)" }[e.why];
      return `${names[e.seat]} pose ${token} ${place(e.cell)}${why}.`;
    }
  }
}

function status(v: CryptideView, names: string[]): string {
  const who = names[v.turn];
  switch (v.phase.type) {
    case "Setup":
      return `Partage initial : ${who} pose un cube là où l'habitat ne peut pas être selon son indice.`;
    case "Turn":
      return `À ${who} d'interroger un joueur ou de fouiller.`;
    case "Relocate":
      return `${who} a déjà un disque sur la case fouillée : il pose son disque sur une autre case possible.`;
    case "Penalty":
      return `${who} pose un cube de pénalité, là où l'habitat ne peut pas être selon son indice.`;
    case "Over":
      return v.reveal ? "Partie terminée." : "";
  }
}

/** The buttons for one picked space during a turn. */
export function TurnControls({
  view,
  actions,
  names,
  submit,
}: {
  view: CryptideView;
  actions: CryptideAction[];
  names: string[];
  submit(a: CryptideAction): void;
}) {
  const asks = actions.filter((a): a is { Ask: { cell: Cell; target: number } } => "Ask" in a);
  const search = actions.find((a) => "Search" in a);
  return (
    <div className="cr-controls">
      {asks.map((a) => (
        <button key={a.Ask.target} type="button" onClick={() => submit(a)}>
          <span className="cr-swatch" style={{ background: SEAT_FILL[a.Ask.target] }} />
          Demander à {names[a.Ask.target]}
        </button>
      ))}
      {search && (
        <button type="button" className="primary" onClick={() => submit(search)}>
          Fouiller ici
        </button>
      )}
      {asks.length === 0 && !search && view.phase.type === "Turn" && <span>Rien à faire sur cette case.</span>}
    </div>
  );
}

export function CryptideBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as CryptideView;
  const legal = legalActions as CryptideAction[];
  const [selected, setSelected] = useState<Cell | null>(null);
  const playable = new Set(legal.map(cellOf));
  const direct = v.phase.type !== "Turn";
  const pick = (cell: Cell) => {
    const here = actionsAt(legal, cell);
    if (direct && here.length === 1) {
      submit(here[0]);
      setSelected(null);
    } else {
      setSelected(cell);
    }
  };
  const send = (a: CryptideAction) => {
    submit(a);
    setSelected(null);
  };
  const chosen = selected !== null && playable.has(selected) ? selected : null;
  return (
    <div className="cr-board">
      <ul className="legend">
        {seatNames.slice(0, v.players).map((name, seat) => (
          <li key={seat} className={v.turn === seat && v.phase.type !== "Over" ? "active" : undefined}>
            <span className="cr-swatch" style={{ background: SEAT_FILL[seat] }} />
            {name}
          </li>
        ))}
      </ul>
      <p className="cr-status">{status(v, seatNames)}</p>
      {v.my_hint && (
        <p className="cr-my-hint">
          <strong>Ton indice :</strong> {hintLabel(v.my_hint)}
        </p>
      )}
      <div className="cr-table felt">
        <HexMap
          squares={v.squares}
          tokens={v.tokens}
          playable={playable}
          selected={chosen}
          focus={v.focus}
          habitat={v.reveal?.habitat ?? null}
          onPick={pick}
        />
      </div>
      {chosen !== null && me !== null && v.phase.type === "Turn" && (
        <div className="cr-picked">
          <p>{place(chosen)}</p>
          <TurnControls view={v} actions={actionsAt(legal, chosen)} names={seatNames} submit={send} />
        </div>
      )}
      {v.reveal && (
        <div className="cr-reveal">
          <h3>Les indices</h3>
          <ul>
            {v.reveal.hints.map((h, seat) => (
              <li key={seat}>
                <span className="cr-swatch" style={{ background: SEAT_FILL[seat] }} />
                {seatNames[seat]} : {hintLabel(h)}
              </li>
            ))}
          </ul>
          <p>L'habitat était en {place(v.reveal.habitat)}.</p>
        </div>
      )}
      <HintHelp hints={v.hints} />
      <ol className="cr-log" reversed start={v.log.length}>
        {v.log
          .slice(-10)
          .reverse()
          .map((e, i) => (
            <li key={v.log.length - i}>{describeEvent(e, seatNames)}</li>
          ))}
      </ol>
    </div>
  );
}
