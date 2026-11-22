import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import type { BoardProps } from "../registry";
import { loadStats, recordResult } from "./stats";
import type { Color, FiveByFifteenAction, FiveByFifteenView, Pos, Route, Tile } from "./types";
import "./five-by-fifteen.css";

export const LINE_NAMES: Record<Color, string> = {
  Emerald: "Émeraude",
  Crimson: "Cramoisi",
  Azure: "Azur",
  Amber: "Ambre",
  Violet: "Violet",
};

const ROUTE_NAMES: Record<Route, string> = {
  Short: "Ligne courte",
  Medium: "Ligne moyenne",
  Long: "Ligne longue",
};

const same = (a: Pos, b: Pos) => a.row === b.row && a.col === b.col;

/** The legal action that takes the tile at `from` to the gap `to`: a placing, or the 15 leading its train. */
export function actionFor(legal: FiveByFifteenAction[], from: Pos, to: Pos): FiveByFifteenAction | null {
  for (const a of legal) {
    if (typeof a === "string") continue;
    if ("Place" in a && same(a.Place.from, from) && same(a.Place.to, to)) return a;
    if ("Shift" in a && a.Shift.row === to.row && a.Shift.to === to.col && same(from, { row: to.row, col: to.col - 1 }))
      return a;
  }
  return null;
}

/** Every move of the tile at `from`. */
function movesFrom(legal: FiveByFifteenAction[], from: Pos): { to: Pos; action: FiveByFifteenAction }[] {
  return legal.flatMap((a): { to: Pos; action: FiveByFifteenAction }[] => {
    if (typeof a === "string") return [];
    if ("Place" in a) return same(a.Place.from, from) ? [{ to: a.Place.to, action: a }] : [];
    return same(from, { row: a.Shift.row, col: a.Shift.to - 1 }) ? [{ to: { row: a.Shift.row, col: a.Shift.to }, action: a }] : [];
  });
}

/** Every tile that can go into the gap `to`. */
function sourcesOf(legal: FiveByFifteenAction[], to: Pos): Pos[] {
  return legal.flatMap((a) => {
    if (typeof a === "string") return [];
    if ("Place" in a) return same(a.Place.to, to) ? [a.Place.from] : [];
    return a.Shift.row === to.row && a.Shift.to === to.col ? [{ row: to.row, col: to.col - 1 }] : [];
  });
}

type Selection = { kind: "tile" | "gap"; pos: Pos } | null;

function cellAt(x: number, y: number): Pos | null {
  const el = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-row]");
  if (!el) return null;
  return { row: Number(el.dataset.row), col: Number(el.dataset.col) };
}

function outcomeText(v: FiveByFifteenView): string | null {
  switch (v.status) {
    case "Won":
      return `Les cinq trains sont formés : gagné en ${v.moves} coups !`;
    case "Stuck":
      return "Plus aucun coup possible : perdu.";
    case "GaveUp":
      return "Partie abandonnée.";
    default:
      return null;
  }
}

function Wagon({ tile, classes }: { tile: Tile; classes: string }) {
  return (
    <div className={`fx-tile fx-c-${tile.color.toLowerCase()} ${classes}`}>
      {tile.number}
      <span className="loco-chimney" />
      <span className="loco-puff" />
      <span className="loco-coal">
        <span />
        <span />
      </span>
      <span className="loco-cab" />
    </div>
  );
}

