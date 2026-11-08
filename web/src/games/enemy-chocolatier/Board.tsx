import type { BoardProps } from "../registry";
import boardJson from "./board.json";
import type {
  Ability,
  BoardJson,
  ChocolatierAction,
  ChocolatierEvent,
  ChocolatierView,
  End,
  HoodView,
  Ingredient,
} from "./types";
import "./enemy-chocolatier.css";

const BOARD = boardJson as BoardJson;

/** Card order of the recipes. */
const INGREDIENTS: Ingredient[] = ["Chocolate", "Sugar", "Nuts", "Love"];
export const INGREDIENT_NAME: Record<Ingredient, string> = {
  Chocolate: "chocolat",
  Sugar: "sucre",
  Nuts: "noisettes",
  Love: "amour",
};
export const ABILITY_NAME: Record<Ability, string> = {
  BuyThree: "Achat ×3",
  BuyClosed: "Achat fermé",
  BuyAnywhere: "Achat partout",
  AllDollars: "Tous $ +1",
  AllPoints: "Tous ★ +1",
  FiveDollarsPoint: "5 $ = 1 pt",
  TenDollarHouses: "Maisons à 10 $",
};
/** Counters of the seats, distinct from the colours of the houses. */
export const SEAT_FILL = ["#2463c9", "#2f9a48", "#f4f1ea", "#222026", "#8a4fd6", "#1fb5c9"];

const plural = (n: number, word: string) => `${n} ${word}${n > 1 ? "s" : ""}`;

type Point = [number, number];

/** Where a point of a section, drawn in its unturned frame, lands in the city. */
export function place(origin: Point, quarters: number, [x, y]: Point): Point {
  const [ox, oy] = origin;
  switch (((quarters % 4) + 4) % 4) {
    case 0:
      return [ox + x, oy + y];
    case 1:
      return [ox + BOARD.height - y, oy + x];
    case 2:
      return [ox + BOARD.width - x, oy + BOARD.height - y];
    default:
      return [ox + y, oy + BOARD.width - x];
  }
}

function placeRect(origin: Point, quarters: number, [x, y, w, h]: number[]) {
  const [ax, ay] = place(origin, quarters, [x, y]);
  const [bx, by] = place(origin, quarters, [x + w, y + h]);
  return { x: Math.min(ax, bx), y: Math.min(ay, by), w: Math.abs(bx - ax), h: Math.abs(by - ay) };
}

function colorClass(color: Ingredient | null | "Gray") {
  return `ec-${color ?? "Gray"}`;
}

export function hoodName(h: HoodView): string {
  return h.color ? `quartier ${INGREDIENT_NAME[h.color]}` : "quartier gris";
}

export function eventText(e: ChocolatierEvent, v: ChocolatierView, names: string[]): string {
  if (e === "RainCheck") return "prend un bon";
  if ("Income" in e) {
    const { dollars, points } = e.Income;
    return points > 0 ? `touche ${dollars} $ et ${plural(points, "point")}` : `touche ${dollars} $`;
  }
  if ("Bought" in e) {
    const { house, from, cost, rainCheck } = e.Bought;
    const hood = v.hoods[v.houses[house].hood];
    const whose = from === null ? "" : ` à ${names[from]}`;
    return `achète une maison (${hoodName(hood)}) ${cost} $${whose}${rainCheck ? " avec un bon" : ""}`;
  }
  const hood = v.hoods[e.Closed.hood];
  return `ferme le ${hoodName(hood)}${hood.ability ? ` (${ABILITY_NAME[hood.ability]})` : ""}`;
}

export function endText(end: End, names: string[]): string {
  if (end === "RoundLimit") return "Limite de 60 tours de table atteinte : classement aux points.";
  if ("Points" in end) return `${names[end.Points]} atteint 20 points et gagne !`;
  return `${names[end.Recipe]} complète sa recette secrète et gagne !`;
}

function Star({ x, y, r, value }: { x: number; y: number; r: number; value: number }) {
  const pts = Array.from({ length: 10 }, (_, i) => {
    const a = -Math.PI / 2 + (i * Math.PI) / 5;
    const rr = i % 2 === 0 ? r : r * 0.45;
    return `${x + rr * Math.cos(a)},${y + rr * Math.sin(a)}`;
  }).join(" ");
  return (
    <g className="ec-star">
      <polygon points={pts} />
      <text x={x} y={y + 2}>
        {value}
      </text>
    </g>
  );
}

