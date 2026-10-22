import type { BoardProps } from "../../registry";
import { Card, Table } from "../cards";
import type { MarketDayAction, MarketDayEvent, MarketDayView } from "./types";

/** The special cards, by rank. */
const SPECIAL: Record<number, string> = { 1: "Bricoleur", 2: "Chevalier", 3: "Royal", 7: "Pluie", 9: "Bandit" };

/** "d'Ana", "de Hugo". */
function of(name: string): string {
  return /^[aeiouyàâéèêëîïôûü]/i.test(name) ? `d'${name}` : `de ${name}`;
}

export function describeMarketDayEvent(e: MarketDayEvent, all: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `Main ${e.hand} : ${all[e.dealer]} distribue.`;
    case "Played":
      return `${all[e.seat]} pose un ${e.card}${SPECIAL[e.card] ? ` (${SPECIAL[e.card]})` : ""}.`;
    case "Swapped":
      return `Le Bricoleur ${of(all[e.seat])} prend le ${e.card} ${of(all[e.with])}.`;
    case "TrickScored": {
      const gains = e.gained.map((g, i) => `${all[i]} +${g} (${e.scores[i]})`).join(", ");
      const fate = (f: string, label: string) => {
        const cards = e.cards.filter(([, , x]) => x === f).map(([, c]) => c);
        return cards.length > 0 ? ` ${label} : ${cards.join(", ")}.` : "";
      };
      return `Fin du tour : ${gains}.${fate("Killed", "Tués")}${fate("Spoiled", "Gâchés")}`;
    }
    case "HandOver":
      return "Fin de la main.";
  }
}

export function MarketDayBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as MarketDayView;
  const legal = legalActions as MarketDayAction[];

  const seats = v.seats.map((seat, i) => (
    <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span title="Score">{seat.score}</span>
      </h3>
      <p className="hint">
        {seat.handSize} en main{v.leader === i ? " · mène" : ""}
      </p>
    </section>
  ));

  const center = (
    <>
      <p className="pairs-round">
        Main {v.handNo} sur {v.hands}
      </p>
      <p className="hint">Le marché</p>
      <div className="pairs-line" aria-label="Le marché">
        {v.trick.map(([seat, card], k) => {
          const move = legal.find((a) => typeof a === "object" && "Swap" in a && a.Swap.with === k);
          return (
            <span key={k} className="md-stall">
              <Card
                rank={card}
                title={move ? `Échanger ton Bricoleur contre ce ${card}` : SPECIAL[card]}
                onClick={move ? () => submit(move) : undefined}
              />
              <small>{seatNames[seat]}</small>
            </span>
          );
        })}
      </div>
      {legal.includes("Keep") && (
        <div className="pairs-moves">
          <button type="button" onClick={() => submit("Keep")}>
            Garder mon Bricoleur
          </button>
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            const move =
              v.hand.indexOf(card) === k
                ? legal.find((a) => typeof a === "object" && "Play" in a && a.Play.card === card)
                : undefined;
            return (
              <Card
                key={k}
                rank={card}
                title={move ? `Poser ce ${card}${SPECIAL[card] ? ` (${SPECIAL[card]})` : ""}` : SPECIAL[card]}
                onClick={move ? () => submit(move) : undefined}
              />
            );
          })}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeMarketDayEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
