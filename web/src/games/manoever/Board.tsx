import { useState } from "react";
import type { BoardProps } from "../registry";
import boardJson from "./board.json";
import type { BoardData, Dest, ManoeverAction, ManoeverView, MoveRecord, Mover, Soldier } from "./types";

const BOARD = boardJson as BoardData;

/** Seat colours, as the rules' tokens: red, yellow, blue, green. */
const COLOURS = ["#d23b2f", "#e3b821", "#2f6fd6", "#2e9b4a"];
const COLOUR_NAMES = ["rouge", "jaune", "bleu", "vert"];
const ROMAN = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII"];

const CELL_R = 30;
const SOLDIER_R = 19;
/** How far up each level of a stack is drawn. */
const LEVEL_DY = 10;

function sameDest(a: Dest, b: Dest): boolean {
  return a === "Barracks" || b === "Barracks" ? a === b : a.Square === b.Square;
}

function ordinal(n: number): string {
  return n === 1 ? "1re" : `${n}e`;
}

/** « 4e × étage 2 × noire = 24 »: how the engine made the maximum. */
export function describeMax(m: Mover): string {
  const bonus = m.bonus === 3 ? " × noire" : m.bonus === 2 ? " × blanche" : "";
  return `${ordinal(m.position)} × étage ${m.height}${bonus} = ${m.max}`;
}

/** The selected soldier, if the view still asks `me` to move it. */
export function selectionAfter(v: ManoeverView, me: number | null, selected: number | null): number | null {
  if (selected === null || v.turn === null || v.turn !== me) return null;
  return v.toMove.some((m) => m.number === selected) ? selected : null;
}

/**
 * What a click on soldier `s` does: select it (one of my soldiers still to
 * move, even on a square the selected soldier could reach, so switching
 * selection never plays a move by accident), play the selected soldier onto
 * its square (`dest`), or nothing.
 */
export function soldierClick(
  v: ManoeverView,
  me: number | null,
  selected: number | null,
  s: Soldier,
  top: boolean,
  dest: boolean,
): "select" | "play" | null {
  const movable = top && s.seat === me && v.turn === me && v.toMove.some((m) => m.number === s.number);
  if (movable && s.number !== selected) return "select";
  if (dest) return "play";
  return movable ? "select" : null;
}

function elide(name: string): string {
  return /^[aeiouyhàâéèêëîïôöûüAEIOUYHÀÂÉÈÊËÎÏÔÖÛÜ]/.test(name) ? `d'${name}` : `de ${name}`;
}

function describeMove(r: MoveRecord): string {
  const to = r.to === "Barracks" ? `caserne (place ${ROMAN[(r.place ?? 1) - 1]})` : `${r.to.Square}`;
  const gray = r.gray ? `, pièce grise de ${r.gray}` : "";
  return `soldat ${r.number}, ${r.from} → ${to}${gray}`;
}

function endText(v: ManoeverView, names: string[]): string {
  const best = Math.max(...v.scores);
  const winners = v.scores.flatMap((s, i) => (s === best ? [names[i]] : []));
  return winners.length === 1
    ? `${winners[0]} gagne avec ${best} points.`
    : `Égalité à ${best} points : ${winners.join(" et ")}.`;
}

