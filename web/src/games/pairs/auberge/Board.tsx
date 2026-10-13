import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { AubergeAction, AubergeEvent, AubergeView } from "./types";

export function describeAubergeEvent(e: AubergeEvent, names: string[]): string {
  switch (e.type) {
    case "Placed":
      return `${names[e.seat]} pose un ${e.card}.`;
    case "Starts":
      return `${names[e.seat]} commence.`;
    case "Drew":
      return `${names[e.seat]} pioche.`;
    case "Attacked":
      return `${names[e.seat]} attaque ${names[e.target]} avec un ${e.card}.`;
    case "Dared":
      return `${names[e.seat]} défie ${names[e.target]} : un ${e.card}, ${e.paired ? "qui fait une paire" : "pas de paire"}.`;
    case "Accused": {
      const shown = e.hand.length > 0 ? ` Main montrée : ${e.hand.join(", ")}.` : "";
      return `${names[e.seat]} accuse ${names[e.target]}, ${e.guilty ? "à raison" : "à tort"}.${shown}`;
    }
    case "Out":
      return `${names[e.seat]} est éliminé·e.`;
    case "RoundWon":
      return e.seat === null ? "Manche bloquée : personne ne la gagne." : `${names[e.seat]} gagne la manche.`;
    case "Reshuffled":
      return "La défausse est remélangée.";
  }
}

export function AubergeBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as AubergeView;
  const legal = legalActions as AubergeAction[];
  const places = legal.flatMap((a) => (typeof a === "object" && "Place" in a ? [a.Place.card] : []));
  const attacks = legal.flatMap((a) => (typeof a === "object" && "Attack" in a ? [a.Attack] : []));

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.out ? " lost" : ""}`;
    const dare = legal.find((a) => typeof a === "object" && "Dare" in a && a.Dare.target === i);
    const accuse = legal.find((a) => typeof a === "object" && "Accuse" in a && a.Accuse.target === i);
    const mine = attacks.filter((a) => a.target === i);
    return (
      <section key={i} className={cls}>
        <h3>
          <span>
            {seatNames[i]}
            {seat.out ? " (éliminé)" : ""}
          </span>
          <span title="Manches gagnées">★ {seat.wins}</span>
        </h3>
        <div className="stack">
          {seat.front.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        {!seat.out && <CardBack count={seat.handSize} label="en main" />}
        {i !== me && !seat.out && seat.handSize > 0 && (
          <p className="odds">Paire estimée : {Math.round(seat.pairOdds / 10)} %</p>
        )}
        {(dare || accuse || mine.length > 0) && (
          <div className="pairs-moves">
            {dare && (
              <button type="button" onClick={() => submit(dare)}>
                Défier
              </button>
            )}
            {accuse && (
              <button type="button" onClick={() => submit(accuse)}>
                Accuser
              </button>
            )}
            {mine.map((a) => (
              <Card
                key={a.card}
                rank={a.card}
                small
                title={`Attaquer ${seatNames[i]} avec un ${a.card}`}
                onClick={() => submit({ Attack: a } satisfies AubergeAction)}
              />
            ))}
          </div>
        )}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.round} / {v.rounds}
      </p>
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
        <CardBack count={v.discard} label="Défausse" />
      </div>
      {v.discardSeen.length > 0 && (
        <div className="pairs-seen" title="Cartes défaussées face visible">
          {v.discardSeen.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {legal.includes("Draw") && (
        <button type="button" onClick={() => submit("Draw" satisfies AubergeAction)}>
          Piocher
        </button>
      )}
      {me !== null && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per rank the engine accepts, on its first card.
            const placeable = places.includes(card) && v.hand.indexOf(card) === k;
            return (
              <Card
                key={k}
                rank={card}
                title={placeable ? `Poser ce ${card}` : undefined}
                onClick={placeable ? () => submit({ Place: { card } } satisfies AubergeAction) : undefined}
              />
            );
          })}
        </div>
      )}
      {v.phase === "Setup" && places.length > 0 && <p className="hint">Choisis la carte à poser devant toi.</p>}
      {v.phase === "Setup" && me !== null && places.length === 0 && (
        <p className="hint">En attente des autres joueurs…</p>
      )}
      {v.ownPair !== null && (
        <p className="hint warn">Tu as une paire de {v.ownPair} : une accusation t&apos;éliminerait.</p>
      )}
      <div className="pairs-unlocated" title="Cartes que tu ne peux pas situer">
        {v.unlocated.map((count, i) => (
          <span key={i}>
            <Card rank={i + 1} small />×{count}
          </span>
        ))}
      </div>
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeAubergeEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