export function FiveByFifteenBoard({ view, legalActions, me, submit, assist }: BoardProps) {
  const v = view as FiveByFifteenView;
  const legal = legalActions as FiveByFifteenAction[];
  const playing = v.status === "Playing" && me !== null;
  const [selection, setSelection] = useState<Selection>(null);
  const [drag, setDrag] = useState<{ from: Pos; x: number; y: number; moved: boolean } | null>(null);
  const [stats, setStats] = useState(loadStats);
  const recorded = useRef(false);

  // One result per game: recorded when the game ends, reset by a new one.
  useEffect(() => {
    if (v.status === "Playing") {
      recorded.current = false;
      return;
    }
    if (recorded.current) return;
    recorded.current = true;
    recordResult(v.route, v.status === "Won");
    setStats(loadStats());
  }, [v.status, v.route]);

  useEffect(() => setSelection(null), [v.moves]);

  const play = (action: FiveByFifteenAction | null) => {
    if (!action || !playing) return;
    setSelection(null);
    submit(action);
  };

  const targets = selection?.kind === "tile" ? movesFrom(legal, selection.pos).map((m) => m.to) : [];
  const sources = selection?.kind === "gap" ? sourcesOf(legal, selection.pos) : [];

  const clickTile = (pos: Pos) => {
    if (!playing) return;
    if (selection?.kind === "gap" && sources.some((s) => same(s, pos))) return play(actionFor(legal, pos, selection.pos));
    setSelection(selection?.kind === "tile" && same(selection.pos, pos) ? null : { kind: "tile", pos });
  };
  const clickGap = (pos: Pos) => {
    if (!playing) return;
    if (selection?.kind === "tile" && targets.some((t) => same(t, pos))) return play(actionFor(legal, selection.pos, pos));
    setSelection(selection?.kind === "gap" && same(selection.pos, pos) ? null : { kind: "gap", pos });
  };
  const doubleClick = (pos: Pos) => {
    const moves = movesFrom(legal, pos);
    if (moves.length === 1) play(moves[0].action);
  };

  const startDrag = (pos: Pos, e: ReactPointerEvent) => {
    if (!playing || e.button !== 0) return;
    setDrag({ from: pos, x: e.clientX, y: e.clientY, moved: false });
  };
  useEffect(() => {
    if (!drag) return;
    const move = (e: PointerEvent) =>
      setDrag((d) => d && { ...d, x: e.clientX, y: e.clientY, moved: d.moved || Math.hypot(e.clientX - d.x, e.clientY - d.y) > 6 });
    const up = (e: PointerEvent) => {
      const d = drag;
      setDrag(null);
      if (!d.moved) return;
      const to = cellAt(e.clientX, e.clientY);
      if (to) play(actionFor(legal, d.from, to));
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up, { once: true });
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
  });

  const linked = new Set<string>();
  const ready = new Set<string>();
  for (const l of v.locomotives) {
    for (let c = l.col - 2; c <= l.col; c++) linked.add(`${l.row}:${c}`);
    if (l.ready) ready.add(`${l.row}:${l.col}`);
  }
  const dragged = drag?.moved ? v.cells[drag.from.row][drag.from.col] : null;
  const result = outcomeText(v);
  const done = stats[v.route];

  return (
    <div className="fx">
      <div className="fx-status">
        <span>
          {ROUTE_NAMES[v.route]} · train de {v.minRun} wagons ou plus
        </span>
        <span>{v.moves} coups</span>
      </div>
      {v.repeated && playing && (
        <p className="fx-warning">Cette position est déjà apparue : tu tournes peut-être en boucle.</p>
      )}
      <div className="fx-scroll">
        <div className="fx-grid" role="grid" aria-label="Dépôt de 5 voies">
          <div className="fx-row fx-rulers" aria-hidden="true">
            <div className="fx-ruler">1</div>
            {Array.from({ length: 15 }, (_, c) => (
              <div key={c} className="fx-ruler">
                {c < 14 ? c + 2 : ""}
              </div>
            ))}
          </div>
          {v.anchors.map((color, r) => (
            <div key={r} className="fx-row" role="row">
              <div className={`fx-anchor fx-c-${color.toLowerCase()}`} role="rowheader">
                <b>1</b>
                <small>{LINE_NAMES[color]}</small>
              </div>
              {v.cells[r].map((tile, c) => {
                const pos = { row: r, col: c };
                const key = `${r}:${c}`;
                const done = v.sorted[r] === 14;
                const cellClasses = [
                  "fx-cell",
                  c < v.sorted[r] ? "fx-placed" : "",
                  done && c === 14 ? "fx-locked" : "",
                  targets.some((t) => same(t, pos)) ? "fx-target" : "",
                  selection?.kind === "gap" && same(selection.pos, pos) ? "fx-selected" : "",
                ].join(" ");
                if (!tile) {
                  return (
                    <div
                      key={c}
                      className={cellClasses}
                      data-row={r}
                      data-col={c}
                      role="gridcell"
                      aria-label={`trou, voie ${r + 1}, colonne ${c + 1}`}
                      onClick={() => clickGap(pos)}
                    />
                  );
                }
                const tileClasses = [
                  linked.has(key) ? `loco-linked loco-${tile.number}` : "",
                  ready.has(key) ? "loco-ready" : "",
                  selection?.kind === "tile" && same(selection.pos, pos) ? "fx-picked" : "",
                  sources.some((s) => same(s, pos)) ? "fx-source" : "",
                  drag?.moved && same(drag.from, pos) ? "fx-dragging" : "",
                ].join(" ");
                return (
                  <div
                    key={c}
                    className={cellClasses}
                    data-row={r}
                    data-col={c}
                    role="gridcell"
                    aria-label={`${LINE_NAMES[tile.color]} ${tile.number}`}
                    onClick={() => clickTile(pos)}
                    onDoubleClick={() => doubleClick(pos)}
                    onPointerDown={(e) => startDrag(pos, e)}
                  >
                    <Wagon tile={tile} classes={tileClasses} />
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      </div>
      {dragged && drag && (
        <div className="fx-ghost" style={{ left: drag.x, top: drag.y }}>
          <Wagon tile={dragged} classes="" />
        </div>
      )}
      {result && <p className="status">{result}</p>}
      {playing && (
        <p className="hint">
          Clique un wagon pour voir où il peut aller, ou un trou pour voir qui peut y entrer ; ou fais glisser le wagon.
        </p>
      )}
      <div className="row">
        {assist && v.status === "Playing" && (
          <button type="button" disabled={v.moves === 0} onClick={() => assist.takeBack()}>
            Annuler
          </button>
        )}
        {playing && (
          <button type="button" className="ghost" onClick={() => play("GiveUp")}>
            Abandonner
          </button>
        )}
      </div>
      <p className="hint">
        {ROUTE_NAMES[v.route]} : {done.won} gagnée{done.won > 1 ? "s" : ""} sur {done.played} partie
        {done.played > 1 ? "s" : ""}.
      </p>
    </div>
  );
}
