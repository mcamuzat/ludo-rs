import { useState } from "react";
import type { BoardProps } from "../registry";
import { COLOR_NAME, LEADER_NAME } from "../tigre-euphrate/Board";
import type { Color, Conflict, EndReason, Face, FinalScore, Pos, Slot, TecAction, TecEvent, TecView } from "./types";
import "./tigre-euphrate-cartes.css";

const ALL: Color[] = ["Black", "Red", "Blue", "Green"];
/** [R] The four dynasties, one per seat. */
export const DYNASTY = ["Arc", "Lion", "Taureau", "Vase"];
// U+FE0E: the text form, not the emoji.
const GLYPH = ["♐︎", "♌︎", "♉︎", "⚱︎"];
const CARD_NAME: Record<Color, string> = { Black: "un peuple", Red: "un temple", Blue: "une agriculture", Green: "un marché" };
const CARD_LETTER: Record<Color, string> = { Black: "P", Red: "T", Blue: "A", Green: "M" };
const MAX_ROWS = 8;

const plural = (n: number, word: string) => `${n} ${word}${n > 1 ? "s" : ""}`;
const key = (p: Pos) => JSON.stringify(p);

export function posName(p: Pos): string {
  if ("Head" in p) return `le trésor ${p.Head + 1}`;
  if ("Gap" in p) return `la jonction ${p.Gap + 1}-${p.Gap + 2}`;
  return `la colonne ${p.Card[0] + 1}, carte ${p.Card[1] + 1}`;
}

const shipName = (c: Color) => `le navire bleu-${COLOR_NAME[c]}`;

export function eventText(e: TecEvent, names: string[]): string {
  if ("Leader" in e) {
    const { seat, color, moved, to } = e.Leader;
    return `${names[seat]} ${moved ? "déplace" : "pose"} son ${LEADER_NAME[color]} sur ${posName(to)}`;
  }
  if ("Extend" in e) {
    const { seat, color, column, scorer } = e.Extend;
    const point = scorer === null ? "" : ` (1 point ${COLOR_NAME[color]} pour ${names[scorer]})`;
    return `${names[seat]} pose ${CARD_NAME[color]} en colonne ${column + 1}${point}`;
  }
  if ("Join" in e) return `${names[e.Join.seat]} relie deux royaumes en ${e.Join.gap + 1}-${e.Join.gap + 2}`;
  if ("Catastrophe" in e) {
    const { seat, at, color } = e.Catastrophe;
    return `${names[seat]} détruit ${CARD_NAME[color]} (${posName(at)})`;
  }
  if ("Conflict" in e) {
    const c = e.Conflict;
    const taken = c.removed > 0 ? `, ${plural(c.removed, "carte")} prise${c.removed > 1 ? "s" : ""}` : "";
    return `${names[c.winner]} gagne le conflit des ${LEADER_NAME[c.color]}s (${c.attackerTotal} contre ${c.defenderTotal})${taken}`;
  }
  if ("Ship" in e) return `${names[e.Ship.seat]} bâtit ${shipName(e.Ship.ship)} en colonne ${e.Ship.column + 1}`;
  if ("Returned" in e) return `le ${LEADER_NAME[e.Returned.color]} de ${names[e.Returned.seat]} rentre`;
  if ("Treasure" in e) return `le marchand de ${names[e.Treasure.seat]} prend le trésor ${e.Treasure.column + 1}`;
  return `${names[e.ShipPoint.seat]} marque 1 point ${COLOR_NAME[e.ShipPoint.color]} grâce à un navire`;
}

export function conflictText(c: Conflict, names: string[]): string {
  const kind = c.kind === "Internal" ? "interne" : "externe";
  const unit = c.kind === "Internal" ? "cartes rouges" : `cartes ${COLOR_NAME[c.color]}es`;
  const added = c.attackerAdded === null ? "" : ` + ${c.attackerAdded}`;
  return `Conflit ${kind} des ${LEADER_NAME[c.color]}s (${unit}) : ${names[c.attacker]} attaque avec ${c.attackerBase}${added}, ${names[c.defender]} défend avec ${c.defenderBase}.`;
}

