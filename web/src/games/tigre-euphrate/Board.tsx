import { useState } from "react";
import type { BoardProps } from "../registry";
import type { Color, Conflict, EndReason, FinalScore, TigreAction, TigreEvent, TigreView } from "./types";
import "./tigre-euphrate.css";

const W = 16;
const H = 11;
const S = 40;
const ALL: Color[] = ["Black", "Red", "Blue", "Green"];

export const COLOR_NAME: Record<Color, string> = { Black: "noir", Red: "rouge", Blue: "bleu", Green: "vert" };
export const LEADER_NAME: Record<Color, string> = { Black: "roi", Red: "prêtre", Blue: "fermier", Green: "marchand" };
const TILE_NAME: Record<Color, string> = { Black: "une population", Red: "un temple", Blue: "une ferme", Green: "un marché" };
const TILE_LETTER: Record<Color, string> = { Black: "P", Red: "T", Blue: "F", Green: "M" };
/** [R] The four dynasties, one per seat. */
export const DYNASTY = ["Lion", "Buffle", "Arc", "Poterie"];
// U+FE0E: the text form, not the emoji.
const GLYPH = ["♌\uFE0E", "♉\uFE0E", "♐\uFE0E", "⚱\uFE0E"];

export function cellName(c: number): string {
  return `${"ABCDEFGHIJKLMNOP"[c % W]}${Math.floor(c / W) + 1}`;
}

const plural = (n: number, word: string) => `${n} ${word}${n > 1 ? "s" : ""}`;

export function eventText(e: TigreEvent, names: string[]): string {
  if ("Tile" in e) {
    const { seat, color, at, scorer } = e.Tile;
    const point = scorer === null ? "" : ` (1 point ${COLOR_NAME[color]} pour ${names[scorer]})`;
    return `${names[seat]} pose ${TILE_NAME[color]} en ${cellName(at)}${point}`;
  }
  if ("Junction" in e) return `${names[e.Junction.seat]} réunit deux royaumes en ${cellName(e.Junction.at)}`;
  if ("Leader" in e) {
    const { seat, color, from, to } = e.Leader;
    return `${names[seat]} ${from === null ? "pose" : "déplace"} son ${LEADER_NAME[color]} en ${cellName(to)}`;
  }
  if ("Withdraw" in e) return `${names[e.Withdraw.seat]} reprend son ${LEADER_NAME[e.Withdraw.color]}`;
  if ("Catastrophe" in e) {
    const { seat, at, destroyed } = e.Catastrophe;
    return `${names[seat]} pose une catastrophe en ${cellName(at)}${destroyed ? ` (détruit ${TILE_NAME[destroyed]})` : ""}`;
  }
  if ("Swap" in e) return `${names[e.Swap.seat]} change ${plural(e.Swap.count, "tuile")}`;
  if ("Conflict" in e) {
    const c = e.Conflict;
    const removed = c.removed > 0 ? `, ${plural(c.removed, "tuile")} retirée${c.removed > 1 ? "s" : ""}` : "";
    return `${names[c.winner]} gagne le conflit des ${LEADER_NAME[c.color]}s (${c.attackerTotal} contre ${c.defenderTotal})${removed}`;
  }
  if ("Returned" in e) return `le ${LEADER_NAME[e.Returned.color]} de ${names[e.Returned.seat]} n'a plus de temple et rentre`;
  if ("Monument" in e) return `${names[e.Monument.seat]} bâtit un monument en ${cellName(e.Monument.at)}`;
  if ("NoMonument" in e) return `${names[e.NoMonument.seat]} ne bâtit pas de monument`;
  if ("Treasures" in e) return `${names[e.Treasures.seat]} prend ${plural(e.Treasures.count, "trésor")}`;
  return `${names[e.MonumentPoint.seat]} marque 1 point ${COLOR_NAME[e.MonumentPoint.color]} grâce à un monument`;
}

export function conflictText(c: Conflict, names: string[]): string {
  const kind = c.kind === "Internal" ? "interne" : "externe";
  const unit = c.kind === "Internal" ? "temples" : `tuiles ${COLOR_NAME[c.color]}es`;
  const added = c.attackerAdded === null ? "" : ` + ${c.attackerAdded}`;
  return `Conflit ${kind} des ${LEADER_NAME[c.color]}s (${unit}) : ${names[c.attacker]} attaque avec ${c.attackerBase}${added}, ${names[c.defender]} défend avec ${c.defenderBase}.`;
}

