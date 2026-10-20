import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { TheJudgeAction, TheJudgeEvent, TheJudgeView } from "./types";

/** "Ana", "Ana et Hugo", "Ana, Hugo et Cléo". */
function names(seats: number[], all: string[]): string {
  const named = seats.map((s) => all[s]);
  return named.length > 1 ? `${named.slice(0, -1).join(", ")} et ${named[named.length - 1]}` : named.join("");
}

export function describeTheJudgeEvent(e: TheJudgeEvent, all: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `Manche ${e.round} : 7 cartes chacun, 8 pour le Juge. ${all[e.leader]} commence.`;
    case "Added":
      return `${all[e.seat]} ajoute un ${e.card} à sa recette.`;
    case "Passed":
      return `${all[e.seat]} passe.`;
    case "Contest": {
      const reveals = e.reveals
        .map(({ card, eliminated }) =>
          eliminated.length > 0 ? `${card} (élimine ${names(eliminated, all)})` : `${card}`,
        )
        .join(", ");
      const judge = e.reveals.length > 0 ? `Le Juge retourne : ${reveals}. ` : "Le Juge n'a rien à retourner. ";
      const scored =
        e.survivors.length > 0
          ? `Marquent : ${e.survivors.map((s) => `${all[s]} +${e.gained[s]} (${e.scores[s]})`).join(", ")}.`
          : "Personne ne marque.";
      return judge + scored;
    }
  }
}

export function TheJudgeBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as TheJudgeView;
  const legal = legalActions as TheJudgeAction[];

  const seats = v.seats.map((seat, i) => (
    <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span title="Score">{seat.score}</span>
      </h3>
      <div className="stack">
        {seat.recipe.map((card, k) => (
          <Card key={k} rank={card} />
        ))}
      </div>
      <p className="hint">
        Recette : {seat.recipePoints} points{seat.passed ? " · a fini" : ""}
      </p>
      <CardBack count={seat.handSize} label="en main" />
    </section>
  ));

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} · Partie en {v.target} points
      </p>
      <CardBack count={v.judgeSize} label="le Juge" />
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per rank the engine accepts, on its first card.
            const move =
              v.hand.indexOf(card) === k
                ? legal.find((a) => typeof a === "object" && a.Add.card === card)
                : undefined;
            return (
              <Card
                key={k}
                rank={card}
                title={move ? `Ajouter ce ${card} à ta recette` : undefined}
                onClick={move ? () => submit(move) : undefined}
              />
            );
          })}
        </div>
      )}
      {legal.includes("Pass") && (
        <div className="pairs-moves">
          <button type="button" onClick={() => submit("Pass")}>
            J'ai fini de cuisiner
          </button>
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeTheJudgeEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
