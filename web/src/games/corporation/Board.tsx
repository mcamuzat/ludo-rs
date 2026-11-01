import { useState } from "react";
import type { BoardProps } from "../registry";
import type { Color, CorporationAction, CorporationView, End, Step } from "./types";
import "./corporation.css";

const ALL: Color[] = ["Red", "Blue", "Green", "Yellow"];
const NAME: Record<Color, string> = { Red: "rouge", Blue: "bleu", Green: "vert", Yellow: "jaune" };
const DESKS = 4;
const BOARDROOM = 12;

// Geometry of the building (SVG units).
const W = 320;
const TOP = 56;
const FLOOR = 44;
const DESK_H = 52;
const DESK_Y = TOP + 8 * FLOOR;
const R = 14;

function area(pos: number) {
  if (pos < DESKS) return { x: (pos * W) / DESKS, y: DESK_Y, w: W / DESKS, h: DESK_H };
  if (pos === BOARDROOM) return { x: 0, y: 0, w: W, h: TOP };
  const floor = pos - DESKS + 1;
  return { x: 0, y: TOP + (8 - floor) * FLOOR, w: W, h: FLOOR };
}

/** Circle 2k / 2k + 1 is promotion k + 1, numbered from the bottom. */
function circleAt(circle: number) {
  const k = Math.floor(circle / 2);
  return { x: circle % 2 === 0 ? 372 : 418, y: 370 - k * 56 };
}

export function placeName(pos: number): string {
  if (pos < DESKS) return `l'accueil ${NAME[ALL[pos]]}`;
  if (pos === BOARDROOM) return "la salle du conseil";
  const floor = pos - DESKS + 1;
  return `${floor === 1 ? "1er" : `${floor}e`} étage`;
}

function fromPlace(pos: number): string {
  return pos < DESKS || pos === BOARDROOM ? `de ${placeName(pos)}` : `du ${placeName(pos)}`;
}

function toPlace(pos: number): string {
  return pos < DESKS || pos === BOARDROOM ? `à ${placeName(pos)}` : `au ${placeName(pos)}`;
}

export function stepText(step: Step): string {
  if ("Swap" in step) {
    const [pa, pb] = [step.Swap.a, step.Swap.b].map((c) => Math.floor(c / 2) + 1);
    return pa === pb ? `retourne la promotion ${pa}` : `échange deux jetons (promotions ${pa} et ${pb})`;
  }
  if ("Enter" in step) return `fait entrer un employé ${NAME[step.Enter.color]} à ${placeName(step.Enter.desk)}`;
  const { color, from, to } = step.Promote;
  return `un ${NAME[color]} monte ${fromPlace(from)} ${toPlace(to)}`;
}

/** The steps in words, an employee's successive promotions told as one climb. */
export function stepsText(steps: Step[]): string {
  const merged: Step[] = [];
  for (const step of steps) {
    const prev = merged[merged.length - 1];
    if (prev && "Promote" in prev && "Promote" in step && prev.Promote.to === step.Promote.from && prev.Promote.color === step.Promote.color) {
      merged[merged.length - 1] = { Promote: { ...prev.Promote, to: step.Promote.to } };
    } else {
      merged.push(step);
    }
  }
  return merged.map(stepText).join(" ; ");
}

export function endText(end: End, names: string[]): string {
  if (end === "Quiet") return "Partie nulle : 20 tours de table sans entrée ni promotion.";
  return `${names[end.Won.seat]} gagne : ses trois employés sont à la salle du conseil.`;
}

function Employee({ x, y, color, ring }: { x: number; y: number; color: Color; ring?: boolean }) {
  return <circle cx={x} cy={y} r={R} className={`corp-man corp-${color}${ring ? " eligible" : ""}`} />;
}