export function endText(end: EndReason): string {
  switch (end) {
    case "Treasures":
      return "Il ne reste qu'un ou deux trésors : la partie est finie.";
    case "Bag":
      return "Le sac ne suffit plus à refaire une main : la partie est finie.";
    case "Quiet":
      return "20 tours de table sans tuile, catastrophe ni échange : la partie est finie.";
  }
}

type Selection =
  | { kind: "tile"; color: Color }
  | { kind: "leader"; color: Color }
  | { kind: "catastrophe" }
  | { kind: "swap"; tiles: number[] }
  | null;

const same = (a: TigreAction, b: TigreAction) => JSON.stringify(a) === JSON.stringify(b);

function targets(legal: TigreAction[], sel: Selection): Map<number, TigreAction> {
  const m = new Map<number, TigreAction>();
  for (const a of legal) {
    if (typeof a !== "object") continue;
    if (sel?.kind === "tile" && "Tile" in a && a.Tile.color === sel.color) m.set(a.Tile.to, a);
    if (sel?.kind === "leader" && "Leader" in a && a.Leader.color === sel.color) m.set(a.Leader.to, a);
    if (sel?.kind === "catastrophe" && "Catastrophe" in a) m.set(a.Catastrophe.at, a);
    if ("LeaveTreasure" in a) m.set(a.LeaveTreasure.at, a);
  }
  return m;
}

function Scores({ scores, treasures }: { scores: number[]; treasures: number }) {
  return (
    <span className="te-scores">
      {ALL.map((c, i) => (
        <span key={c} className={`te-score te-${c}`} title={COLOR_NAME[c]}>
          {scores[i]}
        </span>
      ))}
      {treasures > 0 && <span className="te-score te-treasure-count" title="trésors">{`+${treasures}`}</span>}
    </span>
  );
}

