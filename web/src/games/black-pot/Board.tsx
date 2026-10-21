import { useState } from "react";
import type { BoardProps } from "../registry";
import boardJson from "./board.json";
import type { BlackPotAction, BlackPotView, BoardData, End, Expr, Group, Op, TurnRecord } from "./types";
import "./black-pot.css";

const BOARD = boardJson as BoardData;
const SIZE = 60;
const SIGNS: Record<Op, string> = { Add: "+", Sub: "−", Mul: "×", Div: "÷" };
const OPS: Op[] = ["Add", "Sub", "Mul", "Div"];
const TEAM_NAMES = ["ivoire", "marron"];
const PIPS: Record<number, [number, number][]> = {
  1: [[0, 0]],
  2: [[-1, -1], [1, 1]],
  3: [[-1, -1], [0, 0], [1, 1]],
  4: [[-1, -1], [1, -1], [-1, 1], [1, 1]],
  5: [[-1, -1], [1, -1], [0, 0], [-1, 1], [1, 1]],
  6: [[-1, -1], [1, -1], [-1, 0], [1, 0], [-1, 1], [1, 1]],
};

/** The move being composed: groups already placed, the calculation in
 * progress, and an operation waiting for its second number. */
export interface Draft {
  groups: Group[];
  current: Expr | null;
  pendingOp: Op | null;
}

export const EMPTY_DRAFT: Draft = { groups: [], current: null, pendingOp: null };

function diceOf(e: Expr): number[] {
  return "Die" in e ? [e.Die] : [...diceOf(e.Op.left), ...diceOf(e.Op.right)];
}

export function usedDice(d: Draft): number[] {
  return [...d.groups.flatMap((g) => diceOf(g.expr)), ...(d.current ? diceOf(d.current) : [])].sort();
}

/** A die: the first number of a calculation, or the second after an operation. */
export function clickDie(d: Draft, i: number): Draft {
  if (usedDice(d).includes(i)) return d;
  if (d.current === null) return { ...d, current: { Die: i } };
  if (d.pendingOp === null) return d;
  return { ...d, current: { Op: { op: d.pendingOp, left: d.current, right: { Die: i } } }, pendingOp: null };
}

export function clickOp(d: Draft, op: Op): Draft {
  return d.current === null ? d : { ...d, pendingOp: op };
}

/** Swaps the two sides of the last operation: 6 ÷ (4 − 2) from (4 − 2) ÷ 6. */
export function swap(d: Draft): Draft {
  if (d.current === null || !("Op" in d.current) || d.pendingOp !== null) return d;
  const { op, left, right } = d.current.Op;
  return { ...d, current: { Op: { op, left: right, right: left } } };
}

/** Ends the calculation on `cell`, or on « = 0 » (`null`). */
export function assign(d: Draft, cell: number | null): Draft {
  if (d.current === null || d.pendingOp !== null) return d;
  return { groups: [...d.groups, { expr: d.current, cell }], current: null, pendingOp: null };
}

export function toAction(d: Draft): BlackPotAction | null {
  if (d.current !== null || d.pendingOp !== null || usedDice(d).length !== 3) return null;
  return { Play: { groups: d.groups } };
}

/** The calculation as typed, with brackets: formatting only. */
export function exprText(e: Expr, dice: number[]): string {
  if ("Die" in e) return `${dice[e.Die]}`;
  const side = (s: Expr) => ("Die" in s ? exprText(s, dice) : `(${exprText(s, dice)})`);
  return `${side(e.Op.left)} ${SIGNS[e.Op.op]} ${side(e.Op.right)}`;
}

/** A draft belongs to one roll and one player's turn. */
export function draftKey(v: BlackPotView): string {
  return `${v.dice.join(",")}|${v.roller}|${v.turn}|${v.queue.length}`;
}

