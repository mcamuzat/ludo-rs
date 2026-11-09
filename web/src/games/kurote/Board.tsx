import { useState } from "react";
import type { BoardProps } from "../registry";
import type { End, KuroteAction, KuroteView, Last } from "./types";
import "./kurote.css";

const COLORS = ["Vert", "Jaune"];
const SIZE = 5;
const STEP = 60;
/** Room on each side of the board for the wizards in reserve. */
const SIDE = 56;

type Selection = { from: number } | "reserve" | null;

/** Where a cell is drawn: the board is turned so that the viewer's end is at the bottom. */
function place(cell: number, flip: boolean) {
  const r = Math.floor(cell / SIZE);
  const c = cell % SIZE;
  const [dr, dc] = flip ? [SIZE - 1 - r, SIZE - 1 - c] : [r, c];
  return { x: SIDE + dc * STEP, y: dr * STEP };
}

function WizardShape({ cx, cy, owner, lying }: { cx: number; cy: number; owner: number; lying?: boolean }) {
  return (
    <g className={`kr-wizard kr-owner-${owner}${lying ? " exhausted" : ""}`} transform={`translate(${cx} ${cy}) rotate(${lying ? 90 : 0})`}>
      <path d="M -15 20 L 15 20 L 8 0 L -8 0 Z" />
      <circle cx={0} cy={-4} r={7} />
      <path d="M -12 -9 L 12 -9 L 3 -24 L 0 -27 Z" />
    </g>
  );
}

export function cellName(cell: number): string {
  return `${"abcde"[cell % SIZE]}${SIZE - Math.floor(cell / SIZE)}`;
}

export function lastText(last: Last, names: string[]): string {
  const who = names[last.seat];
  let text: string;
  if (last.action === "Pass") text = `${who} passe`;
  else if ("Place" in last.action) text = `${who} pose un sorcier de réserve en ${cellName(last.action.Place.to)}`;
  else text = `${who} téléporte un sorcier de ${cellName(last.action.Move.from)} à ${cellName(last.action.Move.to)}`;
  const n = last.captured.length;
  if (n > 0) text += ` et capture ${n === 1 ? "un sorcier" : `${n} sorciers`}`;
  return `${text}.`;
}

export function endText(end: End, names: string[]): string {
  if (end === "Blocked") return "Partie nulle : aucun des deux ne peut plus jouer.";
  if (end === "Limit") return "Partie nulle : 80 tours sans capture.";
  return `${names[end.Captures.winner]} gagne : trois sorciers capturés.`;
}

