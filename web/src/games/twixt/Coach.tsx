import { holeName } from "./Board";
import { GRADE_NAMES, lossText, percent } from "./coach";
import type { Advice, Candidate } from "./types";

interface Props {
  advice: Advice;
  heat: boolean;
  onHeat(on: boolean): void;
  /** The candidate under the pointer (a ghost peg on the board), or null. */
  onHover(c: Candidate | null): void;
}

/** The coach's numbers for the side to move. */
export function CoachPanel({ advice, heat, onHeat, onHover }: Props) {
  return (
    <section className="panel tw-coach" aria-label="Coach">
      <p>
        <strong>{`Tes chances : ${percent(advice.win)}`}</strong>
      </p>
      {advice.last && (
        <p className={`tw-grade-${advice.last.grade}`}>
          {`Ton dernier coup : ${GRADE_NAMES[advice.last.grade]} (${lossText(advice.last.loss)})`}
        </p>
      )}
      <p className="hint">Les coups préférés du réseau, avec tes chances après :</p>
      <ol className="tw-candidates">
        {advice.best.map((c) => (
          <li key={holeName(c.at)}>
            <button type="button" className="ghost" onMouseEnter={() => onHover(c)} onMouseLeave={() => onHover(null)} onFocus={() => onHover(c)} onBlur={() => onHover(null)}>
              {`${holeName(c.at)} — ${percent(c.win)}`}
            </button>
          </li>
        ))}
      </ol>
      <label className="tw-heat-toggle">
        <input type="checkbox" checked={heat} onChange={(e) => onHeat(e.target.checked)} /> Carte de chaleur
      </label>
    </section>
  );
}
