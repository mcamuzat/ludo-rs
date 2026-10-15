import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { KittyAction, KittyEvent, KittyView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeKittyEvent(e: KittyEvent, names: string[]): string {
  switch (e.type) {
    case "TargetRevealed":
      return `Manche ${e.round} : ${e.cards.length > 1 ? "cibles" : "cible"} ${e.cards.join(" et ")}.`;
    case "BidsRevealed": {
      const bids = e.bids.map((card, i) => `${names[i]} ${card}`).join(", ");
      const winners =
        e.winners.length === 0 ? "aucune mise unique" : `gagnant : ${e.winners.map((w) => names[w]).join(", puis ")}`;
      return `Mises : ${bids} ; ${winners}.`;
    }
    case "Took":
      return `${names[e.seat]} prend le ${e.card}.`;
    case "Gave":
      return `${names[e.seat]} a gagné avec le 1 et donne le ${e.card} à ${names[e.to]}.`;
    case "Discarded":
      return `Personne ne prend ${e.cards.length > 1 ? "les" : "le"} ${e.cards.join(" et ")} : défaussé.`;
    case "Scored": {
      const scores = e.scores.map((points, i) => `${names[i]} ${plural(points, "point")}`).join(", ");
      return `Décompte : ${scores}.`;
    }
  }
}

export function KittyBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as KittyView;
  const legal = legalActions as KittyAction[];
  const scored = v.events.find((e) => e.type === "Scored");

  const seats = v.handSizes.map((size, i) => {
    const gives = legal.filter((a): a is { Give: { card: number; to: number } } => "Give" in a && a.Give.to === i);
    return (
      <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          {v.scores.length > 0 && <span>{plural(v.scores[i], "point")}</span>}
        </h3>
        {scored ? (
          <div className="stack">
            {scored.hands[i].map((card, k) => (
              <Card key={k} rank={card} small />
            ))}
          </div>
        ) : (
          <CardBack count={size} label="en main" />
        )}
        {gives.length > 0 && (
          <div className="pairs-moves">
            {gives.map((a, k) => (
              <button key={k} type="button" onClick={() => submit(a)}>
                Donner le {a.Give.card}
              </button>
            ))}
          </div>
        )}
      </section>
    );
  });

  const verb = v.phase === "Pass" ? "Passer à gauche" : "Miser";
  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} / {v.roundsTotal}
        {v.soft ? " · Kitty douce" : ""}
      </p>
      <div className="pairs-hand">
        <CardBack count={v.kittySize} label="Kitty" />
        {v.target.map((card, k) => {
          const take = legal.find((a) => "Take" in a && a.Take.card === card);
          const first = v.target.indexOf(card) === k;
          return (
            <Card
              key={k}
              rank={card}
              title={take && first ? `Prendre le ${card}` : undefined}
              onClick={take && first ? () => submit(take) : undefined}
            />
          );
        })}
      </div>
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            const first = v.hand.indexOf(card) === k;
            const move = first
              ? legal.find((a) => ("Pass" in a && a.Pass.card === card) || ("Bid" in a && a.Bid.card === card))
              : undefined;
            return (
              <Card
                key={k}
                rank={card}
                title={move ? `${verb} ce ${card}` : undefined}
                onClick={move ? () => submit(move) : undefined}
              />
            );
          })}
        </div>
      )}
      {me !== null && (v.passed !== null || v.received !== null) && (
        <p className="hint">
          {v.passed !== null ? `Tu as passé un ${v.passed}` : ""}
          {v.passed !== null && v.received !== null ? ", " : ""}
          {v.received !== null ? `tu as reçu un ${v.received}` : ""}.
        </p>
      )}
      {(v.phase === "Pass" || v.phase === "Bid") && legal.length > 0 && (
        <p className="hint">
          {v.phase === "Pass" ? "Choisis la carte à passer à gauche." : "Choisis ta mise : la plus basse mise unique gagne."}
        </p>
      )}
      {(v.phase === "Pass" || v.phase === "Bid") && me !== null && legal.length === 0 && (
        <p className="hint">
          {v.placed !== null ? `Ton choix : ${v.placed}. ` : ""}En attente des autres joueurs…
        </p>
      )}
      {legal.some((a) => "Give" in a) && <p className="hint">Tu as gagné avec le 1 : donne une carte à un autre joueur.</p>}
      {legal.some((a) => "Take" in a) && <p className="hint">Choisis la carte que tu prends.</p>}
      {v.discard.length > 0 && (
        <div className="pairs-seen">
          {v.discard.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeKittyEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
