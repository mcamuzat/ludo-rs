import type { BoardProps } from "../registry";
import type { Color, Point, TwixtAction, TwixtView } from "./types";
import "./twixt.css";

const SIZE = 24;
const STEP = 20;
const MARGIN = 26;
const EXTENT = (SIZE - 1) * STEP;

export const COLOR_NAMES: Record<Color, string> = { Red: "Rouge", Black: "Noir" };

export function seatOf(c: Color): number {
  return c === "Red" ? 0 : 1;
}

/** twixtbot's notation: column letter, then row from 1. */
export function holeName(p: Point): string {
  return `${String.fromCharCode(65 + p.x)}${p.y + 1}`;
}

export function statusText(v: TwixtView, names: string[]): string {
  if (v.result === "Draw") return "Partie nulle : plus personne ne peut relier ses bords.";
  if (v.result) return `${names[seatOf(v.result.Won)]} (${COLOR_NAMES[v.result.Won]}) gagne.`;
  return `À ${names[seatOf(v.turn)]} (${COLOR_NAMES[v.turn]}) de jouer.`;
}

const same = (a: Point, b: Point) => a.x === b.x && a.y === b.y;
const isCorner = (x: number, y: number) => (x === 0 || x === SIZE - 1) && (y === 0 || y === SIZE - 1);

export interface Overlay {
  /** A peg shown faded (a coach candidate under the pointer). */
  ghost?: { at: Point; color: Color } | null;
  /** The policy's probability per hole, drawn as halos. */
  heat?: [Point, number][] | null;
}

interface SvgProps extends Overlay {
  view: TwixtView;
  /** Holes that can be clicked. */
  playable?: Point[];
  onHole?(p: Point): void;
}

/** The board itself: holes, border lines, links, pegs. */
export function TwixtSvg({ view, playable = [], onHole, ghost, heat }: SvgProps) {
  const holes = [];
  for (let x = 0; x < SIZE; x++) {
    for (let y = 0; y < SIZE; y++) {
      if (isCorner(x, y)) continue;
      const p = { x, y };
      const can = onHole !== undefined && playable.some((q) => same(q, p));
      holes.push(
        <g
          key={`${x}-${y}`}
          className={`tw-cell${can ? " playable" : ""}`}
          role={can ? "button" : undefined}
          aria-label={can ? `trou ${holeName(p)}` : undefined}
          onClick={can ? () => onHole(p) : undefined}
        >
          {can && <circle cx={x * STEP} cy={y * STEP} r={STEP / 2 - 1} className="tw-target" />}
          <circle cx={x * STEP} cy={y * STEP} r={2.5} className="tw-hole" />
        </g>,
      );
    }
  }
  const maxHeat = Math.max(1e-9, ...(heat ?? []).map(([, h]) => h));
  const inChain = (p: Point) => view.chain?.some((q) => same(q, p)) ?? false;
  const half = STEP / 2;
  return (
    <svg
      className="tw-board felt"
      viewBox={`${-MARGIN} ${-MARGIN} ${EXTENT + 2 * MARGIN} ${EXTENT + 2 * MARGIN}`}
      role="img"
      aria-label="Plateau de TwixT"
    >
      {Array.from({ length: SIZE }, (_, i) => (
        <g key={i} className="tw-coord">
          <text x={i * STEP} y={-MARGIN / 2 - 2}>
            {String.fromCharCode(65 + i)}
          </text>
          <text x={-MARGIN / 2 - 2} y={i * STEP}>
            {i + 1}
          </text>
        </g>
      ))}
      {/* Red's border rows (top, bottom), Black's border columns (left, right). */}
      <line className="tw-border tw-red" x1={half} y1={half} x2={EXTENT - half} y2={half} />
      <line className="tw-border tw-red" x1={half} y1={EXTENT - half} x2={EXTENT - half} y2={EXTENT - half} />
      <line className="tw-border tw-black" x1={half} y1={half} x2={half} y2={EXTENT - half} />
      <line className="tw-border tw-black" x1={EXTENT - half} y1={half} x2={EXTENT - half} y2={EXTENT - half} />
      {heat?.map(([p, h]) => (
        <circle
          key={`h${p.x}-${p.y}`}
          cx={p.x * STEP}
          cy={p.y * STEP}
          r={2 + 7 * Math.sqrt(h / maxHeat)}
          className="tw-heat"
          opacity={0.25 + 0.6 * (h / maxHeat)}
        />
      ))}
      {holes}
      {view.links.map(([c, a, b]) => (
        <line
          key={`l${a.x}-${a.y}-${b.x}-${b.y}`}
          className={`tw-link tw-${c.toLowerCase()}`}
          x1={a.x * STEP}
          y1={a.y * STEP}
          x2={b.x * STEP}
          y2={b.y * STEP}
        />
      ))}
      {view.pegs.map(([p, c]) => (
        <circle
          key={`p${p.x}-${p.y}`}
          cx={p.x * STEP}
          cy={p.y * STEP}
          r={6}
          className={`tw-peg tw-${c.toLowerCase()}${inChain(p) ? " tw-chain" : ""}`}
        />
      ))}
      {view.last && <circle cx={view.last.x * STEP} cy={view.last.y * STEP} r={9} className="tw-last" />}
      {ghost && (
        <circle cx={ghost.at.x * STEP} cy={ghost.at.y * STEP} r={6} className={`tw-peg tw-ghost tw-${ghost.color.toLowerCase()}`} />
      )}
    </svg>
  );
}

export function TwixtBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as TwixtView;
  const legal = legalActions as TwixtAction[];
  const myTurn = me !== null && !v.result && seatOf(v.turn) === me && legal.length > 0;
  const playable = myTurn ? legal.flatMap((a) => (a === "Swap" ? [] : [a.Place])) : [];
  const canSwap = myTurn && legal.includes("Swap");
  return (
    <div className="twixt">
      <ul className="legend">
        {(["Red", "Black"] as const).map((c) => (
          <li key={c} className={!v.result && v.turn === c ? "active" : undefined}>
            <span className={`swatch tw-${c.toLowerCase()}`} />
            {`${COLOR_NAMES[c]} : ${seatNames[seatOf(c)]} — relie ${c === "Red" ? "le haut et le bas" : "la gauche et la droite"}`}
          </li>
        ))}
      </ul>
      <TwixtSvg view={v} playable={playable} onHole={myTurn ? (p) => submit({ Place: p } satisfies TwixtAction) : undefined} />
      <p className="status">{statusText(v, seatNames)}</p>
      {canSwap && (
        <button type="button" className="primary" onClick={() => submit("Swap" satisfies TwixtAction)}>
          Prendre son pion (swap)
        </button>
      )}
    </div>
  );
}
