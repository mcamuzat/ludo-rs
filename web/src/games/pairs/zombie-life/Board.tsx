import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { ZombieLifeAction, ZombieLifeEvent, ZombieLifeView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeZombieLifeEvent(e: ZombieLifeEvent, names: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `${names[e.seat]} reçoit un ${e.card}.`;
    case "Starts":
      return `${names[e.seat]} commence.`;
    case "Drew":
      return `${names[e.seat]} pioche un ${e.card}.`;
    case "Placed":
      return e.zone === "Refuge"
        ? `${names[e.seat]} pose le ${e.card} dans son Refuge.`
        : `${names[e.seat]} pose le ${e.card} dans la Horde.`;
    case "Took":
      return `${names[e.seat]} prend le ${e.card} de la Horde.`;
    case "Passed":
      return e.hidden === 0
        ? `${names[e.seat]} se cache.`
        : `${names[e.seat]} se cache et retourne ${plural(e.hidden, "carte")}.`;
    case "Overrun":
      return `${names[e.seat]} pioche un ${e.card} : submergé par les zombies !`;
    case "Scored":
      return e.points === 0
        ? `${names[e.seat]} ne marque rien.`
        : `${names[e.seat]} marque ${plural(e.points, "point")}.`;
    case "Dealer":
      return `${names[e.seat]} distribue la nouvelle manche.`;
    case "Reshuffled":
      return "La défausse est remélangée.";
  }
}

const STATUS_LABEL = { In: null, Passed: "caché", Out: "submergé" } as const;

export function ZombieLifeBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as ZombieLifeView;
  const legal = legalActions as ZombieLifeAction[];
  const find = (match: (a: Exclude<ZombieLifeAction, string>) => boolean) =>
    legal.find((a) => typeof a === "object" && match(a));

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.status === "Out" ? " lost" : ""}`;
    const label = STATUS_LABEL[seat.status];
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Points">
            {seat.score} / {v.target}
          </span>
        </h3>
        <div className="stack">
          {seat.faceUp.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
          {seat.faceDown > 0 && <CardBack count={seat.faceDown} label="cachées" />}
        </div>
        {label && <p className="odds">{label}</p>}
      </section>
    );
  });

  const turnName = v.turn === null ? "" : seatNames[v.turn];
  const toRefuge = find((a) => "Place" in a && a.Place.zone === "Refuge");
  const toHorde = find((a) => "Place" in a && a.Place.zone === "Horde");

  const center = (
    <>
      <p className="pairs-round">Manche {v.round}</p>
      <div className="pairs-street" title="La Horde : les cartes que chacun peut prendre">
        <span>Horde :</span>
        {v.horde.map((card, k) => {
          const move = find((a) => "Take" in a && a.Take.card === card);
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
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
        <CardBack count={v.discard} label="Défausse" />
      </div>
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {v.phase === "Placing" && v.drawn !== null && (
        <div className="pairs-hand">
          <span>{turnName} a pioché :</span>
          <Card rank={v.drawn} />
        </div>
      )}
      {v.phase === "Dealing" && (
        <>
          <div className="pairs-hand">
            <span>Cartes de {turnName} :</span>
            {v.dealing.map((card, k) => {
              // One button per rank the engine accepts, on its first card.
              const move =
                v.dealing.indexOf(card) === k ? find((a) => "Give" in a && a.Give.card === card) : undefined;
              const to = v.dealTo === null ? "" : seatNames[v.dealTo];
              return (
                <Card
                  key={k}
                  rank={card}
                  title={move ? `Donner ce ${card} à ${to}` : undefined}
                  onClick={move ? () => submit(move) : undefined}
                />
              );
            })}
          </div>
          {v.dealTo !== null && (
            <p className="hint">
              {legal.length > 0
                ? `Choisis la carte de ${seatNames[v.dealTo]}.`
                : `${turnName} distribue : la prochaine carte est pour ${seatNames[v.dealTo]}.`}
            </p>
          )}
        </>
      )}
      <div className="pairs-moves">
        {legal.includes("Draw") && (
          <button type="button" onClick={() => submit("Draw" satisfies ZombieLifeAction)}>
            Piocher
          </button>
        )}
        {toRefuge && (
          <button type="button" onClick={() => submit(toRefuge)}>
            Dans mon Refuge
          </button>
        )}
        {toHorde && (
          <button type="button" onClick={() => submit(toHorde)}>
            Dans la Horde
          </button>
        )}
        {legal.includes("Pass") && (
          <button type="button" onClick={() => submit("Pass" satisfies ZombieLifeAction)}>
            Passer
          </button>
        )}
      </div>
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeZombieLifeEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