export function describeTurn(t: TurnRecord, name: string): string {
  if ("NotFound" in t) return `${name} ne trouve pas (${t.NotFound.dice.join(", ")}).`;
  if ("Passed" in t) return `${name} n'a aucun coup avec ${t.Passed.dice.join(", ")}.`;
  const { groups, points } = t.Played;
  const text = groups.map((g) => g.text).join(" ; ");
  return points > 0 ? `${name} : ${text} (${points} points)` : `${name} : ${text}`;
}

function teamLabel(v: BlackPotView, team: number, names: string[]): string {
  return `${v.teams[team].seats.map((s) => names[s]).join(" et ")} (${TEAM_NAMES[team]})`;
}

function endText(v: BlackPotView, end: End, names: string[]): string {
  const why =
    end === "QuietLimit"
      ? "Arrêt : 10 tours de table sans pion posé."
      : `${teamLabel(v, end.AllPlaced.team, names)} a posé son dernier pion.`;
  const result =
    v.winners.length > 1 ? "Égalité." : `${teamLabel(v, v.winners[0], names)} gagne.`;
  return `${why} ${result}`;
}

function Die({ face, x, y, s }: { face: number; x: number; y: number; s: number }) {
  return (
    <>
      <rect x={x - s / 2} y={y - s / 2} width={s} height={s} rx={s / 6} className="bp-die" />
      {(PIPS[face] ?? []).map(([dx, dy], k) => (
        <circle key={k} cx={x + (dx * s) / 4} cy={y + (dy * s) / 4} r={s / 11} className="bp-pip" />
      ))}
    </>
  );
}

function star(cx: number, cy: number, r: number): string {
  return Array.from({ length: 16 }, (_, i) => {
    const rr = i % 2 === 0 ? r : r * 0.62;
    const a = (Math.PI * i) / 8;
    return `${cx + rr * Math.cos(a)},${cy + rr * Math.sin(a)}`;
  }).join(" ");
}

