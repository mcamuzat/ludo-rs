import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { TheCaveAction, TheCaveEvent, TheCaveView } from "./types";

export function describeTheCaveEvent(e: TheCaveEvent, all: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `Manche ${e.round} : ${all[e.dealer]} distribue, ${all[e.first]} commence.`;
    case "Played":
      return e.captured.length > 0
        ? `${all[e.seat]} pose un ${e.card} et capture ${e.captured.join(", ")}.`
        : `${all[e.seat]} pose un ${e.card} dans la grotte vide.`;
    case "Refilled":
      return `La grotte est complétée : ${e.cards.join(", ")}.`;
    case "RoundOver":
      return `Fin de la manche : ${e.points.map((p, i) => `${all[i]} +${p} (${e.scores[i]})`).join(", ")}.`;
  }
}

export function TheCaveBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as TheCaveView;
  const legal = legalActions as TheCaveAction[];

  const seats = v.seats.map((seat, i) => (
    <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span title="Score">{seat.score}</span>
      </h3>
      <div className="stack">
        {seat.captured.map((card, k) => (
          <Card key={k} rank={card} small />
        ))}
      </div>
      <p className="hint">Capturé : {seat.captured.length} cartes</p>
      <CardBack count={seat.handSize} label="en main" />
    </section>
  ));

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} sur {v.rounds}
      </p>
      <div className="pairs-line" aria-label="La grotte">
        {v.cave.map((card, k) => (
          <Card key={k} rank={card} />
        ))}
      </div>
      <p className="hint">La grotte</p>
      <CardBack count={v.deckSize} label="pioche" />
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per rank the engine accepts, on its first card.
            const move = v.hand.indexOf(card) === k ? legal.find((a) => a.Play.card === card) : undefined;
            return (
              <Card
                key={k}
                rank={card}
                title={move ? `Poser ce ${card} dans la grotte` : undefined}
                onClick={move ? () => submit(move) : undefined}
              />
            );
          })}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeTheCaveEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