export function endText(end: EndReason): string {
  switch (end) {
    case "Treasures":
      return "Il ne reste qu'un trésor sur la table : la partie est finie.";
    case "Deck":
      return "La pioche ne suffit plus à compléter les mains : la partie est finie.";
    case "Quiet":
      return "20 tours de table sans pioche : la partie est finie.";
  }
}

type Selection = { kind: "card"; color: Color } | { kind: "leader"; color: Color } | { kind: "catastrophe" } | null;

/** Where a click plays, for the current selection. Column ends are `Card` positions one past the last card. */
function targets(legal: TecAction[], sel: Selection, columns: Slot[][]): Map<string, TecAction> {
  const m = new Map<string, TecAction>();
  for (const a of legal) {
    if (typeof a !== "object") continue;
    if (sel?.kind === "card" && "Extend" in a && a.Extend.color === sel.color) {
      m.set(key({ Card: [a.Extend.column, columns[a.Extend.column].length] }), a);
    }
    if (sel?.kind === "card" && "Join" in a && a.Join.color === sel.color) m.set(key({ Gap: a.Join.gap }), a);
    if (sel?.kind === "leader" && "Leader" in a && a.Leader.color === sel.color) m.set(key(a.Leader.to), a);
    if (sel?.kind === "catastrophe" && "Catastrophe" in a) m.set(key(a.Catastrophe.at), a);
  }
  return m;
}

function faceClass(f: Face): string {
  if (f === "Treasure") return "tc-Red tc-treasure";
  if (f === "Exchanged") return "tc-Red tc-exchanged";
  if ("Civ" in f) return `tc-${f.Civ}`;
  return `tc-ship tc-ship-${f.Ship}`;
}

function faceLabel(f: Face): string {
  if (f === "Treasure") return "trésor";
  if (f === "Exchanged") return "temple (trésor pris)";
  if ("Civ" in f) return CARD_NAME[f.Civ];
  return shipName(f.Ship);
}

function faceMark(f: Face): string {
  if (f === "Treasure") return "✦";
  if (f === "Exchanged") return "T";
  if ("Civ" in f) return CARD_LETTER[f.Civ];
  return "⛵︎";
}

interface CardProps {
  slot: Slot | null;
  pos: Pos;
  target: boolean;
  faceDown?: boolean;
  selectedLeader: boolean;
  onClick?: () => void;
}

function Card({ slot, pos, target, faceDown, selectedLeader, onClick }: CardProps) {
  const label = slot ? `${faceLabel(slot.face)}, ${posName(pos)}` : posName(pos);
  const cls = slot
    ? `tc-card ${faceDown ? "tc-facedown" : faceClass(slot.face)}${target ? " target" : ""}`
    : `tc-card tc-empty${target ? " target" : ""}`;
  const content = (
    <>
      {slot && !faceDown && <span className="tc-mark">{faceMark(slot.face)}</span>}
      {slot?.leader && (
        <span className={`tc-leader tc-${slot.leader.color}${selectedLeader ? " selected" : ""}`} title={LEADER_NAME[slot.leader.color]}>
          {GLYPH[slot.leader.seat]}
        </span>
      )}
    </>
  );
  return onClick ? (
    <button type="button" className={cls} onClick={onClick} aria-label={label} data-pos={key(pos)}>
      {content}
    </button>
  ) : (
    <div className={cls} aria-label={slot ? label : undefined} data-pos={key(pos)}>
      {content}
    </div>
  );
}

function Scores({ scores, treasures }: { scores: number[]; treasures: number }) {
  return (
    <span className="tc-scores">
      {ALL.map((c, i) => (
        <span key={c} className={`tc-score tc-${c}`} title={`points ${COLOR_NAME[c]}s`}>
          {scores[i]}
        </span>
      ))}
      {treasures > 0 && (
        <span className="tc-score tc-treasure-count" title="trésors">
          {`+${treasures}`}
        </span>
      )}
    </span>
  );
}

