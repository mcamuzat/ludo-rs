import { useState } from "react";
import type { BoardProps } from "../registry";
import boardJson from "./board.json";
import type { BantuAction, BantuView, BoardData, Dest, End, LastMove, Spot } from "./types";

const BOARD = boardJson as BoardData;

/** Seat colours, as in the rules' diagrams. */
const COLOURS = ["#d23b2f", "#2e9b4a", "#2f6fd6", "#e3b821"];
const COLOUR_NAMES = ["rouge", "vert", "bleu", "jaune"];

/** Start squares in the centre, by piece number (line 1 at the bottom). */
const SQUARES = [
  { x: 500, y: 610 },
  { x: 610, y: 500 },
  { x: 500, y: 390 },
  { x: 390, y: 500 },
];
const HOME_OFFSETS = [
  [-24, -24],
  [24, -24],
  [-24, 24],
  [24, 24],
];

const CELL_R = 20;

function sameDest(a: Dest, b: Dest): boolean {
  return a === "Finish" || b === "Finish" ? a === b : a.Cell === b.Cell;
}

/** Where piece `piece` of the player to move may go. */
export function destinations(piece: number, legal: BantuAction[]): Dest[] {
  return legal.filter((a) => a.Move.piece === piece).map((a) => a.Move.to);
}

function elide(name: string): string {
  return /^[aeiouyhàâéèêëîïôöûüAEIOUYHÀÂÉÈÊËÎÏÔÖÛÜ]/.test(name) ? `d'${name}` : `de ${name}`;
}

export function describeLast(last: LastMove, names: string[]): string {
  const who = names[last.seat];
  if (last.to.kind === "Finished") return `${who} amène son pion ${last.piece} à l'arrivée.`;
  const verb = last.from.kind === "Home" ? "sort" : "avance";
  const taken = last.captured
    ? ` et renvoie le pion ${last.captured[1]} ${elide(names[last.captured[0]])} au départ`
    : "";
  return `${who} ${verb} son pion ${last.piece}${taken}.`;
}

export function endText(end: End, names: string[]): string {
  if (end === "QuietLimit")
    return "Arrêt : 30 tours de table sans progrès. Classement par pions arrivés, puis par chemin restant.";
  if (end === "Blocked") return "Arrêt : plus personne ne peut jouer. Classement par pions arrivés, puis par chemin restant.";
  return `${names[end.Won]} a ramené tous ses pions : partie gagnée.`;
}

function spotKey(spot: Spot): string {
  return spot.kind === "On" ? `c${spot.cell}` : spot.kind;
}

