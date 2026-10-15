import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { SweepAction, SweepEvent, SweepView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeSweepEvent(e: SweepEvent, names: string[]): string {
  switch (e.type) {
    case "TargetsDealt":
      return `Manche ${e.round} : cibles ${e.targets.join(", ")}. ${names[e.leader]} tire en premier.`;
    case "Shot":
      return `${names[e.seat]} tire un ${e.card}.`;
    case "Captured":
      return `${names[e.seat]} ramasse ${e.cards.join(", ")}.`;
    case "Discarded":
      return `Restent sur la table, défaussés : ${e.cards.join(", ")}.`;
    case "Scored":
      return `Décompte : ${e.scores.map((points, i) => `${names[i]} ${plural(points, "point")}`).join(", ")}.`;
  }
}

export function SweepBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as SweepView;
  const legal = legalActions as SweepAction[];
  const scored = v.events.find((e) => e.type === "Scored");

  const seats = v.handSizes.map((size, i) => {
    const shot = v.shooters[i];
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
          <div className="pairs-hand">
            <CardBack count={size} label="en main" />
            {shot !== null && (
              <span className="pairs-pile">
                <Card rank={shot} />
                tireur
              </span>
            )}
          </div>
        )}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} / {v.roundsTotal}
      </p>
      <div className="pairs-hand">
        {v.targets.map((card, k) => {
          const first = v.targets.indexOf(card) === k;
          const move = first ? legal.find((a) => "Capture" in a && a.Capture.card === card) : undefined;
          return (
            <Card
              key={k}
              rank={card}
              title={move ? `Ramasser ce ${card}` : undefined}
              onClick={move ? () => submit(move) : undefined}
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
            const move = first ? legal.find((a) => "Shoot" in a && a.Shoot.card === card) : undefined;
            return (
              <Card
                key={k}
                rank={card}
                title={move ? `Tirer ce ${card}` : undefined}
                onClick={move ? () => submit(move) : undefined}
              />
            );
          })}
        </div>
      )}
      {v.phase === "Play" && legal.length > 0 && <p className="hint">Choisis ton tireur.</p>}
      {v.phase === "Capture" && legal.length > 0 && (
        <p className="hint">Tireurs à égalité : choisis la cible que tu ramasses.</p>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeSweepEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
