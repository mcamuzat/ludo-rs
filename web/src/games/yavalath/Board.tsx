import type { BoardProps } from "../registry";
import type { Cell, Color, YavalathAction, YavalathView } from "./types";

const SIZE = 26;
const SQRT3 = Math.sqrt(3);
const FILL: Record<Color, string> = { White: "#f4f1ea", Black: "#23211f", Red: "#c8322b" };
const NAMES: Record<Color, string> = { White: "Blanc", Black: "Noir", Red: "Rouge" };

/** Pointy-top axial layout. */
function center(c: Cell) {
  return { x: SIZE * SQRT3 * (c.q + c.r / 2), y: SIZE * 1.5 * c.r };
}

function hexPoints(x: number, y: number): string {
  return Array.from({ length: 6 }, (_, k) => {
    const angle = (Math.PI / 180) * (60 * k - 30);
    return `${x + SIZE * Math.cos(angle)},${y + SIZE * Math.sin(angle)}`;
  }).join(" ");
}

function same(a: Cell, b: Cell) {
  return a.q === b.q && a.r === b.r;
}

export function YavalathBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as YavalathView;
  const legal = legalActions as YavalathAction[];
  const placeable = legal.flatMap((a) => (typeof a === "object" ? [a.Place] : []));
  const canSwap = legal.includes("Swap");
  const inLine = v.lines.flat();

  return (
    <div className="yavalath">
      <ul className="legend">
        {v.seatColors.map((color, seat) => (
          <li key={seat} className={v.current === seat ? "active" : undefined}>
            <span className="swatch" style={{ background: FILL[color] }} />
            {NAMES[color]} : {seatNames[seat]}
            {v.eliminated.includes(seat) ? " (éliminé)" : ""}
          </li>
        ))}
      </ul>
      <svg className="hexboard" viewBox="-210 -190 420 380" role="img" aria-label="Plateau de Yavalath">
        {v.cells.map((c) => {
          const { x, y } = center(c);
          const playable = placeable.some((p) => same(p, c));
          const forced = v.forced.some((f) => same(f, c));
          return (
            <g
              key={`${c.q},${c.r}`}
              className={playable ? "cell playable" : "cell"}
              onClick={playable ? () => submit({ Place: { q: c.q, r: c.r } } satisfies YavalathAction) : undefined}
            >
              <polygon points={hexPoints(x, y)} className={forced ? "hex forced" : "hex"} />
              {c.stone && (
                <circle
                  cx={x}
                  cy={y}
                  r={SIZE * 0.62}
                  fill={FILL[c.stone]}
                  className={inLine.some((l) => same(l, c)) ? "stone in-line" : "stone"}
                />
              )}
              {v.lastMove && same(v.lastMove, c) && <circle cx={x} cy={y} r={4} className="last-move" />}
            </g>
          );
        })}
      </svg>
      {canSwap && (
        <button onClick={() => submit("Swap" satisfies YavalathAction)}>Échanger les couleurs</button>
      )}
      {me !== null && v.current === me && v.forced.length > 0 && (
        <p className="hint">Tu dois bloquer la victoire du joueur suivant.</p>
      )}
    </div>
  );
}