export function CorporationBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as CorporationView;
  const legal = legalActions as CorporationAction[];
  const myTurn = me !== null && v.turn === me && legal.length > 0;
  const [color, setColor] = useState<Color | null>(null);
  const [circle, setCircle] = useState<number | null>(null);
  const has = (a: CorporationAction) => legal.some((l) => JSON.stringify(l) === JSON.stringify(a));
  const canSkip = myTurn && legal.includes("Skip");

  const placeOn = (c: number): CorporationAction | null =>
    v.phase === "Draft" && color !== null && has({ Place: { circle: c, color } }) ? { Place: { circle: c, color } } : null;
  const swapWith = (c: number): CorporationAction | null => {
    if (v.phase !== "Swap" || circle === null || circle === c) return null;
    const a: CorporationAction = { Swap: { a: Math.min(circle, c), b: Math.max(circle, c) } };
    return has(a) ? a : null;
  };
  const canPickCircle = (c: number) =>
    myTurn && v.phase === "Swap" && legal.some((l) => typeof l === "object" && "Swap" in l && (l.Swap.a === c || l.Swap.b === c));
  const enterAt = (desk: number): CorporationAction | null =>
    v.phase === "Entry" && color !== null && has({ Enter: { color, desk } }) ? { Enter: { color, desk } } : null;
  const promote = (pos: number): CorporationAction | null =>
    v.phase === "Promote" && has({ Promote: { from: pos } }) ? { Promote: { from: pos } } : null;

  const play = (a: CorporationAction) => {
    setColor(null);
    setCircle(null);
    submit(a);
  };

  const onCircle = (c: number) => {
    const a = placeOn(c) ?? swapWith(c);
    if (a) return play(a);
    if (canPickCircle(c)) setCircle(circle === c ? null : c);
  };

  const owner = (c: Color) => v.colors.indexOf(c);

  return (
    <div className="corporation">
      <ul className="legend">
        {ALL.map((c) => {
          const seat = owner(c);
          return (
            <li key={c} className={seat >= 0 && v.turn === seat ? "active" : undefined}>
              <span className={`swatch corp-${c}`} />
              {seat >= 0 ? `${seatNames[seat]}` : "neutre"}
              {` — ${v.arrived[ALL.indexOf(c)]}/3 au conseil`}
            </li>
          );
        })}
      </ul>
      <svg className="corp-board felt" viewBox="-8 -8 482 476" role="img" aria-label="Immeuble de Corporation">
        {v.rooms.map((room, pos) => {
          const { x, y, w, h } = area(pos);
          const action = enterAt(pos) ?? promote(pos);
          const stack = v.stacks[pos];
          const step = stack.length > 1 ? Math.min(26, (w - 2 * R - 12) / (stack.length - 1)) : 0;
          const cy = y + h / 2;
          return (
            <g
              key={pos}
              className={`corp-room${action && myTurn ? " playable" : ""}`}
              onClick={action && myTurn ? () => play(action) : undefined}
              role={action && myTurn ? "button" : undefined}
              aria-label={action && myTurn ? placeName(pos) : undefined}
            >
              <rect x={x + 1} y={y + 1} width={w - 2} height={h - 2} rx={4} className={room ? `corp-office corp-${room}` : "corp-boardroom"} />
              <text x={x + w - 8} y={y + 14} className="corp-label">
                {pos === BOARDROOM ? "Salle du conseil" : pos < DESKS ? "" : pos - DESKS + 1}
              </text>
              {stack.map((c, i) => (
                <Employee
                  key={i}
                  x={x + R + 8 + i * step}
                  y={cy + (pos === BOARDROOM ? 6 : 0)}
                  color={c}
                  ring={i === stack.length - 1 && v.eligible.includes(pos)}
                />
              ))}
            </g>
          );
        })}
        <rect x={346} y={60} width={98} height={340} rx={49} className="corp-table" />
        {v.table.map((c, i) => {
          const { x, y } = circleAt(i);
          const action = myTurn ? (placeOn(i) ?? swapWith(i)) : null;
          const clickable = action !== null || canPickCircle(i);
          return (
            <g
              key={i}
              className={`corp-circle${clickable ? " playable" : ""}${circle === i ? " selected" : ""}`}
              onClick={clickable ? () => onCircle(i) : undefined}
              role={clickable ? "button" : undefined}
              aria-label={clickable ? `promotion ${Math.floor(i / 2) + 1} ${i % 2 === 0 ? "gauche" : "droite"}` : undefined}
            >
              <circle cx={x} cy={y} r={20} className={c ? `corp-counter corp-${c}` : "corp-free"} />
            </g>
          );
        })}
        {[0, 1, 2, 3, 4, 5].map((k) => (
          <text key={k} x={456} y={370 - k * 56 + 1} className="corp-number">
            {k + 1}
          </text>
        ))}
      </svg>

      {v.end ? (
        <p className="status">{endText(v.end, seatNames)}</p>
      ) : (
        <>
          {myTurn && v.phase === "Draft" && (
            <>
              <div className="corp-picks">
                {ALL.map((c, i) => (
                  <button
                    key={c}
                    type="button"
                    className={`corp-pick corp-${c}${color === c ? " selected" : ""}`}
                    disabled={v.supply[i] === 0}
                    aria-pressed={color === c}
                    onClick={() => setColor(c)}
                  >
                    {v.supply[i]}
                  </button>
                ))}
              </div>
              <p className="hint">Choisis une couleur de jeton, puis un cercle libre de la table.</p>
            </>
          )}
          {myTurn && v.phase === "Swap" && (
            <>
              <p className="hint">
                {circle === null
                  ? "Échange deux jetons de la table : clique-les l'un après l'autre."
                  : "Clique le second jeton à échanger."}
              </p>
              <button type="button" onClick={() => play("Skip")} disabled={!canSkip}>
                Ne rien échanger
              </button>
            </>
          )}
          {myTurn && v.phase === "Entry" && (
            <>
              <div className="corp-picks">
                {ALL.map((c, i) => (
                  <button
                    key={c}
                    type="button"
                    className={`corp-pick corp-${c}${color === c ? " selected" : ""}`}
                    disabled={v.outside[i] === 0}
                    aria-pressed={color === c}
                    onClick={() => setColor(c)}
                  >
                    {v.outside[i]}
                  </button>
                ))}
              </div>
              <p className="hint">
                {color === null
                  ? "Fais entrer un employé : choisis sa couleur, puis une case de l'accueil."
                  : `Choisis la case de l'accueil où entre l'employé ${NAME[color]}.`}
              </p>
              <button type="button" onClick={() => play("Skip")} disabled={!canSkip}>
                Ne faire entrer personne
              </button>
            </>
          )}
          {myTurn && v.phase === "Promote" && (
            <p className="hint">Plusieurs promotions s'appliquent : clique la pile dont l'employé du sommet monte d'abord.</p>
          )}
          {!myTurn && v.turn !== null && <p className="hint">{`À ${seatNames[v.turn]} de jouer.`}</p>}
          {v.phase !== "Draft" && (
            <p className="hint">
              {`Dehors : ${ALL.map((c, i) => `${v.outside[i]} ${NAME[c]}`).join(", ")}.`}
              {v.quietLeft <= 5 && ` Plus que ${v.quietLeft} tour(s) de table sans promotion avant la nulle.`}
            </p>
          )}
        </>
      )}
      {v.turn !== null && v.steps.length > 0 && <p className="hint">{`Ce tour : ${stepsText(v.steps)}.`}</p>}
      {v.lastTurn && (
        <p className="hint">
          {`Tour précédent (${seatNames[v.lastTurn.seat]}) : ${
            v.lastTurn.steps.length > 0 ? stepsText(v.lastTurn.steps) : "rien"
          }.`}
        </p>
      )}
    </div>
  );
}
