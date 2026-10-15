import type { BoardProps } from "../../registry";
import { Card, CardBack } from "../cards";
import type { PendletonAction, PendletonEvent, PendletonView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describePendletonEvent(e: PendletonEvent, names: string[]): string {
  switch (e.type) {
    case "RoundStarted":
      return `Nouvelle manche, cibles ${e.targets.join(", ")}. ${names[e.first]} commence.`;
    case "Played":
      return `${names[e.seat]} pose un ${e.card} dans la colonne du ${e.target}.`;
    case "RoundScored": {
      const why =
        e.card === null
          ? "La pioche est vide"
          : `${names[e.seat]} pioche un ${e.card}, injouable partout`;
      const scores = e.round.map((points, i) => `${names[i]} ${plural(points, "point")}`).join(", ");
      const totals = e.totals.map((points, i) => `${names[i]} ${points}`).join(", ");
      return `${why} : fin de la manche. ${scores} (total : ${totals}).`;
    }
  }
}

export function PendletonBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as PendletonView;
  const legal = legalActions as PendletonAction[];
  const bottom = me ?? 0;
  const top = 1 - bottom;

  const seat = (i: number) => (
    <section className={`pairs-seat${v.turn === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span title="Points">
          {v.scores[i]} / {v.targetScore}
        </span>
      </h3>
    </section>
  );

  const side = (cards: number[], points: number, where: "top" | "bottom") => (
    <div className={`pendleton-side ${where}`}>
      {where === "bottom" && <span className="pendleton-points">{points}</span>}
      {(where === "top" ? [...cards].reverse() : cards).map((card, k) => (
        <Card key={k} rank={card} small />
      ))}
      {where === "top" && <span className="pendleton-points">{points}</span>}
    </div>
  );

  return (
    <div className="pendleton felt">
      <div className="pendleton-head">{seat(top)}</div>
      <div className="pendleton-columns">
        {v.columns.map((column, c) => {
          const move = legal.find((a) => a.Play.column === c);
          return (
            <div key={c} className="pendleton-column">
              {side(column.sides[top], column.points[top], "top")}
              <div className="pendleton-target">
                <Card
                  rank={column.target}
                  title={move && v.drawn !== null ? `Poser ton ${v.drawn} dans la colonne du ${column.target}` : undefined}
                  onClick={move ? () => submit(move) : undefined}
                />
              </div>
              {side(column.sides[bottom], column.points[bottom], "bottom")}
            </div>
          );
        })}
      </div>
      <div className="pendleton-head">{seat(bottom)}</div>
      <div className="pendleton-center">
        <p className="pairs-round">Manche {v.roundNo}</p>
        <div className="pairs-hand">
          <CardBack count={v.deckSize} label="Pioche" />
          {v.drawn !== null && v.turn !== null && (
            <span className="pairs-pile">
              <Card rank={v.drawn} />
              {seatNames[v.turn]} a pioché
            </span>
          )}
        </div>
        {legal.length > 0 && <p className="hint">Choisis la colonne de ton {v.drawn} : clique sur sa cible.</p>}
        {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
        <ul className="pairs-events">
          {v.events.map((e, i) => (
            <li key={i}>{describePendletonEvent(e, seatNames)}</li>
          ))}
        </ul>
      </div>
    </div>
  );
}