function Bill({ x, y, value }: { x: number; y: number; value: number | string }) {
  return (
    <g className="ec-bill">
      <rect x={x - 32} y={y - 15} width={64} height={30} rx={5} />
      <text x={x} y={y + 1}>{`${value} $`}</text>
    </g>
  );
}

/** Where a hood's section lies, and its drawing. */
function frame(v: ChocolatierView, h: HoodView) {
  const slot = v.layout.findIndex((p) => p.section === h.section);
  return {
    origin: v.slots[slot],
    quarters: v.layout[slot].quarters,
    drawn: BOARD.sections[h.section].hoods[h.index],
  };
}

function City({ v, names, onBuy }: { v: ChocolatierView; names: string[]; onBuy: (house: number) => void }) {
  const offers = new Map(v.offers.map((o) => [o.house, o]));
  const factory = BOARD.factory.map(([x, y]) => `${v.factoryAt[0] + x},${v.factoryAt[1] + y}`).join(" ");
  const [fx, fy] = v.factoryAt;
  return (
    <svg className="ec-city" viewBox={`0 0 ${v.width} ${v.height}`} role="img" aria-label="La ville autour de l'usine">
      <rect x={0} y={0} width={v.width} height={v.height} className="ec-street" />
      <g className="ec-factory">
        <polygon points={factory} />
        <text x={fx + BOARD.width / 2} y={fy + BOARD.height / 2 - 30} className="ec-factory-name">
          Usine
        </text>
        <Bill x={fx + BOARD.width / 2} y={fy + BOARD.height / 2 + 40} value={2} />
      </g>
      {v.hoods.map((h, id) => {
        const { origin, quarters, drawn } = frame(v, h);
        const shape = drawn.shape.map((p) => place(origin, quarters, p).join(",")).join(" ");
        const closedBy = h.status.kind === "Closed" ? h.status.by : null;
        return (
          <g key={id} className={`ec-hood ${colorClass(h.color)} ${h.status.kind === "Unavailable" ? "ec-far" : ""}`}>
            <polygon
              points={shape}
              className="ec-hood-shape"
              style={closedBy === null ? undefined : { stroke: SEAT_FILL[closedBy], strokeWidth: 9 }}
            >
              <title>{`${hoodName(h)}${closedBy === null ? "" : ` fermé par ${names[closedBy]}`}`}</title>
            </polygon>
            {h.houses.map((house, k) => {
              const r = placeRect(origin, quarters, drawn.houses[k]);
              const info = v.houses[house];
              const offer = offers.get(house);
              const cx = r.x + r.w / 2;
              const cy = r.y + r.h / 2;
              return (
                <g
                  key={house}
                  className={`ec-house${offer ? " playable" : ""}`}
                  onClick={offer ? () => onBuy(house) : undefined}
                  role={offer ? "button" : undefined}
                  aria-label={offer ? `Acheter cette maison ${offer.cost} $` : undefined}
                >
                  <rect x={r.x + 3} y={r.y + 3} width={r.w - 6} height={r.h - 6} rx={6} className="ec-house-tile" />
                  <text x={cx} y={cy} className="ec-price">
                    {info.price}
                  </text>
                  {info.owner !== null && (
                    <circle
                      cx={r.x + r.w - 17}
                      cy={r.y + r.h - 17}
                      r={12}
                      className="ec-counter"
                      style={{ fill: SEAT_FILL[info.owner] }}
                    />
                  )}
                  {offer && (
                    <>
                      <rect x={r.x + 1} y={r.y + 1} width={r.w - 2} height={r.h - 2} rx={7} className="ec-target" />
                      {offer.cost !== info.price && (
                        <text x={r.x + 8} y={r.y + 16} className="ec-cost">{`${offer.cost} $`}</text>
                      )}
                    </>
                  )}
                </g>
              );
            })}
          </g>
        );
      })}
      {v.hoods.map((h, id) => {
        const { origin, quarters, drawn } = frame(v, h);
        return (
          <g key={id} className={h.status.kind === "Unavailable" ? "ec-far" : undefined}>
            {drawn.symbols.map((s, k) => {
              const [x, y] = place(origin, quarters, s.at);
              if (s.kind === "dollars") return <Bill key={k} x={x} y={y} value={s.value as number} />;
              if (s.kind === "points") return <Star key={k} x={x} y={y} r={26} value={s.value as number} />;
              const label = ABILITY_NAME[s.value as Ability];
              const w = 12 * label.length + 14;
              // A label keeps the room it had on the unturned section.
              const turn = quarters % 2 === 1 ? ` rotate(-90 ${x} ${y})` : "";
              return (
                <g key={k} className="ec-ability" transform={turn || undefined}>
                  <rect x={x - w / 2} y={y - 15} width={w} height={30} rx={5} />
                  <text x={x} y={y + 1}>
                    {label}
                  </text>
                </g>
              );
            })}
          </g>
        );
      })}
    </svg>
  );
}

