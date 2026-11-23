import { useEffect, useState } from "react";
import type { BoardProps } from "../registry";
import type { Dir, Finish, HandDomino, PlacedView, WuHsingAction, WuHsingView } from "./types";
import {
  DOMINO_BODY,
  SEAT_COLORS,
  cellsOf,
  key,
  placementsOf,
  previewAt,
  previewClick,
  type Aid,
  type Cell,
} from "./interaction";
import "./wu-hsing.css";

export type { Aid } from "./interaction";

const AID_KEY = "wu-hsing.aid";
const AIDS: [Aid, string][] = [
  ["none", "aucune"],
  ["playable", "dominos jouables"],
  ["places", "emplacements"],
];

function readAid(): Aid {
  try {
    const v = localStorage.getItem(AID_KEY);
    if (v === "none" || v === "playable" || v === "places") return v;
  } catch {
    // Storage unavailable: the default.
  }
  return "places";
}

function saveAid(aid: Aid) {
  try {
    localStorage.setItem(AID_KEY, aid);
  } catch {
    // Storage unavailable: kept for this game only.
  }
}

const CELL = 40;
const MARGIN = 2;
/** Room above the grid for the points of a placement in the top row. */
const LABEL = 16;
/** The five elements: 1 Wood, 2 Fire, 3 Earth, 4 Metal, 5 Water. */
const VALUE_COLORS: Record<number, { bg: string; dot: string }> = {
  1: { bg: "#3f9142", dot: "#eaf6ea" },
  2: { bg: "#d1453b", dot: "#fff0ee" },
  3: { bg: "#d9a441", dot: "#2b1d05" },
  4: { bg: "#9fb0bb", dot: "#16202b" },
  5: { bg: "#34599c", dot: "#eaf1ff" },
};
/** Pip places in a 3×3 grid numbered 1 to 9. */
const PIPS: Record<number, number[]> = { 1: [5], 2: [1, 9], 3: [1, 5, 9], 4: [1, 3, 7, 9], 5: [1, 3, 5, 7, 9] };
const NEUTRAL = "#7a7a7a";


function listFr(items: (string | number)[]): string {
  if (items.length <= 1) return items.join("");
  return `${items.slice(0, -1).join(", ")} et ${items[items.length - 1]}`;
}

export function finishText(f: Finish, names: string[]): string {
  const best = Math.max(...f.totals);
  const winners = f.totals.flatMap((t, i) => (t === best ? [names[i]] : []));
  const detail = `(${f.totals.join(" contre ")}, pénalités ${listFr(f.penalties)})`;
  return winners.length === 1 ? `${winners[0]} gagne ${detail}.` : `Égalité entre ${listFr(winners)} ${detail}.`;
}

function Half({ x, y, value }: { x: number; y: number; value: number }) {
  const c = VALUE_COLORS[value];
  return (
    <g>
      <rect x={x + 2} y={y + 2} width={CELL - 4} height={CELL - 4} rx={5} fill={c.bg} />
      {PIPS[value].map((p) => (
        <circle
          key={p}
          cx={x + CELL * (0.25 + 0.25 * ((p - 1) % 3))}
          cy={y + CELL * (0.25 + 0.25 * Math.floor((p - 1) / 3))}
          r={CELL * 0.07}
          fill={c.dot}
        />
      ))}
    </g>
  );
}

/** A domino whose anchor's top-left corner is at (x, y). */
function Domino({
  x,
  y,
  dir,
  first,
  second,
  className,
  stroke,
}: {
  x: number;
  y: number;
  dir: Dir;
  first: number;
  second: number;
  className: string;
  stroke: string;
}) {
  const [dx, dy] = dir === "Right" ? [CELL, 0] : [0, CELL];
  return (
    <g className={className}>
      <rect x={x} y={y} width={CELL + dx} height={CELL + dy} rx={7} fill={DOMINO_BODY} stroke={stroke} strokeWidth={3} />
      <Half x={x} y={y} value={first} />
      <Half x={x + dx} y={y + dy} value={second} />
    </g>
  );
}