function FinalTable({ scores, names }: { scores: FinalScore[]; names: string[] }) {
  const order = scores.map((s, i) => ({ ...s, seat: i })).sort((a, b) => a.rank - b.rank);
  return (
    <table className="tc-final">
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

export function TigreEuphrateCartesBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as TecView;
  const legal = legalActions as TecAction[];
  const deciding = me !== null && v.deciding === me && legal.length > 0;
  const [sel, setSel] = useState<Selection>(null);
  const phase = v.phase;
  const acting = deciding && phase === "Action";
  const selection = acting ? sel : null;
  const reachable = acting ? targets(legal, selection, v.columns) : new Map<string, TecAction>();
  const selectedLeader = selection?.kind === "leader" ? selection.color : null;

  const play = (a: TecAction) => {
    setSel(null);
    submit(a);
  };
  const toggle = (s: Selection) => setSel(JSON.stringify(sel) === JSON.stringify(s) ? null : s);

  const cardAt = (slot: Slot | null, pos: Pos, faceDown = false) => {
    const a = reachable.get(key(pos));
    const mine = acting && slot?.leader?.seat === me ? slot.leader.color : null;
    const onClick = a ? () => play(a) : mine ? () => toggle({ kind: "leader", color: mine }) : undefined;
    return (
      <Card
        key={key(pos)}
        slot={slot}
        pos={pos}
        target={!!a}
        faceDown={faceDown}
        selectedLeader={mine !== null && mine === selectedLeader}
        onClick={onClick}
      />
    );
  };

  const conflict = typeof phase === "object" && "Conflict" in phase ? phase.Conflict : null;
  const rows = Math.max(MAX_ROWS, ...v.columns.map((c) => c.length + 1));

  return (
    <div className="tigre-euphrate-cartes">
      <ul className="tc-players">
        {Array.from({ length: v.players }, (_, seat) => (
          <li key={seat} className={v.turn === seat ? "active" : undefined}>
            <span className="tc-glyph">{GLYPH[seat]}</span>
            <span className="tc-name">{`${seatNames[seat]} (${DYNASTY[seat]})`}</span>
            <Scores scores={v.scores[seat]} treasures={v.treasures[seat]} />
            <span className="tc-meta">
              {plural(v.handSizes[seat], "carte")}
              {v.catastrophes[seat] && " · catastrophe"}
              {v.supply[seat].length > 0 && ` · en réserve : ${v.supply[seat].map((c) => LEADER_NAME[c]).join(", ")}`}
            </span>
          </li>
        ))}
      </ul>

      <div className="tc-table felt" role="group" aria-label="Table de Tigre et Euphrate">
        {v.heads.map((h, c) => (
          <div key={`h${c}`} className="tc-cell" style={{ gridColumn: 2 * c + 1, gridRow: 1 }}>
            {cardAt(h, { Head: c })}
          </div>
        ))}
        {v.gaps.map((g, i) => {
          const pos: Pos = { Gap: i };
          if (!g && !reachable.has(key(pos))) return null;
          return (
            <div key={`g${i}`} className="tc-cell tc-gap" style={{ gridColumn: 2 * i + 2, gridRow: 1 }}>
              {cardAt(g, pos, v.join === i)}
            </div>
          );
        })}
        {v.columns.map((col, c) =>
          Array.from({ length: rows }, (_, r) => {
            const pos: Pos = { Card: [c, r] };
            const slot = col[r] ?? null;
            if (!slot && !reachable.has(key(pos))) return null;
            return (
              <div key={`c${c}-${r}`} className="tc-cell" style={{ gridColumn: 2 * c + 1, gridRow: r + 2 }}>
                {cardAt(slot, pos)}
              </div>
            );
          }),
        )}
      </div>

      {v.hand && (
        <div className="tc-hand">
          <span className="tc-label">Ma main</span>
          {ALL.map((c, i) => {
            const playable = legal.some((a) => typeof a === "object" && (("Extend" in a && a.Extend.color === c) || ("Join" in a && a.Join.color === c)));
            const picked = selection?.kind === "card" && selection.color === c;
            return (
              <button
                key={c}
                type="button"
                className={`tc-pick tc-${c}${picked ? " selected" : ""}`}
                disabled={!acting || !playable}
                aria-pressed={picked}
                aria-label={`${v.hand![i]} ${CARD_NAME[c].replace(/^une? /, "")}`}
                onClick={() => toggle({ kind: "card", color: c })}
              >
                {v.hand![i]}
              </button>
            );
          })}
        </div>
      )}

      {v.end ? (
        <>
          <p className="status">{endText(v.end)}</p>
          {v.finalScores && <FinalTable scores={v.finalScores} names={seatNames} />}
        </>
      ) : (
        <div className="tc-panel">
          {acting && (
            <>
              <div className="tc-actions">
                {ALL.map((c) => {
                  const movable = legal.some((a) => typeof a === "object" && "Leader" in a && a.Leader.color === c);
                  const picked = selection?.kind === "leader" && selection.color === c;
                  return (
                    <button
                      key={c}
                      type="button"
                      className={`tc-leader-pick tc-${c}${picked ? " selected" : ""}`}
                      disabled={!movable}
                      aria-pressed={picked}
                      onClick={() => toggle({ kind: "leader", color: c })}
                    >
                      {`${GLYPH[me!]} ${LEADER_NAME[c]}`}
                    </button>
                  );
                })}
                <button
                  type="button"
                  className={selection?.kind === "catastrophe" ? "selected" : undefined}
                  disabled={!legal.some((a) => typeof a === "object" && "Catastrophe" in a)}
                  onClick={() => toggle({ kind: "catastrophe" })}
                >
                  Catastrophe
                </button>
                <button type="button" onClick={() => play("EndTurn")}>
                  {v.actionsLeft === 2 ? "Passer mon tour" : "Finir mon tour"}
                </button>
              </div>
              <p className="hint">
                {selection === null
                  ? `${plural(v.actionsLeft, "action")} : choisis une carte de ta main, un chef (aussi en cliquant le tien sur la table) ou ta catastrophe.`
                  : selection.kind === "card"
                    ? `Pose ${CARD_NAME[selection.color]} au bas d'une colonne, ou entre deux trésors pour relier deux royaumes.`
                    : selection.kind === "leader"
                      ? `Choisis une carte libre pour ton ${LEADER_NAME[selection.color]}.`
                      : "Choisis la carte à détruire."}
              </p>
            </>
          )}
          {conflict && (
            <>
              <p className="hint">{conflictText(conflict.conflict, seatNames)}</p>
              {deciding && conflict.most !== null && (
                <div className="tc-actions">
                  {Array.from({ length: conflict.most + 1 }, (_, n) => (
                    <button key={n} type="button" onClick={() => play({ Commit: { count: n } })}>
                      {n === 0 ? "Ne rien ajouter" : `Ajouter ${n}`}
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
                <div className="tc-actions">
                  {phase.ChooseConflict.colors.map((c) => (
                    <button key={c} type="button" onClick={() => play({ Resolve: { color: c } })}>
                      {`Les ${LEADER_NAME[c]}s`}
                    </button>
                  ))}
                </div>
              )}
            </>
          )}
          {typeof phase === "object" && "Ship" in phase && (
            <>
              <p className="hint">{`Quatre cartes d'une couleur en colonne ${phase.Ship.column + 1} : on peut y bâtir un navire.`}</p>
              {deciding && (
                <div className="tc-actions">
                  {phase.Ship.options.map((c) => (
                    <button key={c} type="button" onClick={() => play({ Ship: { ship: c } })}>
                      {`Bâtir ${shipName(c)}`}
                    </button>
                  ))}
                  <button type="button" onClick={() => play("NoShip")}>
                    Ne pas bâtir
                  </button>
                </div>
              )}
            </>
          )}
          {!deciding && v.deciding !== null && <p className="hint">{`À ${seatNames[v.deciding]} de jouer.`}</p>}
          <p className="hint">
            {`Pioche : ${plural(v.deck, "carte")} · trésors sur la table : ${v.treasuresLeft}`}
            {v.ships.length > 0 && ` · navires à bâtir : ${v.ships.map((c) => `bleu-${COLOR_NAME[c]}`).join(", ")}`}
            {v.quietLeft <= 5 && ` · plus que ${v.quietLeft} tour(s) sans pioche avant la fin`}
          </p>
        </div>
      )}
      {v.turn !== null && v.steps.length > 0 && <p className="hint">{`Ce tour : ${v.steps.map((e) => eventText(e, seatNames)).join(" ; ")}.`}</p>}
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