export function ManoeverBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as ManoeverView;
  const legal = legalActions as ManoeverAction[];
  const [picked, setPicked] = useState<number | null>(null);
  const selected = selectionAfter(v, me, picked);
  const mover = v.toMove.find((m) => m.number === selected) ?? null;
  const targets = mover?.dests ?? [];
  const target = (d: Dest) => targets.some((t) => sameDest(t, d));
  const canRoll = legal.includes("Roll");
  const play = (to: Dest) => {
    if (selected === null) return;
    setPicked(null);
    submit({ Move: { soldier: selected, to } } satisfies ManoeverAction);
  };
  const mine = (n: number) => v.turn === me && v.toMove.some((m) => m.number === n);

  // Squares reached in each player's last turn (and this one so far).
  const landed = new Set<number>();
  for (const r of [...v.lastTurns.flat(), ...v.turnLog]) if (r.to !== "Barracks") landed.add(r.to.Square);

  const track = BOARD.squares.map((p) => `${p.x},${p.y}`).join(" ");
  const discAt = new Map(BOARD.discs.map((d) => [d.square, d.kind]));
  const col = BOARD.barracks;
  const barracksBox = {
    x: col[0].x - 46,
    y: col[0].y - 44,
    w: 92,
    h: col[11].y - col[0].y + 88,
  };

  const status = v.over
    ? endText(v, seatNames)
    : v.turn === null
      ? ""
      : v.turn === me
        ? v.setup
          ? "À toi : lance le dé pour sortir un soldat."
          : "À toi : choisis un soldat, puis sa case."
        : `${seatNames[v.turn]} ${v.setup ? "lance le dé" : "manœuvre"}…`;

  return (
    <div className="manoever">
      <ul className="legend">
        {v.scores.map((score, seat) => (
          <li key={seat} className={v.turn === seat ? "active" : undefined}>
            <span className="swatch" style={{ background: COLOURS[seat] }} />
            {`${seatNames[seat]} (${COLOUR_NAMES[seat]}) : ${score} pt`}
            {v.setup && v.waiting[seat] > 0 ? ` · ${v.waiting[seat]} à sortir` : ""}
          </li>
        ))}
      </ul>
      <svg
        className="manoever-board felt"
        viewBox={`0 0 ${BOARD.width} ${BOARD.height}`}
        role="img"
        aria-label="Plateau de Das glorreiche Manöver"
      >
        <polyline points={track} className="manoever-track" />
        <rect
          x={barracksBox.x}
          y={barracksBox.y}
          width={barracksBox.w}
          height={barracksBox.h}
          rx={18}
          className={target("Barracks") ? "manoever-barracks target" : "manoever-barracks"}
          onClick={target("Barracks") ? () => play("Barracks") : undefined}
        />
        {col.map((p, i) => {
          const s = v.barracks[i];
          return (
            <g key={`b${i}`} className="manoever-place" onClick={target("Barracks") ? () => play("Barracks") : undefined}>
              <rect x={p.x - 32} y={p.y - 30} width={64} height={60} rx={10} />
              {s ? (
                <g>
                  <circle cx={p.x} cy={p.y} r={SOLDIER_R} fill={COLOURS[s.seat]} className="soldier-disc" />
                  <text x={p.x} y={p.y + 7} textAnchor="middle" className="soldier-number">
                    {s.number}
                  </text>
                </g>
              ) : (
                <text x={p.x} y={p.y + 9} textAnchor="middle" className="manoever-roman">
                  {ROMAN[i]}
                </text>
              )}
            </g>
          );
        })}
        {BOARD.squares.map((p, i) => {
          const sq = i + 1;
          const dest = target({ Square: sq });
          const cls = [
            "manoever-cell",
            BOARD.endSquares.includes(sq) ? "end" : "",
            dest ? "target" : "",
            landed.has(sq) ? "last" : "",
          ]
            .filter(Boolean)
            .join(" ");
          return (
            <g key={`c${sq}`} onClick={dest ? () => play({ Square: sq }) : undefined}>
              <circle cx={p.x} cy={p.y} r={CELL_R} className={cls} />
            </g>
          );
        })}
        {v.grays
          .filter((g) => g.takenBy === null)
          .map((g) => {
            const p = BOARD.squares[g.square - 1];
            return (
              <g key={`g${g.square}`} className="manoever-gray" pointerEvents="none">
                <circle cx={p.x} cy={p.y} r={16} />
                <text x={p.x} y={p.y + 6} textAnchor="middle">
                  {g.value}
                </text>
              </g>
            );
          })}
        {v.stacks.map((stack, i) => {
          const sq = i + 1;
          const p = BOARD.squares[i];
          const kind = discAt.get(sq);
          const base = kind ? 1 : 0;
          const dest = target({ Square: sq });
          return (
            <g key={`s${sq}`} onClick={dest ? () => play({ Square: sq }) : undefined}>
              {kind && (
                <circle cx={p.x} cy={p.y} r={22} className={kind === "Black" ? "manoever-disc black" : "manoever-disc white"} />
              )}
              {stack.map((s, level) => {
                const y = p.y - (base + level) * LEVEL_DY;
                const top = level === stack.length - 1;
                const click = soldierClick(v, me, selected, s, top, dest);
                const movable = top && s.seat === me && mine(s.number);
                const name = `Soldat ${s.number} (${COLOUR_NAMES[s.seat]})`;
                return (
                  <g
                    key={`${s.seat}-${s.number}`}
                    data-square={sq}
                    className={movable ? "soldier movable" : "soldier"}
                    onClick={
                      click === "select"
                        ? (e) => {
                            e.stopPropagation();
                            setPicked(selected === s.number ? null : s.number);
                          }
                        : undefined
                    }
                  >
                    <title>{name}</title>
                    <circle
                      cx={p.x}
                      cy={y}
                      r={SOLDIER_R}
                      fill={COLOURS[s.seat]}
                      className={movable && selected === s.number ? "soldier-disc selected" : "soldier-disc"}
                    />
                    <text x={p.x} y={y + 7} textAnchor="middle" className="soldier-number">
                      {s.number}
                    </text>
                  </g>
                );
              })}
            </g>
          );
        })}
        {BOARD.squares.map((p, i) => (
          <text key={`n${i}`} x={p.x + CELL_R - 2} y={p.y + CELL_R - 2} textAnchor="middle" className="manoever-cell-number">
            {i + 1}
          </text>
        ))}
      </svg>
      <p className="status">{status}</p>
      {canRoll && (
        <button type="button" className="primary" onClick={() => submit("Roll" satisfies ManoeverAction)}>
          Lancer le dé
        </button>
      )}
      {v.setup && v.lastRoll && (
        <p className="hint">{`${seatNames[v.lastRoll[0]]} a sorti un soldat sur la case ${v.lastRoll[1]}.`}</p>
      )}
      {mover && <p className="hint">{`Soldat ${mover.number} : jusqu'à ${describeMax(mover)} cases.`}</p>}
      {v.turn === me && !v.setup && !v.over && v.toMove.length > 0 && (
        <p className="hint">{`À déplacer ce tour : ${v.toMove.map((m) => m.number).join(", ")}.`}</p>
      )}
      {v.lastTurns.map((moves, seat) =>
        moves.length > 0 ? (
          <p key={seat} className="hint">
            {`Dernier tour ${elide(seatNames[seat])} : ${moves.map(describeMove).join(" ; ")}.`}
          </p>
        ) : null,
      )}
    </div>
  );
}
