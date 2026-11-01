import { useState } from "react";
import type { BoardProps } from "../registry";
import type { BlackHoleAction, BlackHoleView, Finish } from "./types";
import "./black-hole.css";

const COLORS = ["Rouge", "Bleu"];
const STEP = 54;
const RADIUS = 24;

/** Row and place in the row of a cell numbered from the apex. */
function rowCol(cell: number): [number, number] {
  let row = 0;
  while (((row + 1) * (row + 2)) / 2 <= cell) row++;
  return [row, cell - (row * (row + 1)) / 2];
}

function center(cell: number) {
  const [row, col] = rowCol(cell);
  return { x: (col - row / 2) * STEP, y: row * STEP * 0.87 };
}

export function finishText(f: Finish, names: string[]): string {
  const [red, blue] = f.sums;
  const score = `${red} contre ${blue}`;
  if (red === blue) return `Partie nulle : ${score}.`;
  return `${names[red < blue ? 0 : 1]} gagne (${red < blue ? `${red} contre ${blue}` : `${blue} contre ${red}`}).`;
}

export function BlackHoleBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as BlackHoleView;
  const legal = legalActions as BlackHoleAction[];
  const myTurn = me !== null && v.turn === me && legal.length > 0;
  const free = v.order === "Free";
  const [picked, setPicked] = useState<number | null>(null);
  const myTiles = me !== null ? v.remaining[me] : [];
  // In the free order, the tile picked (or the smallest); else the one to lay.
  const tile = free ? (picked !== null && myTiles.includes(picked) ? picked : (myTiles[0] ?? null)) : v.next;
  const playable = (cell: number) =>
    myTurn && legal.some((a) => a.Place.cell === cell && a.Place.tile === tile);

  return (
    <div className="black-hole">
      <ul className="legend">
        {[0, 1].map((seat) => (
          <li key={seat} className={v.turn === seat ? "active" : undefined}>
            <span className={`swatch bh-owner-${seat}`} />
            {`${COLORS[seat]} : ${seatNames[seat]}`}
            {v.finish && ` — ${v.finish.sums[seat]} autour du trou`}
          </li>
        ))}
      </ul>
      <svg className="bh-board felt" viewBox="-170 -40 340 320" role="img" aria-label="Plateau de Black Hole">
        {v.cells.map((t, cell) => {
          const { x, y } = center(cell);
          const hole = v.finish?.hole === cell;
          const around = v.around.includes(cell);
          const canPlay = playable(cell);
          return (
            <g
              key={cell}
              className={`bh-cell${canPlay ? " playable" : ""}${around ? " around" : ""}`}
              onClick={canPlay && tile !== null ? () => submit({ Place: { cell, tile } } satisfies BlackHoleAction) : undefined}
              aria-label={canPlay ? `case ${cell + 1}` : undefined}
              role={canPlay ? "button" : undefined}
            >
              <circle cx={x} cy={y} r={RADIUS} className={hole ? "bh-hole" : "bh-slot"} />
              {t && (
                <>
                  <circle cx={x} cy={y} r={RADIUS - 2} className={`bh-tile bh-owner-${t.owner}`} />
                  <text x={x} y={y + 1} className={`bh-value${t.value === 6 || t.value === 9 ? " underline" : ""}`}>
                    {t.value}
                  </text>
                </>
              )}
              {v.last === cell && !v.finish && <circle cx={x} cy={y + RADIUS - 6} r={3} className="bh-last" />}
            </g>
          );
        })}
      </svg>
      {v.finish ? (
        <p className="status">{finishText(v.finish, seatNames)}</p>
      ) : (
        <>
          {free && myTurn && (
            <div className="bh-tiles">
              {myTiles.map((t) => (
                <button
                  key={t}
                  type="button"
                  className={`bh-pick bh-owner-${me}${t === tile ? " selected" : ""}`}
                  aria-pressed={t === tile}
                  onClick={() => setPicked(t)}
                >
                  {t}
                </button>
              ))}
            </div>
          )}
          {myTurn && (
            <p className="hint">
              {free ? `Choisis une tuile, puis une case : tu poses le ${tile}.` : `Pose ton ${tile} sur une case vide.`}
            </p>
          )}
          {!myTurn && v.turn !== null && v.next !== null && (
            <p className="hint">{`${seatNames[v.turn]} pose son ${v.next}.`}</p>
          )}
          <p className="hint">
            {[0, 1].map((seat) => `${COLORS[seat]} : reste ${v.remaining[seat].join(", ") || "rien"}`).join(" · ")}
          </p>
        </>
      )}
    </div>
  );
}