export function BantuBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as BantuView;
  const legal = legalActions as BantuAction[];
  // A selection is kept only for the decision it was made for.
  const key = JSON.stringify(legal);
  const [chosen, setChosen] = useState<{ key: string; piece: number | null }>({ key, piece: null });
  const selected = chosen.key === key ? chosen.piece : null;
  const targets = selected === null ? [] : destinations(selected, legal);
  const target = (d: Dest) => targets.some((t) => sameDest(t, d));
  const play = (to: Dest) => {
    if (selected === null) return;
    setChosen({ key, piece: null });
    submit({ Move: { piece: selected, to } } satisfies BantuAction);
  };
  const lastTo = v.last ? spotKey(v.last.to) : null;

  const pieceAt = (seat: number, index: number, x: number, y: number, r: number) => {
    const piece = index + 1;
    const movable = seat === me && destinations(piece, legal).length > 0;
    const name = `Pion ${piece} ${elide(seatNames[seat])}`;
    return (
      <g
        key={`p${seat}-${piece}`}
        className={movable ? "piece movable" : "piece"}
        aria-label={name}
        onClick={movable ? () => setChosen({ key, piece: selected === piece ? null : piece }) : undefined}
      >
        <title>{name}</title>
        <circle
          cx={x}
          cy={y}
          r={r}
          fill={COLOURS[seat]}
          className={selected === piece && movable ? "piece-disc selected" : "piece-disc"}
        />
        <text x={x} y={y + r * 0.38} textAnchor="middle" fontSize={r * 1.1} className="piece-number">
          {piece}
        </text>
      </g>
    );
  };

  const onBoard = v.pieces.flatMap((pieces, seat) =>
    pieces.flatMap((spot, i) => {
      if (spot.kind !== "On") return [];
      const c = BOARD.cells[spot.cell];
      return [pieceAt(seat, i, c.x, c.y, 17)];
    }),
  );
  const atHome = v.pieces.flatMap((pieces, seat) =>
    pieces.flatMap((spot, i) => {
      if (spot.kind !== "Home") return [];
      const [dx, dy] = HOME_OFFSETS[seat];
      return [pieceAt(seat, i, SQUARES[i].x + dx, SQUARES[i].y + dy, 13)];
    }),
  );

  const status = v.end
    ? endText(v.end, seatNames)
    : v.turn === me
      ? "À toi : choisis un pion, puis sa destination."
      : v.turn !== null
        ? `${seatNames[v.turn]} réfléchit…`
        : "";

  return (
    <div className="bantu">
      <ul className="legend">
        {v.pieces.map((pieces, seat) => (
          <li key={seat} className={v.turn === seat ? "active" : undefined}>
            <span className="swatch" style={{ background: COLOURS[seat] }} />
            {`${seatNames[seat]} (${COLOUR_NAMES[seat]}) : ${pieces.filter((p) => p.kind === "Finished").length}/${pieces.length} arrivés`}
          </li>
        ))}
      </ul>
      <svg className="bantu-board felt" viewBox="0 0 1000 1000" role="img" aria-label="Plateau de Bantu">
        <defs>
          <marker id="bantu-head" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="4" markerHeight="4" orient="auto">
            <path d="M0,0 L10,5 L0,10 z" className="bantu-head" />
          </marker>
        </defs>
        <circle cx={500} cy={500} r={205} className="bantu-disc" />
        {SQUARES.map((s, i) => (
          <g key={i}>
            <rect x={s.x - 34} y={s.y - 34} width={68} height={68} transform={`rotate(45 ${s.x} ${s.y})`} className="bantu-square" />
            <text x={s.x} y={s.y + 12} textAnchor="middle" className="bantu-square-number">
              {i + 1}
            </text>
          </g>
        ))}
        {BOARD.links.map(([a, b], i) => {
          const p = BOARD.cells[a];
          const q = BOARD.cells[b];
          const len = Math.hypot(q.x - p.x, q.y - p.y);
          const k = 1 - (CELL_R + 2) / len;
          return (
            <line
              key={`l${i}`}
              x1={p.x}
              y1={p.y}
              x2={p.x + (q.x - p.x) * k}
              y2={p.y + (q.y - p.y) * k}
              className="bantu-link"
              markerEnd="url(#bantu-head)"
            />
          );
        })}
        {BOARD.rows.map((row, i) => (
          <polyline key={`r${i}`} points={row.map((c) => `${BOARD.cells[c].x},${BOARD.cells[c].y}`).join(" ")} className="bantu-row" />
        ))}
        {BOARD.startLines.map((line, i) => (
          <polyline
            key={`s${i}`}
            points={line.map((c) => `${BOARD.cells[c].x},${BOARD.cells[c].y}`).join(" ")}
            className="bantu-line"
          />
        ))}
        <g
          className={target("Finish") ? "bantu-finish target" : "bantu-finish"}
          onClick={target("Finish") ? () => play("Finish") : undefined}
        >
          <polygon points="28,528 288,512 292,538 34,566" />
          <text x={60} y={553} transform="rotate(-4 60 553)">
            ARRIVÉE
          </text>
        </g>
        {BOARD.cells.map((c, i) => {
          const dest = target({ Cell: i });
          const cls = ["bantu-cell", c.arrow ? "arrow" : "", dest ? "target" : "", lastTo === `c${i}` ? "last" : ""]
            .filter(Boolean)
            .join(" ");
          return (
            <circle
              key={i}
              data-cell={i}
              cx={c.x}
              cy={c.y}
              r={CELL_R}
              className={cls}
              onClick={dest ? () => play({ Cell: i }) : undefined}
            />
          );
        })}
        {atHome}
        {onBoard}
        {v.protected.map((c) => (
          <circle key={`g${c}`} cx={BOARD.cells[c].x} cy={BOARD.cells[c].y} r={CELL_R + 3} className="bantu-guard" />
        ))}
      </svg>
      <p className="status">{status}</p>
      {me !== null && v.turn === me && !v.end && (
        <p className="hint">
          {`Ce tour, tes pions avancent de : ${v.lengths
            .map((n, i) => (n > 0 ? `${i + 1} → ${n}` : null))
            .filter(Boolean)
            .join(", ")}.`}
        </p>
      )}
      {v.last && <p className="hint">{describeLast(v.last, seatNames)}</p>}
      {!v.end && <p className="hint">{`Limite : encore ${v.quietLeft} tours de table sans progrès avant l'arrêt.`}</p>}
    </div>
  );
}
