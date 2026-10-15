import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { PortAction, PortEvent, PortStatus, PortView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describePortEvent(e: PortEvent, names: string[]): string {
  switch (e.type) {
    case "RoundStarted": {
      const who = e.seats.length < names.length ? ` entre ${e.seats.map((s) => names[s]).join(" et ")}` : "";
      return `Manche ${e.round}${who}. ${names[e.first]} commence.`;
    }
    case "Dealt":
      return `${names[e.seat]} reçoit un ${e.card}.`;
    case "Replaced":
      return `Départage : le ${e.card} de ${names[e.seat]} ferait une paire, il est remplacé.`;
    case "Busted":
      return `${names[e.seat]} tire un ${e.card} : paire, éliminé de la manche.`;
    case "Locked":
      return `${names[e.seat]} s'arrête.`;
    case "Reshuffled":
      return "La défausse est remélangée, 5 cartes sont brûlées.";
    case "RoundScored": {
      const parts = e.points.flatMap((points, i) =>
        e.totals[i] === null && points === 0 ? [] : [`${names[i]} ${e.totals[i]} → ${plural(points, "point")}`],
      );
      return `Fin de manche : ${parts.length > 0 ? parts.join(", ") : "tout le monde a sauté"}.`;
    }
  }
}

const STATUS: Record<PortStatus, string> = {
  Playing: "",
  Locked: "Arrêté",
  Busted: "Paire !",
  Out: "Hors manche",
};

export function PortBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as PortView;
  const legal = legalActions as PortAction[];

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.status === "Busted" ? " lost" : ""}`;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Points de partie">
            {seat.score} / {v.target}
          </span>
        </h3>
        <div className="stack">
          {seat.stack.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        <p className="hint">
          {seat.stack.length > 0 ? `Total ${seat.total}` : ""}
          {STATUS[seat.status] ? ` · ${STATUS[seat.status]}` : ""}
        </p>
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">Manche {v.roundNo}</p>
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
        <CardBack count={v.discard} label="Défausse" />
      </div>
      {legal.length > 0 && (
        <div className="pairs-moves">
          {legal.includes("Hit") && (
            <button type="button" onClick={() => submit("Hit" satisfies PortAction)}>
              Prendre une carte
            </button>
          )}
          <button type="button" onClick={() => submit("Lock" satisfies PortAction)}>
            S'arrêter
          </button>
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describePortEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
