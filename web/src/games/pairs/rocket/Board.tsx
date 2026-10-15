import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { RocketAction, RocketEvent, RocketStatus, RocketView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeRocketEvent(e: RocketEvent, names: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `Manche ${e.round} : ${e.cards.map((card, i) => `${names[i]} ${card}`).join(", ")} ; la banque montre un ${e.dealer}.`;
    case "Hit":
      return `${names[e.seat]} tire un ${e.card}.`;
    case "Busted":
      return `${names[e.seat]} tire un ${e.card} : paire, paie ${plural(e.paid, "jeton")} à la banque.`;
    case "Folded":
      return `${names[e.seat]} se couche et paie ${plural(e.paid, "jeton")}.`;
    case "DealerHit":
      return `La banque tire un ${e.card}.`;
    case "DealerLost": {
      const why = e.card === 3 ? "un 3, carte fatale" : `une paire de ${e.card}`;
      const who = e.seats.length > 0 ? `paie ${plural(e.amount, "jeton")} à ${e.seats.map((s) => names[s]).join(", ")}` : "ne paie personne";
      return `La banque tire ${why} : elle ${who}.`;
    }
    case "Reshuffled":
      return "Le paquet est remélangé.";
  }
}

const STATUS: Record<RocketStatus, string> = { In: "", Folded: "Couché", Busted: "Paire !" };

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

export function RocketBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as RocketView;
  const legal = legalActions as RocketAction[];

  const seats = v.stacks.map((stack, i) => {
    const out = v.status[i] !== "In";
    return (
      <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}${out ? " lost" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Jetons contre la banque">{signed(v.chips[i])}</span>
        </h3>
        <div className="stack">
          {stack.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        {STATUS[v.status[i]] && <p className="hint">{STATUS[v.status[i]]}</p>}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} / {v.roundsTotal}
      </p>
      <div className="pairs-hand">
        <CardBack count={v.deck} label="Pioche" />
        <span>Banque :</span>
        {v.dealer.map((card, k) => (
          <Card key={k} rank={card} />
        ))}
      </div>
      {legal.length > 0 && (
        <div className="pairs-moves">
          {legal.includes("Hit") && (
            <button type="button" onClick={() => submit("Hit" satisfies RocketAction)}>
              Tirer
            </button>
          )}
          <button type="button" onClick={() => submit("Fold" satisfies RocketAction)}>
            Se coucher ({plural(v.foldCost ?? 0, "jeton")})
          </button>
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeRocketEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
