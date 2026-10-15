import type { BoardProps } from "../../registry";
import { Card, Table } from "../cards";
import type { StarboardAction, StarboardEvent, StarboardStatus, StarboardView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeStarboardEvent(e: StarboardEvent, names: string[]): string {
  switch (e.type) {
    case "RoundStarted":
      return `Manche ${e.round} : ${e.center.join(" et ")} au milieu. ${names[e.button]} a le bouton.`;
    case "Hit": {
      const knocked = e.knocked.length > 0 ? ` ${e.knocked.map((s) => names[s]).join(" et ")} ${e.knocked.length > 1 ? "sont éliminés" : "est éliminé"}.` : "";
      return `${names[e.seat]} paie 1 jeton et tire un ${e.card} : pas de paire.${knocked}`;
    }
    case "Busted":
      return `${names[e.seat]} paie 1 jeton et tire un ${e.card} : paire, éliminé.`;
    case "Locked":
      return e.auto ? `${names[e.seat]}, seul en jeu, se bloque d'office.` : `${names[e.seat]} se bloque.`;
    case "RoundEnded": {
      const left = e.left > 0 ? ` ${plural(e.left, "jeton")} reste${e.left > 1 ? "nt" : ""} au pot.` : "";
      if (e.winners.length === 0) return `Personne ne survit : le pot reste.${left}`;
      const who = e.winners.map((w) => names[w]).join(", ");
      return `${who} ${e.winners.length > 1 ? "se partagent" : "prend"} le pot : ${plural(e.share, "jeton")}${e.winners.length > 1 ? " chacun" : ""}.${left}`;
    }
  }
}

const STATUS: Record<StarboardStatus, string> = {
  In: "En jeu",
  Locked: "Bloqué",
  Busted: "Paire !",
  Knocked: "Éliminé",
};

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

export function StarboardBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as StarboardView;
  const legal = legalActions as StarboardAction[];

  const seats = v.seats.map((seat, i) => {
    const out = seat.status === "Busted" || seat.status === "Knocked";
    return (
      <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}${out ? " lost" : ""}`}>
        <h3>
          <span>
            {seatNames[i]}
            {v.button === i ? " ●" : ""}
          </span>
          <span title="Jetons depuis le début de la partie">{signed(seat.chips)}</span>
        </h3>
        <p className="hint">{STATUS[seat.status]}</p>
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} / {v.roundsTotal} · Pot : {v.pot}
      </p>
      <div className="pairs-hand">
        {v.center.map((card, k) => (
          <Card key={k} rank={card} />
        ))}
      </div>
      {legal.length > 0 && (
        <div className="pairs-moves">
          {legal.includes("Hit") && (
            <button type="button" onClick={() => submit("Hit" satisfies StarboardAction)}>
              Tirer (1 jeton)
            </button>
          )}
          <button type="button" onClick={() => submit("Lock" satisfies StarboardAction)}>
            Se bloquer
          </button>
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeStarboardEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
