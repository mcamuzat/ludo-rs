import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { BreadBasketAction, BreadBasketEvent, BreadBasketView, BreadCard } from "./types";

/** A number with a typographic minus. */
function num(n: number): string {
  return n < 0 ? `−${-n}` : `${n}`;
}

function signed(n: number): string {
  return n < 0 ? num(n) : `+${n}`;
}

function cardName(card: BreadCard): string {
  return card === -5 ? "un Oignon" : `un ${card}`;
}

export function describeBreadBasketEvent(e: BreadBasketEvent, all: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `Main ${e.round} : ${all[e.dealer]} distribue, ${all[e.first]} commence.`;
    case "Played":
      return `${all[e.seat]} pose ${cardName(e.card)}.`;
    case "Sandwich":
      return `${all[e.seat]} fait un sandwich ${e.bread === -5 ? "à l'Oignon" : `au ${e.bread}`} : ${e.filling
        .map(num)
        .join(", ")} (${signed(e.points)}).`;
    case "Air":
      return `${all[e.seat]} fait un sandwich d'air ${e.card === -5 ? "d'Oignons" : `de ${e.card}`} et doit piocher.`;
    case "Drew":
      return `${all[e.seat]} pioche.`;
    case "LineStarted":
      return `La file repart avec ${cardName(e.card)}.`;
    case "WentOut":
      return `${all[e.seat]} n'a plus de cartes : un dernier tour pour les autres.`;
    case "HandOver": {
      const why = e.end === "DeckEmpty" ? "pioche vide" : "plus de cartes";
      const lines = e.gained.map((g, i) => `${all[i]} ${signed(g)} − ${e.penalties[i]} (${num(e.scores[i])})`);
      return `Fin de la main (${why}) : ${lines.join(", ")}.`;
    }
  }
}

export function BreadBasketBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as BreadBasketView;
  const legal = legalActions as BreadBasketAction[];

  const seats = v.seats.map((seat, i) => (
    <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span title="Score">{num(seat.score)}</span>
      </h3>
      <div className="stack">
        {seat.taken.map((card, k) => (
          <Card key={k} rank={card} small />
        ))}
      </div>
      <p className="hint">
        Ramassé : {num(seat.takenPoints)} points{seat.out ? " · n'a plus de cartes" : ""}
      </p>
      <CardBack count={seat.handSize} label="en main" />
    </section>
  ));

  const center = (
    <>
      <p className="pairs-round">
        Main {v.roundNo} sur {v.rounds}
      </p>
      <div className="pairs-line" aria-label="La file">
        {v.line.map((card, k) => (
          <Card key={k} rank={card} />
        ))}
      </div>
      <div className="pairs-piles">
        <CardBack count={v.deckSize} label="pioche" />
        <span className="pairs-pile">
          {v.discard.length > 0 && <Card rank={v.discard[v.discard.length - 1]} small />}
          défausse ({v.discard.length})
        </span>
      </div>
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per value the engine accepts, on its first card.
            const move =
              v.hand.indexOf(card) === k
                ? legal.find((a) => typeof a === "object" && a.Play.card === card)
                : undefined;
            return (
              <Card
                key={k}
                rank={card}
                title={move ? `Poser ${cardName(card)}` : undefined}
                onClick={move ? () => submit(move) : undefined}
              />
            );
          })}
        </div>
      )}
      {legal.includes("Draw") && (
        <div className="pairs-moves">
          <button type="button" onClick={() => submit("Draw")}>
            Piocher
          </button>
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeBreadBasketEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
