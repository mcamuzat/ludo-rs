import { COLOR_NAMES } from "./Board";
import { GRADE_NAMES, percent } from "./coach";
import type { Review } from "./types";

const W = 600;
const H = 160;
const PAD = 12;

interface Props {
  /** Red's win probability after each ply. */
  curve: number[];
  review: Review;
  names: string[];
  /** The ply shown on the board, or null for the final position. */
  selected: number | null;
  onSelect(ply: number | null): void;
}

/** Move k (1-based) is Red's when k is odd: moves alternate, swap included. */
function moverOf(ply: number): "Red" | "Black" {
  return ply % 2 === 1 ? "Red" : "Black";
}

/** The game's curve of Red's chances, the grades and the key moments. */
export function ReviewPanel({ curve, review, names, selected, onSelect }: Props) {
  const n = Math.max(1, curve.length - 1);
  const x = (ply: number) => PAD + ((W - 2 * PAD) * ply) / n;
  const y = (p: number) => PAD + (H - 2 * PAD) * (1 - p);
  const path = curve.map((p, ply) => `${ply === 0 ? "M" : "L"}${x(ply).toFixed(1)},${y(p).toFixed(1)}`).join(" ");
  const who = (ply: number) => {
    const c = moverOf(ply);
    return `${names[c === "Red" ? 0 : 1]}, ${COLOR_NAMES[c]}`;
  };
  return (
    <section className="panel tw-review" aria-label="Revue de la partie">
      <svg className="tw-curve" viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Chances de Rouge au fil des coups">
        <line className="tw-half" x1={PAD} x2={W - PAD} y1={y(0.5)} y2={y(0.5)} />
        <path className="tw-curve-line" d={path} />
        {curve.map((p, ply) => {
          const g = review.grades[ply];
          return (
            <circle
              key={ply}
              cx={x(ply)}
              cy={y(p)}
              r={selected === ply ? 6 : g === "Mistake" || g === "Blunder" ? 4.5 : 3}
              className={`tw-point${g ? ` tw-grade-${g}` : ""}${selected === ply ? " selected" : ""}`}
              role="button"
              aria-label={`coup ${ply}`}
              onClick={() => onSelect(ply)}
            >
              <title>{ply === 0 ? `Début : Rouge ${percent(p)}` : `Coup ${ply} (${who(ply)}) : Rouge ${percent(p)}`}</title>
            </circle>
          );
        })}
      </svg>
      {review.key.length > 0 ? (
        <>
          <p className="hint">Moments clés :</p>
          <ul className="tw-key">
            {review.key.map((ply) => (
              <li key={ply}>
                <button type="button" className="ghost" onClick={() => onSelect(ply)}>
                  {`Coup ${ply} (${who(ply)}) : ${GRADE_NAMES[review.grades[ply] ?? "Best"]}`}
                </button>
              </li>
            ))}
          </ul>
        </>
      ) : (
        <p className="hint">Aucune grosse erreur dans cette partie.</p>
      )}
      {selected !== null && (
        <p>
          {`Position après le coup ${selected}. `}
          <button type="button" className="ghost" onClick={() => onSelect(null)}>
            Retour à la fin
          </button>
        </p>
      )}
    </section>
  );
}