export function KuroteBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as KuroteView;
  const legal = legalActions as KuroteAction[];
  const myTurn = me !== null && v.turn === me && legal.length > 0;
  const [selection, setSelection] = useState<Selection>(null);
  const flip = me === 1;

  const moves = legal.flatMap((a) => (typeof a === "object" && "Move" in a ? [a.Move] : []));
  const places = legal.flatMap((a) => (typeof a === "object" && "Place" in a ? [a.Place.to] : []));
  const mustPass = legal.length === 1 && legal[0] === "Pass";
  const movable = new Set(moves.map((m) => m.from));
  const sel = myTurn ? selection : null;
  const targets = new Set(
    sel === "reserve" ? places : sel !== null ? moves.filter((m) => m.from === sel.from).map((m) => m.to) : [],
  );

  const clickCell = (cell: number) => {
    if (!myTurn) return;
    if (targets.has(cell) && sel !== null) {
      submit(sel === "reserve" ? { Place: { to: cell } } : { Move: { from: sel.from, to: cell } });
      setSelection(null);
    } else if (movable.has(cell)) {
      setSelection(sel !== null && sel !== "reserve" && sel.from === cell ? null : { from: cell });
    } else {
      setSelection(null);
    }
  };

  const lastAction = v.last?.action;
  const lastCells = new Set<number>(
    !lastAction || lastAction === "Pass" ? [] : "Move" in lastAction ? [lastAction.Move.from, lastAction.Move.to] : [lastAction.Place.to],
  );
  const width = SIDE * 2 + SIZE * STEP;
  const height = SIZE * STEP;

  // The reserve of the seat whose end is at the bottom goes on the right, the other on the left.
  const bottomSeat = flip ? 1 : 0;
  const reserveSlots = (seat: number) => {
    const x = seat === bottomSeat ? SIDE + SIZE * STEP + SIDE / 2 : SIDE / 2;
    return Array.from({ length: v.reserve[seat] }, (_, k) => ({
      x,
      y: seat === bottomSeat ? height - STEP / 2 - k * STEP : STEP / 2 + k * STEP,
    }));
  };
  const canPlace = myTurn && places.length > 0;

  return (
    <div className="kurote">
      <ul className="legend">
        {[0, 1].map((seat) => (
          <li key={seat} className={v.turn === seat ? "active" : undefined}>
            <span className={`swatch kr-owner-${seat}`} />
            {`${COLORS[seat]} : ${seatNames[seat]} — ${v.captured[seat]}/3 captures`}
          </li>
        ))}
      </ul>
      <svg className="kr-board" viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Plateau de Kurote">
        {v.cells.map((w, cell) => {
          const { x, y } = place(cell, flip);
          const dark = (Math.floor(cell / SIZE) + (cell % SIZE)) % 2 === 1;
          const target = targets.has(cell);
          const pickable = myTurn && movable.has(cell);
          const selected = sel !== null && sel !== "reserve" && sel.from === cell;
          const captured = v.last?.captured.includes(cell) ?? false;
          return (
            <g
              key={cell}
              className={`kr-cell${target ? " target" : ""}${pickable ? " pickable" : ""}${selected ? " selected" : ""}`}
              onClick={myTurn ? () => clickCell(cell) : undefined}
              role={target || pickable ? "button" : undefined}
              aria-label={target ? `aller en ${cellName(cell)}` : pickable ? `sorcier en ${cellName(cell)}` : undefined}
            >
              <rect x={x} y={y} width={STEP} height={STEP} className={dark ? "kr-dark" : "kr-light"} />
              {lastCells.has(cell) && <rect x={x + 3} y={y + 3} width={STEP - 6} height={STEP - 6} className="kr-last" />}
              {target && <circle cx={x + STEP / 2} cy={y + STEP / 2} r={9} className="kr-dot" />}
              {w && <WizardShape cx={x + STEP / 2} cy={y + STEP / 2} owner={w.owner} lying={w.exhausted} />}
              {captured && !w && (
                <path
                  d={`M ${x + 16} ${y + 16} L ${x + STEP - 16} ${y + STEP - 16} M ${x + STEP - 16} ${y + 16} L ${x + 16} ${y + STEP - 16}`}
                  className="kr-cross"
                />
              )}
            </g>
          );
        })}
        {[0, 1].map((seat) =>
          reserveSlots(seat).map(({ x, y }, k) => {
            const mine = canPlace && seat === me;
            return (
              <g
                key={`${seat}-${k}`}
                className={`kr-reserve${mine ? " pickable" : ""}${mine && sel === "reserve" ? " selected" : ""}`}
                onClick={mine ? () => setSelection(sel === "reserve" ? null : "reserve") : undefined}
                role={mine ? "button" : undefined}
                aria-label={mine ? "sorcier de réserve" : undefined}
              >
                <rect x={x - SIDE / 2 + 4} y={y - STEP / 2 + 4} width={SIDE - 8} height={STEP - 8} rx={8} className="kr-slot" />
                <WizardShape cx={x} cy={y} owner={seat} />
              </g>
            );
          }),
        )}
      </svg>
      {v.end ? (
        <p className="status">{endText(v.end, seatNames)}</p>
      ) : (
        <>
          {myTurn && mustPass && (
            <button type="button" onClick={() => submit("Pass" satisfies KuroteAction)}>
              Passer (aucun coup possible)
            </button>
          )}
          {myTurn && !mustPass && (
            <p className="hint">
              {sel === "reserve"
                ? "Choisis une case libre de ta première rangée."
                : sel !== null
                  ? "Choisis où le téléporter."
                  : canPlace
                    ? "Choisis un sorcier à téléporter, ou un sorcier de réserve à poser."
                    : "Choisis un sorcier à téléporter."}
            </p>
          )}
          {v.quietLeft <= 20 && <p className="hint">{`Nulle dans ${v.quietLeft} tours sans capture.`}</p>}
        </>
      )}
      {v.last && <p className="hint">{lastText(v.last, seatNames)}</p>}
    </div>
  );
}
