import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { ContinuousAction, ContinuousEvent, ContinuousView, Rank } from "./types";

export function canHit(legal: ContinuousAction[]): boolean {
  return legal.includes("Hit");
}

/** The cards the player may take by folding, as the engine lists them. */
export function foldTargets(legal: ContinuousAction[]): { seat: number; card: Rank }[] {
  return legal.flatMap((a) => (typeof a === "object" ? [a.Fold] : []));
}

/** « d'Ana », « de Bob ». */
function of(name: string): string {
  return /^[aeiouyhàâéèêëîïôùûœ]/i.test(name) ? `d'${name}` : `de ${name}`;
}

export function describeEvent(e: ContinuousEvent, names: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `${names[e.seat]} reçoit un ${e.card}.`;
    case "Replaced":
      return `Départage : le ${e.card} ${of(names[e.seat])} ferait une paire, il est remplacé.`;
    case "Paired":
      return `${names[e.seat]} fait une paire de ${e.card} : ${e.card} points.`;
    case "Folded":
      return e.seat === e.from
        ? `${names[e.seat]} se couche et garde son ${e.card} : ${e.card} points.`
        : `${names[e.seat]} se couche et prend le ${e.card} ${of(names[e.from])} : ${e.card} points.`;
    case "Reshuffled":
      return "La défausse est remélangée, 5 cartes sont brûlées.";
  }
}

export function ContinuousBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as ContinuousView;
  const legal = legalActions as ContinuousAction[];
  const folds = foldTargets(legal);

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${v.loser === i ? " lost" : ""}`;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span>
            {seat.score} / {v.target}
          </span>
        </h3>
        <div className="stack">
          {seat.stack.map((card, k) => {
            const takeable = folds.some((f) => f.seat === i && f.card === card);
            return (
              <Card
                key={k}
                rank={card}
                title={takeable ? `Se coucher en prenant ce ${card} (${card} points)` : undefined}
                onClick={
                  takeable ? () => submit({ Fold: { seat: i, card } } satisfies ContinuousAction) : undefined
                }
              />
            );
          })}
        </div>
        <div className="scored">
          {seat.scored.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      </section>
    );
  });

  const center = (
    <>
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
        <CardBack count={v.discard} label="Défausse" />
      </div>
      {v.loser !== null && <p className="hint">{seatNames[v.loser]} a perdu.</p>}
      {canHit(legal) && (
        <button type="button" onClick={() => submit("Hit" satisfies ContinuousAction)}>
          Prendre une carte
        </button>
      )}
      {folds.length > 0 && v.lowest !== null && (
        <p className="hint">Se coucher coûte au moins {v.lowest} : clique sur la carte à prendre.</p>
      )}
      <div className="pairs-unlocated" title="Cartes que personne ne peut situer : pioche et cartes brûlées">
        {v.unlocated.map((count, i) => (
          <span key={i}>
            <Card rank={i + 1} small />×{count}
          </span>
        ))}
      </div>
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