function Ingredients({ have, need }: { have: number[]; need?: number[] }) {
  return (
    <span className="ec-ingredients">
      {INGREDIENTS.map((c, i) => (
        <span
          key={c}
          className={`ec-chip ${colorClass(c)}${need && have[i] >= need[i] ? " done" : ""}`}
          title={INGREDIENT_NAME[c]}
        >
          {need ? `${have[i]}/${need[i]}` : have[i]}
        </span>
      ))}
    </span>
  );
}

export function EnemyChocolatierBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as ChocolatierView;
  const legal = legalActions as ChocolatierAction[];
  const acting = me !== null && v.end === null && v.turn === me && legal.length > 0;
  const play = (a: ChocolatierAction) => submit(a);
  const mine = me === null ? null : v.seats[me];

  return (
    <div className="enemy-chocolatier">
      <ul className="ec-players">
        {v.seats.map((s, seat) => (
          <li key={seat} className={v.end === null && v.turn === seat ? "active" : undefined}>
            <span className="ec-token" style={{ background: SEAT_FILL[seat] }} />
            <span className="ec-name">{seatNames[seat]}</span>
            <span className="ec-meta">
              {`${s.money} $ · ${s.points}/${v.targetPoints} ★ · ${plural(s.rainChecks, "bon")} · revenu +${s.incomeDollars} $`}
              {s.incomePoints > 0 && ` +${s.incomePoints} ★`}
            </span>
            <Ingredients have={s.ingredients} need={s.recipe?.need} />
            {s.abilities.length > 0 && (
              <span className="ec-meta">{s.abilities.map((a) => ABILITY_NAME[a]).join(", ")}</span>
            )}
            {v.end && s.recipe && <span className="ec-meta ec-recipe-name">{s.recipe.name}</span>}
          </li>
        ))}
      </ul>

      <City v={v} names={seatNames} onBuy={(house) => acting && play({ Buy: house })} />

      {mine?.recipe && v.end === null && (
        <div className="ec-recipe">
          <span className="ec-label">Ma recette secrète</span>
          <span className="ec-recipe-name">{mine.recipe.name}</span>
          <Ingredients have={mine.ingredients} need={mine.recipe.need} />
        </div>
      )}

      {v.end ? (
        <p className="status">{endText(v.end.end, seatNames)}</p>
      ) : (
        <div className="ec-panel">
          {acting ? (
            <>
              <p className="hint">
                {`Achats : ${v.bought}/${v.limit}`}
                {mine && mine.rainChecks > 0 && ` (+ ${plural(mine.rainChecks, "bon")})`}
                {v.offers.length > 0 ? " · clique une maison surlignée pour l'acheter." : " · plus rien d'achetable."}
              </p>
              <button type="button" onClick={() => play("EndTurn")}>
                {v.bought === 0 ? "Ne rien acheter (prendre un bon)" : "Finir mon tour"}
              </button>
            </>
          ) : (
            <p className="hint">{`À ${seatNames[v.turn]} de jouer.`}</p>
          )}
          {v.roundsLeft <= 5 && <p className="hint">{`Plus que ${plural(v.roundsLeft, "tour")} de table avant la limite.`}</p>}
        </div>
      )}
      {v.end === null && v.events.length > 0 && (
        <p className="hint">{`Ce tour (${seatNames[v.turn]}) : ${v.events.map((e) => eventText(e, v, seatNames)).join(" ; ")}.`}</p>
      )}
      {v.lastTurn && (
        <p className="hint">
          {`Tour précédent (${seatNames[v.lastTurn.seat]}) : ${v.lastTurn.events.map((e) => eventText(e, v, seatNames)).join(" ; ")}.`}
        </p>
      )}
    </div>
  );
}
