import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { PickupAction, PickupEvent, PickupView } from "./types";

export function describePickupEvent(e: PickupEvent, all: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `Donne ${e.deal} : ${all[e.dealer]} distribue.`;
    case "PoolDealt":
      return `Nouvelle réserve : ${e.cards.join(", ")}.`;
    case "Revealed":
      return `On révèle : ${e.cards
        .flatMap((c, i) => (c === null ? [] : [`${all[i]} ${c}`]))
        .join(", ")}.`;
    case "Took": {
      const gone = [
        ...e.knocked.map(([seat, n]) => `${n} chez ${all[seat]}`),
        ...(e.pool_knocked > 0 ? [`${e.pool_knocked} dans la réserve`] : []),
      ];
      const tail = gone.length > 0 ? ` ; éliminés : ${gone.join(", ")}` : "";
      return `${all[e.seat]} joue un ${e.played} et prend un ${e.card}${tail}.`;
    }
    case "Stacked":
      return `${all[e.seat]} n'a rien à prendre et garde son ${e.card}.`;
    case "DealOver":
      return `Fin de la donne : ${e.points.map((p, i) => `${all[i]} +${p} (${e.scores[i]})`).join(", ")}.`;
  }
}

export function PickupBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as PickupView;
  const legal = legalActions as PickupAction[];
  const takes = legal.filter((a): a is { Take: { card: number } } => "Take" in a);

  const seats = v.seats.map((seat, i) => (
    <section key={i} className={`pairs-seat${v.taking?.[0] === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span title="Score">{seat.score}</span>
      </h3>
      <div className="stack">
        {seat.pile.map((card, k) => (
          <Card key={k} rank={card} small />
        ))}
      </div>
      <p className="hint">
        Pile : {seat.pilePoints} points{seat.waiting !== null ? ` · attend : ${seat.waiting}` : ""}
      </p>
      <CardBack count={seat.handSize} label="en main" />
    </section>
  ));

  const center = (
    <>
      <p className="pairs-round">
        Donne {v.dealNo}
        {v.target !== null ? ` · Partie en ${v.target} points` : ""}
      </p>
      <div className="pairs-line" aria-label="La réserve">
        {v.pool.map((card, k) => {
          // One button per rank the engine accepts, on its first card.
          const move = v.pool.indexOf(card) === k ? takes.find((a) => a.Take.card === card) : undefined;
          return (
            <Card
              key={k}
              rank={card}
              title={move ? `Prendre ce ${card}` : undefined}
              onClick={move ? () => submit(move) : undefined}
            />
          );
        })}
      </div>
      <CardBack count={v.deckSize} label="pioche" />
      {takes.length > 0 && v.taking !== null && (
        <p className="hint">Choisis la carte à prendre avec ton {v.taking[1]}.</p>
      )}
      {v.chosen !== null && <p className="hint">Ta carte : {v.chosen} (face cachée)</p>}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            const move =
              v.hand.indexOf(card) === k
                ? legal.find((a) => "Choose" in a && a.Choose.card === card)
                : undefined;
            return (
              <Card
                key={k}
                rank={card}
                raised={v.chosen === card && v.hand.indexOf(card) === k}
                title={move ? `Jouer ce ${card} face cachée` : undefined}
                onClick={move ? () => submit(move) : undefined}
              />
            );
          })}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describePickupEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