export function BlackPotBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as BlackPotView;
  const myTurn = me !== null && v.turn === me && legalActions.length > 0;
  const key = draftKey(v);
  const [kept, setKept] = useState<{ key: string; draft: Draft }>({ key, draft: EMPTY_DRAFT });
  const draft = kept.key === key ? kept.draft : EMPTY_DRAFT;
  const set = (d: Draft) => setKept({ key, draft: d });
  const used = usedDice(draft);
  const aimed = new Set(draft.groups.map((g) => g.cell).filter((c): c is number => c !== null));
  const assigning = myTurn && draft.current !== null && draft.pendingOp === null;
  const action = toAction(draft);
  const centre = (c: number) => ({ x: BOARD.cells[c].col * SIZE + SIZE / 2, y: BOARD.cells[c].row * SIZE + SIZE / 2 });

  const status = v.end
    ? endText(v, v.end, seatNames)
    : myTurn
      ? "À toi : compose un calcul avec les dés, puis clique sa case. Recommence jusqu'à avoir employé les 3 dés."
      : v.turn !== null
        ? `${seatNames[v.turn]} réfléchit…`
        : "";

  return (
    <div className="bp">
      <ul className="legend">
        {v.teams.map((t, team) => (
          <li key={team} className={v.turn !== null && t.seats.includes(v.turn) ? "active" : undefined}>
            <span className={`bp-swatch team-${team}`} />
            {`${teamLabel(v, team, seatNames)} : ${t.score} points, ${t.pawnsLeft} pions en réserve`}
          </li>
        ))}
      </ul>
      <div className="bp-roll">
        <svg viewBox="0 0 210 70" className="bp-dice" role="img" aria-label={`Dés : ${v.dice.join(", ")}`}>
          {v.dice.map((face, i) => (
            <g key={i} className={used.includes(i) ? "used" : undefined}>
              <Die face={face} x={35 + i * 70} y={35} s={54} />
            </g>
          ))}
        </svg>
        <span className="hint">
          {`Tirage de ${seatNames[v.roller]}`}
          {v.queue.length > 1 ? ` : jouent ${v.queue.map((s) => seatNames[s]).join(", ")}` : ""}
        </span>
      </div>
      <svg className="bp-board felt" viewBox={`0 0 ${SIZE * 11} ${SIZE * 11}`} role="img" aria-label="Plateau de Black-Pot">
        {BOARD.cells.map((c, i) => {
          const x = c.col * SIZE;
          const y = c.row * SIZE;
          const free = v.pawns[i] === null && !aimed.has(i);
          const click = assigning && free ? () => set(assign(draft, i)) : undefined;
          return (
            <g key={i} data-cell={i} className={`bp-cell ${c.kind}${click ? " open" : ""}`} onClick={click}>
              <rect x={x} y={y} width={SIZE} height={SIZE} className="bp-square" />
              {c.kind === "BlackPot" && <polygon points={star(x + SIZE / 2, y + SIZE / 2, SIZE * 0.42)} className="bp-star" />}
              <text x={x + SIZE / 2} y={y + SIZE / 2 + 9} textAnchor="middle" className="bp-value">
                {c.value}
              </text>
            </g>
          );
        })}
        {v.alignments.map((a, k) => {
          const p = centre(a.cells[0]);
          const q = centre(a.cells[4]);
          return <line key={k} x1={p.x} y1={p.y} x2={q.x} y2={q.y} className={`bp-line team-${a.team}`} />;
        })}
        {v.pawns.map((p, i) => {
          if (p === null) return null;
          const { x, y } = centre(i);
          return (
            <g key={`p${i}`} className={`bp-pawn team-${p.team}${p.flipped ? " flipped" : ""}`}>
              <circle cx={x} cy={y} r={SIZE * 0.3} />
              {p.flipped && <circle cx={x} cy={y} r={SIZE * 0.13} className="bp-mark" />}
            </g>
          );
        })}
        {[...aimed].map((c) => {
          const { x, y } = centre(c);
          return <circle key={`d${c}`} cx={x} cy={y} r={SIZE * 0.3} className="bp-ghost" />;
        })}
      </svg>
      <p className="status">{status}</p>
      {myTurn && (
        <div className="bp-compose">
          <p className="bp-draft">
            {draft.groups.map((g, k) => (
              <span key={k}>
                {`${exprText(g.expr, v.dice)} → ${g.cell === null ? "0" : `case ${BOARD.cells[g.cell].value}`}`}
              </span>
            ))}
            {draft.current && (
              <span className="current">
                {exprText(draft.current, v.dice)}
                {draft.pendingOp ? ` ${SIGNS[draft.pendingOp]} …` : ""}
              </span>
            )}
            {draft.groups.length === 0 && !draft.current && <span>Choisis un dé.</span>}
          </p>
          <div className="bp-keys">
            {v.dice.map((face, i) => (
              <button key={`d${i}`} disabled={used.includes(i)} onClick={() => set(clickDie(draft, i))}>
                {face}
              </button>
            ))}
            {OPS.map((op) => (
              <button key={op} disabled={draft.current === null} onClick={() => set(clickOp(draft, op))}>
                {SIGNS[op]}
              </button>
            ))}
            <button disabled={draft.current === null || !("Op" in draft.current)} onClick={() => set(swap(draft))} title="Échanger les deux côtés">
              ⇄
            </button>
            <button disabled={!assigning} onClick={() => set(assign(draft, null))}>
              = 0
            </button>
          </div>
          <div className="bp-keys">
            <button onClick={() => set(EMPTY_DRAFT)}>Effacer</button>
            <button className="primary" disabled={action === null} onClick={() => action && submit(action)}>
              Valider
            </button>
            <button onClick={() => submit("NotFound" satisfies BlackPotAction)}>Je ne trouve pas</button>
          </div>
        </div>
      )}
      <ul className="bp-log">
        {v.last.map((t, seat) => (t ? <li key={seat}>{describeTurn(t, seatNames[seat])}</li> : null))}
      </ul>
      {!v.end && <p className="hint">{`Limite : encore ${v.quietLeft} tours de table sans pion posé avant l'arrêt.`}</p>}
    </div>
  );
}
