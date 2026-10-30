import type { BoardProps } from "../registry";
import type { CellView, End, SiberianDiceAction, SiberianDiceView, Source } from "./types";
import "./siberian-dice.css";

const SIZE = 30;
const SQRT3 = Math.sqrt(3);
const COLORS = ["Bleu", "Rouge"];

/** Pointy-top layout of the printed board: 11 on the left, 77 on the right, 41 and 74 on top. */
function center(c: CellView) {
  const q = c.a - 4;
  const r = c.b - c.a;
  return { x: SIZE * SQRT3 * (q + r / 2), y: SIZE * 1.5 * r };
}

function hexPoints(x: number, y: number): string {
  return Array.from({ length: 6 }, (_, k) => {
    const angle = (Math.PI / 180) * (60 * k - 30);
    return `${x + SIZE * Math.cos(angle)},${y + SIZE * Math.sin(angle)}`;
  }).join(" ");
}

/** Pip positions on a die face, in units of a third of its half side. */
const PIPS: Record<number, [number, number][]> = {
  1: [[0, 0]],
  2: [[-1, -1], [1, 1]],
  3: [[-1, -1], [0, 0], [1, 1]],
  4: [[-1, -1], [1, -1], [-1, 1], [1, 1]],
  5: [[-1, -1], [1, -1], [0, 0], [-1, 1], [1, 1]],
  6: [[-1, -1], [1, -1], [-1, 0], [1, 0], [-1, 1], [1, 1]],
};

function Die({ x, y, half, owner, value }: { x: number; y: number; half: number; owner: number; value: number }) {
  const step = half * 0.55;
  return (
    <>
      <rect
        className={`sd-die owner-${owner}`}
        x={x - half}
        y={y - half}
        width={2 * half}
        height={2 * half}
        rx={half * 0.28}
      />
      {(PIPS[value] ?? []).map(([dx, dy], i) => (
        <circle key={i} className="sd-pip" cx={x + dx * step} cy={y + dy * step} r={half * 0.16} />
      ))}
    </>
  );
}

function sameSource(a: Source, b: Source): boolean {
  return a === "Reserve" ? b === "Reserve" : b !== "Reserve" && a.Cell === b.Cell;
}

function endText(end: End, names: string[]): string {
  if (end === "BothSix") return "Partie nulle : six dés parfaits chacun au même coup.";
  if (end === "QuietLimit") return "Partie nulle : 30 tours de table sans nouveau record.";
  return `${names[end.Won.seat]} gagne avec six dés parfaits.`;
}

export function SiberianDiceBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as SiberianDiceView;
  const legal = legalActions as SiberianDiceAction[];
  const canPick = (from: Source) => legal.some((a) => "Pick" in a && sameSource(a.Pick, from));
  const canPlace = (id: number) => legal.some((a) => "Place" in a && a.Place === id);
  const rolled = typeof v.step === "object" ? v.step.Place : null;
  const last = v.last;
  const lastCells = last ? [last.to, ...(last.from === "Reserve" ? [] : [last.from.Cell])] : [];

  return (
    <div className="sd">
      <ul className="sd-players">
        {[0, 1].map((seat) => (
          <li key={seat} className={!v.end && v.turn === seat ? "active" : undefined}>
            <span className={`sd-swatch owner-${seat}`} />
            <span>
              {COLORS[seat]} : {seatNames[seat]}
            </span>
            <button
              className="sd-reserve"
              disabled={me !== seat || !canPick("Reserve")}
              onClick={() => submit({ Pick: "Reserve" } satisfies SiberianDiceAction)}
            >
              Réserve : {v.reserve[seat]}
            </button>
            <span>parfaits {v.perfect[seat]}/6</span>
            {rolled && v.turn === seat && (
              <svg className="sd-rolled" viewBox="-22 -22 44 44" role="img" aria-label={`Dé lancé : ${rolled.value}`}>
                <Die x={0} y={0} half={18} owner={seat} value={rolled.value} />
              </svg>
            )}
          </li>
        ))}
      </ul>
      <svg className="sd-board felt" viewBox="-196 -170 392 340" role="img" aria-label="Plateau de Siberian Dice">
        {v.cells.map((c, id) => {
          const { x, y } = center(c);
          const playable = canPlace(id);
          const pickable = canPick({ Cell: id });
          const classes = [
            "sd-cell",
            playable && "playable",
            pickable && "pickable",
            c.die?.perfect && "perfect",
            lastCells.includes(id) && "last",
          ].filter(Boolean);
          const action: SiberianDiceAction | null = playable
            ? { Place: id }
            : pickable
              ? { Pick: { Cell: id } }
              : null;
          return (
            <g
              key={id}
              data-cell={10 * c.a + c.b}
              className={classes.join(" ")}
              onClick={action ? () => submit(action) : undefined}
            >
              <polygon points={hexPoints(x, y)} className="sd-hex" />
              {c.die && <Die x={x} y={y} half={SIZE * 0.5} owner={c.die.owner} value={c.die.value} />}
            </g>
          );
        })}
      </svg>
      {v.end ? (
        <p className="hint">{endText(v.end, seatNames)}</p>
      ) : (
        <p className="hint">
          {me === v.turn
            ? rolled
              ? `Pose ton ${rolled.value} sur une case vide.`
              : "Choisis un dé à lancer : ta réserve ou un de tes dés posés."
            : rolled
              ? `${seatNames[v.turn]} pose son ${rolled.value}.`
              : `${seatNames[v.turn]} choisit un dé à lancer.`}{" "}
          {`Limite : encore ${v.quietLeft} tours de table sans nouveau record.`}
        </p>
      )}
    </div>
  );
}
