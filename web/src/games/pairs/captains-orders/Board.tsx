import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type {
  CaptainsOrdersAction,
  CaptainsOrdersEvent,
  CaptainsOrdersRoundResult,
  CaptainsOrdersView,
  Rank,
} from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

function aCard(card: Rank | null): string {
  return card === null ? "une carte" : `un ${card}`;
}

function list(seats: number[], names: string[]): string {
  return seats.map((s) => names[s]).join(", ");
}

export function describeCaptainsOrdersEvent(e: CaptainsOrdersEvent, names: string[]): string {
  switch (e.type) {
    case "Ordered":
      return e.belay
        ? `${names[e.seat]} annule l'ordre avec un ${e.card}.`
        : `${names[e.seat]} donne l'ordre ${e.card}.`;
    case "Followed":
      return `${names[e.seat]} obéit et passe ${aCard(e.card)} à ${names[e.owner]}.`;
    case "OrderDone":
      return `L'ordre ${e.card} de ${names[e.owner]} a fait le tour : défaussé.`;
    case "Passed":
      return `${names[e.from]} passe ${aCard(e.card)} à ${names[e.to]}.`;
    case "Asked":
      return `${names[e.from]} demande une carte à ${names[e.to]}.`;
    case "Gave":
      return `${names[e.from]} donne ${aCard(e.card)} à ${names[e.to]}.`;
    case "Discarded":
      return `${names[e.seat]} défausse ${aCard(e.card)} face cachée.`;
  }
}

export function describeRound(r: CaptainsOrdersRoundResult, names: string[]): string {
  const e = r.ending;
  const how =
    e.type === "Accused"
      ? `${names[e.accuser]} accuse ${names[e.target]} : ${e.right ? "démasqué" : "raté"}.`
      : `${names[e.seat]} vide sa main avec un ${e.card} et se déclare Capitaine : ${e.right ? "à raison" : "à tort"}.`;
  const hands = r.hands
    .map((hand, i) => `${names[i]} ${hand.length > 0 ? hand.join("-") : "rien"} (${r.points[i]})`)
    .join(", ");
  const captains = `${r.captains.length > 1 ? "Capitaines" : "Capitaine"} : ${list(r.captains, names)}.`;
  const winners =
    r.winners.length === 0
      ? "Personne ne gagne la manche."
      : `${list(r.winners, names)} ${r.winners.length > 1 ? "gagnent" : "gagne"} la manche.`;
  return `Manche ${r.round} — ${how} Mains : ${hands}. ${captains} ${winners}`;
}

export function CaptainsOrdersBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as CaptainsOrdersView;
  const legal = legalActions as CaptainsOrdersAction[];

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}`;
    const ask = legal.find((a) => "Ask" in a && a.Ask.from === i);
    const accuse = legal.find((a) => "Accuse" in a && a.Accuse.seat === i);
    const passes = legal.filter((a): a is { Pass: { card: Rank; to: number } } => "Pass" in a && a.Pass.to === i);
    const owner = v.order !== null && v.order.owner === i;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>
            {seatNames[i]}
            {v.dealer === i ? " (donne)" : ""}
          </span>
          <span title="Manches gagnées">{plural(seat.wins, "manche")}</span>
        </h3>
        {owner && v.order !== null && (
          <div className="stack" title="Son ordre">
            <Card rank={v.order.card} />
          </div>
        )}
        <CardBack count={seat.handSize} label="en main" />
        {(ask || accuse || passes.length > 0) && (
          <div className="pairs-moves">
            {passes.length > 0 && <span>Passer :</span>}
            {passes.map((a) => (
              <Card
                key={a.Pass.card}
                rank={a.Pass.card}
                small
                title={`Passer ce ${a.Pass.card} à ${seatNames[i]}`}
                onClick={() => submit(a)}
              />
            ))}
            {ask && (
              <button type="button" onClick={() => submit(ask)}>
                Demander une carte
              </button>
            )}
            {accuse && (
              <button type="button" onClick={() => submit(accuse)}>
                Accuser : Capitaine !
              </button>
            )}
          </div>
        )}
      </section>
    );
  });

  const handMove = (card: Rank): [CaptainsOrdersAction, string] | null => {
    for (const a of legal) {
      if ("Order" in a && a.Order.card === card)
        return [a, v.order === null ? `Ordonner ce ${card}` : `Annuler l'ordre avec ce ${card}`];
      if ("Follow" in a && a.Follow.card === card) return [a, `Obéir avec ce ${card}`];
      if ("Give" in a && a.Give.card === card) return [a, `Donner ce ${card}`];
      if ("Discard" in a && a.Discard.card === card) return [a, `Défausser ce ${card}`];
    }
    return null;
  };

  const myTurn = legal.length > 0;
  let hint: string | null = null;
  if (myTurn) {
    switch (v.phase.type) {
      case "Turn":
        hint = "Joue une carte comme ordre, passe une carte, demande-en une, ou accuse un joueur.";
        break;
      case "Respond":
        hint = "Obéis (une carte au moins égale à l'ordre), annule-le avec une carte plus basse, ou accuse.";
        break;
      case "Give":
        hint = `${seatNames[v.phase.asker]} te demande une carte : choisis laquelle donner.`;
        break;
      case "Discard":
        hint = "Défausse une carte face cachée.";
        break;
    }
  }

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.round} · {plural(v.winsNeeded, "manche")} gagnée{v.winsNeeded > 1 ? "s" : ""} pour l'emporter
      </p>
      {v.order !== null && (
        <p>
          Ordre en cours : {v.order.card} ({seatNames[v.order.owner]})
        </p>
      )}
      {v.phase.type === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per rank the engine accepts, on its first card.
            const move = v.hand.indexOf(card) === k ? handMove(card) : null;
            return (
              <Card
                key={k}
                rank={card}
                title={move?.[1]}
                onClick={move ? () => submit(move[0]) : undefined}
              />
            );
          })}
        </div>
      )}
      {v.discarded.length > 0 && <p className="hint">Tes défausses : {v.discarded.join(", ")}</p>}
      {hint && <p className="hint">{hint}</p>}
      {v.lastRound !== null && <p className="hint">{describeRound(v.lastRound, seatNames)}</p>}
      <ul className="pairs-events">
        {v.log.map((e, i) => (
          <li key={i}>{describeCaptainsOrdersEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