function HandTile({ d, selected, playable, onPick }: { d: HandDomino; selected: boolean; playable: boolean; onPick: () => void }) {
  return (
    <button
      type="button"
      className={`wh-hand-domino${playable ? " playable" : ""}${selected ? " selected" : ""}`}
      aria-pressed={selected}
      aria-label={`${d.a}-${d.b}${playable ? ", jouable" : ""}`}
      onClick={onPick}
    >
      <svg viewBox={`0 0 ${CELL * 2} ${CELL}`} width={CELL * 2} height={CELL}>
        <Domino x={0} y={0} dir="Right" first={d.a} second={d.b} className="" stroke={DOMINO_BODY} />
      </svg>
    </button>
  );
}

interface Props extends BoardProps {
  /** Overrides the saved aid (tests). */
  initialAid?: Aid;
}

export function WuHsingBoard({ view, me, seatNames, submit, initialAid }: Props) {
  const v = view as WuHsingView;
  const myTurn = me !== null && v.turn === me && !v.finish;
  const [aid, setAid] = useState<Aid>(() => initialAid ?? readAid());
  const [picked, setPicked] = useState<number | null>(null);
  const [cursor, setCursor] = useState<Cell | null>(null);
  const [dir, setDir] = useState<Dir>("Right");
  const [flipped, setFlipped] = useState(false);
  const [choice, setChoice] = useState(0);

  const legal = myTurn ? v.legal : [];
  const playable = (d: HandDomino) => placementsOf(v.hand, legal, d).length > 0;
  const helped = aid !== "none" && myTurn;

  // The domino in hand: the one picked, else (with help) the first playable, else the first.
  const selected =
    v.hand.find((d) => d.id === picked) ?? (helped ? v.hand.find(playable) : undefined) ?? v.hand[0] ?? null;
  const mine = selected ? placementsOf(v.hand, legal, selected) : [];
  const targets = new Set(aid === "places" ? mine.flatMap(({ p }) => cellsOf(p.row, p.col, p.dir).map(key)) : []);
  const preview =
    myTurn && selected && cursor ? previewAt(v.hand, legal, aid, selected, cursor, dir, flipped, choice) : null;

  const rotate = () => (aid === "places" ? setChoice((c) => c + 1) : setDir((d) => (d === "Right" ? "Down" : "Right")));
  const flip = () => (aid === "places" ? setChoice((c) => c + 1) : setFlipped((f) => !f));

  useEffect(() => {
    if (!myTurn) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "r" || e.key === "R") rotate();
      if (e.key === "f" || e.key === "F") flip();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // The grid: every laid cell, plus a margin.
  const laid = v.placed.flatMap((p) => cellsOf(p.row, p.col, p.dir));
  const rows = laid.map((c) => c[0]);
  const cols = laid.map((c) => c[1]);
  const top = Math.min(...rows) - MARGIN;
  const left = Math.min(...cols) - MARGIN;
  const height = Math.max(...rows) - top + 1 + MARGIN;
  const width = Math.max(...cols) - left + 1 + MARGIN;
  const at = (row: number, col: number) => ({ x: (col - left) * CELL, y: (row - top) * CELL });

  const pick = (d: HandDomino) => {
    setPicked(d.id);
    setChoice(0);
  };
  const touch = (cell: Cell) => {
    setCursor(cell);
    setChoice(0);
  };
  const confirm = () => {
    if (preview?.move) {
      submit(preview.move.action);
      setCursor(null);
      setPicked(null);
    }
  };
  const changeAid = (a: Aid) => {
    setAid(a);
    saveAid(a);
  };

  const lastIndex = v.last?.placed ?? null;
  const status = v.finish
    ? finishText(v.finish, seatNames)
    : v.last
      ? v.last.seat === me
        ? v.last.placed === null
          ? "Tu as passé."
          : `Tu as marqué ${v.last.points} points.`
        : v.last.placed === null
          ? `${seatNames[v.last.seat]} a passé.`
          : `${seatNames[v.last.seat]} a marqué ${v.last.points} points.`
      : "";

  return (
    <div className="wu-hsing">
      <ul className="legend">
        {v.scores.map((score, seat) => (
          <li key={seat} className={v.turn === seat ? "active" : undefined}>
            <span className="swatch" style={{ background: SEAT_COLORS[seat] }} />
            {`${seatNames[seat]} : ${v.finish ? v.finish.totals[seat] : score} points, ${v.handSizes[seat]} domino${v.handSizes[seat] > 1 ? "s" : ""}`}
          </li>
        ))}
      </ul>
      {v.lastRound && !v.finish && <p className="hint">Dernier tour de table.</p>}
      <div className="wh-scroll felt">
        <svg
          className="wh-board"
          viewBox={`0 ${-LABEL} ${width * CELL} ${height * CELL + LABEL}`}
          width={width * CELL}
          height={height * CELL + LABEL}
          role="img"
          aria-label="Plateau de Wu Hsing"
        >
          {Array.from({ length: height }, (_, i) =>
            Array.from({ length: width }, (_, j) => {
              const cell: Cell = [top + i, left + j];
              const target = targets.has(key(cell));
              return (
                <rect
                  key={key(cell)}
                  data-cell={key(cell)}
                  className={`wh-cell${target ? " target" : ""}`}
                  x={j * CELL}
                  y={i * CELL}
                  width={CELL}
                  height={CELL}
                  onClick={myTurn ? () => touch(cell) : undefined}
                />
              );
            }),
          )}
          {v.placed.map((p: PlacedView, i) => {
            const { x, y } = at(p.row, p.col);
            return (
              <Domino
                key={i}
                x={x}
                y={y}
                dir={p.dir}
                first={p.first}
                second={p.second}
                className={`wh-domino${i === lastIndex ? " last" : ""}`}
                stroke={p.by === null ? NEUTRAL : SEAT_COLORS[p.by]}
              />
            );
          })}
          {preview && (
            <g
              className={`wh-preview${preview.move ? " legal" : " illegal"}`}
              role={preview.move ? "button" : undefined}
              aria-label={preview.move ? `poser ici pour ${preview.move.points} points` : "pose illégale"}
            >
              <Domino
                {...at(preview.p.row, preview.p.col)}
                dir={preview.p.dir}
                first={preview.sides[0]}
                second={preview.sides[1]}
                className=""
                stroke={preview.move ? "var(--brass)" : "#ff4d4d"}
              />
              <text
                x={at(preview.p.row, preview.p.col).x + (preview.p.dir === "Right" ? CELL : CELL / 2)}
                y={at(preview.p.row, preview.p.col).y - 6}
                className="wh-points"
              >
                {preview.move ? `+${preview.move.points}` : "illégal"}
              </text>
              {cellsOf(preview.p.row, preview.p.col, preview.p.dir).map((c) => (
                <rect
                  key={key(c)}
                  className="wh-hit"
                  {...at(c[0], c[1])}
                  width={CELL}
                  height={CELL}
                  onClick={() => (cursor && previewClick(preview, c, cursor) === "confirm" ? confirm() : touch(c))}
                />
              ))}
            </g>
          )}
        </svg>
      </div>
      {status && <p className="status">{status}</p>}
      {me !== null && !v.finish && (
        <>
          <div className="wh-hand">
            {v.hand.map((d) => (
              <HandTile
                key={d.id}
                d={d}
                selected={myTurn && selected?.id === d.id}
                playable={helped && playable(d)}
                onPick={() => pick(d)}
              />
            ))}
          </div>
          {myTurn && (
            <>
              <div className="wh-tools">
                <button type="button" onClick={rotate}>
                  Tourner (R)
                </button>
                <button type="button" onClick={flip}>
                  Retourner (F)
                </button>
                <button type="button" onClick={() => submit("Pass" satisfies WuHsingAction)}>
                  Passer
                </button>
              </div>
              <p className="hint">
                {aid === "places"
                  ? "Choisis un domino, touche une case en surbrillance, puis touche l'aperçu pour poser."
                  : "Choisis un domino, touche une case, puis touche l'aperçu pour poser."}
              </p>
            </>
          )}
          <div className="wh-aid" role="group" aria-label="Aide à la pose">
            Aide :
            {AIDS.map(([a, label]) => (
              <button key={a} type="button" aria-pressed={aid === a} onClick={() => changeAid(a)}>
                {label}
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