function FinalTable({ scores, names }: { scores: FinalScore[]; names: string[] }) {
  const order = scores.map((s, i) => ({ ...s, seat: i })).sort((a, b) => a.rank - b.rank);
  return (
    <table className="te-final">
      <thead>
        <tr>
          <th />
          <th>Joueur</th>
          <th>Points (noir, rouge, bleu, vert)</th>
          <th>Trésors</th>
          <th>Couleur la plus faible</th>
        </tr>
      </thead>
      <tbody>
        {order.map((s) => (
          <tr key={s.seat}>
            <td>{s.rank + 1}</td>
            <td>{`${GLYPH[s.seat]} ${names[s.seat]}`}</td>
            <td>
              <Scores scores={s.scores} treasures={0} />
            </td>
            <td>{s.treasures}</td>
            <td>
              <strong>{s.weakest}</strong>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function TigreEuphrateBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as TigreView;
  const legal = legalActions as TigreAction[];
  const deciding = me !== null && v.deciding === me && legal.length > 0;
  const [sel, setSel] = useState<Selection>(null);
  const has = (a: TigreAction) => legal.some((l) => same(l, a));
  const phase = v.phase;
  const acting = deciding && phase === "Action";
  const selection = acting ? sel : null;
  const reachable = deciding ? targets(legal, selection) : new Map<number, TigreAction>();

  const play = (a: TigreAction) => {
    setSel(null);
    submit(a);
  };

  const onCell = (c: number) => {
    const a = reachable.get(c);
    if (a) return play(a);
    const piece = v.cells[c].piece;
    if (acting && piece && typeof piece === "object" && "Leader" in piece && piece.Leader.seat === me) {
      const color = piece.Leader.color;
      setSel(sel?.kind === "leader" && sel.color === color ? null : { kind: "leader", color });
    }
  };

  const monumentSquares = typeof phase === "object" && "Monument" in phase ? phase.Monument.squares : [];
  const conflict = typeof phase === "object" && "Reinforce" in phase ? phase.Reinforce.conflict : null;
  const selectedLeader = selection?.kind === "leader" ? selection.color : null;

  return (
    <div className="tigre-euphrate">
      <ul className="te-players">
        {Array.from({ length: v.players }, (_, seat) => (
          <li key={seat} className={v.turn === seat ? "active" : undefined}>
            <span className="te-glyph">{GLYPH[seat]}</span>
            <span className="te-name">{`${seatNames[seat]} (${DYNASTY[seat]})`}</span>
            <span className="te-meta">
              {`${plural(v.handSizes[seat], "tuile")} · ${plural(v.catastrophes[seat], "catastrophe")} · ${plural(
                v.treasures[seat],
                "trésor",
              )}`}
              {v.supply[seat].length > 0 && ` · en réserve : ${v.supply[seat].map((c) => LEADER_NAME[c]).join(", ")}`}
            </span>
          </li>
        ))}
      </ul>

      <svg className="te-board felt" viewBox={`-4 -4 ${W * S + 8} ${H * S + 8}`} role="img" aria-label="Plateau de Tigre et Euphrate">
        {v.cells.map((cell, c) => {
          const x = (c % W) * S;
          const y = Math.floor(c / W) * S;
          const target = reachable.has(c);
          const piece = cell.piece;
          const mine = acting && piece && typeof piece === "object" && "Leader" in piece && piece.Leader.seat === me;
          const clickable = target || !!mine;
          return (
            <g
              key={c}
              className={`te-cell${clickable ? " playable" : ""}`}
              onClick={clickable ? () => onCell(c) : undefined}
              role={clickable ? "button" : undefined}
              aria-label={clickable ? cellName(c) : undefined}
            >
              <rect x={x} y={y} width={S} height={S} className={cell.river ? "te-river" : "te-land"} />
              {piece === "Catastrophe" && (
                <>
                  <rect x={x + 3} y={y + 3} width={S - 6} height={S - 6} rx={4} className="te-catastrophe" />
                  <text x={x + S / 2} y={y + S / 2} className="te-mark">
                    ✕
                  </text>
                </>
              )}
              {piece && typeof piece === "object" && "Tile" in piece && (
                <>
                  <rect
                    x={x + 3}
                    y={y + 3}
                    width={S - 6}
                    height={S - 6}
                    rx={4}
                    className={piece.Tile.flipped ? "te-tile te-flipped" : `te-tile te-${piece.Tile.color}`}
                  />
                  {!piece.Tile.flipped && (
                    <text x={x + S / 2} y={y + S / 2} className="te-letter">
                      {TILE_LETTER[piece.Tile.color]}
                    </text>
                  )}
                </>
              )}
              {piece && typeof piece === "object" && "Leader" in piece && (
                <>
                  <circle
                    cx={x + S / 2}
                    cy={y + S / 2}
                    r={S / 2 - 5}
                    className={`te-leader te-${piece.Leader.color}${
                      (selectedLeader === piece.Leader.color && piece.Leader.seat === me) ||
                      (conflict && (conflict.attackerAt === c || conflict.defenderAt === c))
                        ? " selected"
                        : ""
                    }`}
                  />
                  <text x={x + S / 2} y={y + S / 2 + 1} className="te-mark">
                    {GLYPH[piece.Leader.seat]}
                  </text>
                </>
              )}
              {cell.treasure && (
                <rect
                  x={x + S - 13}
                  y={y + 4}
                  width={8}
                  height={8}
                  transform={`rotate(45 ${x + S - 9} ${y + 8})`}
                  className={`te-treasure${cell.corner ? " corner" : ""}`}
                />
              )}
              {target && <rect x={x + 1.5} y={y + 1.5} width={S - 3} height={S - 3} rx={5} className="te-target" />}
            </g>
          );
        })}
        {v.monuments.map((m, i) => {
          if (m.at === null) return null;
          const x = (m.at % W) * S + S / 2;
          const y = Math.floor(m.at / W) * S + S / 2;
          return (
            <g key={i} className="te-monument" aria-label={`monument ${COLOR_NAME[m.colors[0]]} et ${COLOR_NAME[m.colors[1]]}`}>
              <rect x={x} y={y} width={S / 2} height={S} className={`te-${m.colors[0]}`} />
              <rect x={x + S / 2} y={y} width={S / 2} height={S} className={`te-${m.colors[1]}`} />
              <rect x={x} y={y} width={S} height={S} className="te-monument-frame" />
              <rect x={x + S / 4} y={y + S / 4} width={S / 2} height={S / 2} className="te-monument-top" />
            </g>
          );
        })}
        {monumentSquares.map((sq) => (
          <rect
            key={sq}
            x={(sq % W) * S + 1}
            y={Math.floor(sq / W) * S + 1}
            width={2 * S - 2}
            height={2 * S - 2}
            rx={6}
            className="te-target"
          />
        ))}
        {v.junction !== null && (
          <rect
            x={(v.junction % W) * S + 1}
            y={Math.floor(v.junction / W) * S + 1}
            width={S - 2}
            height={S - 2}
            rx={6}
            className="te-junction"
          />
        )}
      </svg>

      {v.hand && (
        <div className="te-hand">
          <span className="te-label">Ma main</span>
          {ALL.map((c, i) => (
            <button
              key={c}
              type="button"
              className={`te-pick te-${c}${selection?.kind === "tile" && selection.color === c ? " selected" : ""}`}
              disabled={!acting || v.hand![i] === 0 || !legal.some((a) => typeof a === "object" && "Tile" in a && a.Tile.color === c)}
              aria-pressed={selection?.kind === "tile" && selection.color === c}
              aria-label={`${v.hand![i]} ${TILE_LETTER[c]}`}
              onClick={() => setSel({ kind: "tile", color: c })}
            >
              {v.hand![i]}
            </button>
          ))}
          {v.myScores && (
            <>
              <span className="te-label">Mes points</span>
              <Scores scores={v.myScores} treasures={me === null ? 0 : v.treasures[me]} />
            </>
          )}
        </div>
      )}

      {v.end ? (
        <>
          <p className="status">{endText(v.end)}</p>
          {v.finalScores && <FinalTable scores={v.finalScores} names={seatNames} />}
        </>
      ) : (
        <div className="te-panel">
          {acting && (
            <>
              <div className="te-actions">
                {ALL.map((c) => {
                  const placeable = legal.some((a) => typeof a === "object" && "Leader" in a && a.Leader.color === c);
                  return (
                    <button
                      key={c}
                      type="button"
                      className={`te-leader-pick te-${c}${selection?.kind === "leader" && selection.color === c ? " selected" : ""}`}
                      disabled={!placeable && !has({ Withdraw: { color: c } })}
                      aria-pressed={selection?.kind === "leader" && selection.color === c}
                      onClick={() => setSel({ kind: "leader", color: c })}
                    >
                      {`${GLYPH[me!]} ${LEADER_NAME[c]}`}
                    </button>
                  );
                })}
                <button
                  type="button"
                  className={selection?.kind === "catastrophe" ? "selected" : undefined}
                  disabled={!legal.some((a) => typeof a === "object" && "Catastrophe" in a)}
                  onClick={() => setSel({ kind: "catastrophe" })}
                >
                  {`Catastrophe (${v.catastrophes[me!]})`}
                </button>
                <button type="button" onClick={() => setSel({ kind: "swap", tiles: [0, 0, 0, 0] })}>
                  Changer des tuiles
                </button>
                <button type="button" onClick={() => play("EndTurn")}>
                  {v.actionsLeft === 2 ? "Passer mon tour" : "Finir mon tour"}
                </button>
              </div>
              {selection?.kind === "leader" && has({ Withdraw: { color: selection.color } }) && (
                <button type="button" onClick={() => play({ Withdraw: { color: selection.color } })}>
                  {`Reprendre mon ${LEADER_NAME[selection.color]}`}
                </button>
              )}
              {selection?.kind === "swap" && (
                <div className="te-swap">
                  {ALL.map((c, i) => (
                    <span key={c} className="te-swap-color">
                      <span className={`te-pick small te-${c}`}>{selection.tiles[i]}</span>
                      <button
                        type="button"
                        disabled={selection.tiles[i] >= (v.hand?.[i] ?? 0)}
                        onClick={() => setSel({ kind: "swap", tiles: selection.tiles.map((n, j) => (j === i ? n + 1 : n)) })}
                        aria-label={`une ${COLOR_NAME[c]} de plus`}
                      >
                        +
                      </button>
                      <button
                        type="button"
                        disabled={selection.tiles[i] === 0}
                        onClick={() => setSel({ kind: "swap", tiles: selection.tiles.map((n, j) => (j === i ? n - 1 : n)) })}
                        aria-label={`une ${COLOR_NAME[c]} de moins`}
                      >
                        −
                      </button>
                    </span>
                  ))}
                  <button
                    type="button"
                    disabled={!has({ Swap: { tiles: selection.tiles } })}
                    onClick={() => play({ Swap: { tiles: selection.tiles } })}
                  >
                    {`Défausser et repiocher ${plural(selection.tiles.reduce((a, b) => a + b, 0), "tuile")}`}
                  </button>
                </div>
              )}
              <p className="hint">
                {selection === null
                  ? `${plural(v.actionsLeft, "action")} : choisis une tuile de ta main, un chef (aussi en cliquant le tien sur le plateau), une catastrophe ou un échange.`
                  : selection.kind === "tile"
                    ? `Choisis où poser ${TILE_NAME[selection.color]}.`
                    : selection.kind === "leader"
                      ? `Choisis où poser ton ${LEADER_NAME[selection.color]} : à côté d'un temple, hors des rivières.`
                      : selection.kind === "catastrophe"
                        ? "Choisis où poser la catastrophe."
                        : "Choisis les tuiles à défausser."}
              </p>
            </>
          )}
          {conflict && (
            <>
              <p className="hint">{conflictText(conflict, seatNames)}</p>
              {deciding && typeof phase === "object" && "Reinforce" in phase && (
                <div className="te-actions">
                  {phase.Reinforce.options.map((n) => (
                    <button key={n} type="button" onClick={() => play({ Reinforce: { count: n } })}>
                      {n === 0 ? "Pas de renfort" : `Ajouter ${n}`}
                    </button>
                  ))}
                </div>
              )}
            </>
          )}
          {typeof phase === "object" && "ChooseConflict" in phase && (
            <>
              <p className="hint">Plusieurs conflits : choisis lequel se résout d'abord.</p>
              {deciding && (
                <div className="te-actions">
                  {phase.ChooseConflict.colors.map((c) => (
                    <button key={c} type="button" onClick={() => play({ Resolve: { color: c } })}>
                      {`Les ${LEADER_NAME[c]}s`}
                    </button>
                  ))}
                </div>
              )}
            </>
          )}
          {monumentSquares.length > 0 && (
            <>
              <p className="hint">Un carré de quatre tuiles est complet : on peut y bâtir un monument d'une de ses couleurs.</p>
              {deciding && (
                <div className="te-actions">
                  {legal.map((a) =>
                    typeof a === "object" && "Monument" in a ? (
                      <button key={`${a.Monument.square}-${a.Monument.monument}`} type="button" onClick={() => play(a)}>
                        {`Monument ${v.monuments[a.Monument.monument].colors.map((c) => COLOR_NAME[c]).join("-")}${
                          monumentSquares.length > 1 ? ` en ${cellName(a.Monument.square)}` : ""
                        }`}
                      </button>
                    ) : null,
                  )}
                  <button type="button" onClick={() => play("NoMonument")}>
                    Ne pas bâtir
                  </button>
                </div>
              )}
            </>
          )}
          {typeof phase === "object" && "Treasure" in phase && (
            <p className="hint">
              {deciding
                ? "Ton marchand prend les trésors du royaume sauf un : clique celui qui reste."
                : `${seatNames[phase.Treasure.seat]} choisit le trésor qui reste.`}
            </p>
          )}
          {!deciding && v.deciding !== null && phase === "Action" && <p className="hint">{`À ${seatNames[v.deciding]} de jouer.`}</p>}
          <p className="hint">
            {`Sac : ${plural(v.bag, "tuile")} · trésors sur le plateau : ${v.treasuresLeft}`}
            {v.quietLeft <= 5 && ` · plus que ${v.quietLeft} tour(s) de table sans tuile avant la fin`}
          </p>
        </div>
      )}
      {v.turn !== null && v.steps.length > 0 && (
        <p className="hint">{`Ce tour : ${v.steps.map((e) => eventText(e, seatNames)).join(" ; ")}.`}</p>
      )}
      {v.lastTurn && (
        <p className="hint">
          {`Tour précédent (${seatNames[v.lastTurn.seat]}) : ${
            v.lastTurn.steps.length > 0 ? v.lastTurn.steps.map((e) => eventText(e, seatNames)).join(" ; ") : "rien"
          }.`}
        </p>
      )}
    </div>
  );
}
