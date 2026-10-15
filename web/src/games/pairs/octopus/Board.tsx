import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { OctopusAction, OctopusEvent, OctopusKind, OctopusView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

const KIND_NAMES: Record<OctopusKind, string> = {
  Set: "mains de séries",
  Clean: "mains propres",
  Dirty: "mains sales",
};

function names(seats: number[], all: string[]): string {
  return seats.map((s) => all[s]).join(" et ");
}

export function describeOctopusEvent(e: OctopusEvent, all: string[]): string {
  switch (e.type) {
    case "HandStarted": {
      const antes = e.antes.map((ante, i) => `${all[i]} ${ante}`).join(", ");
      return `Main ${e.hand}. Mises : ${antes}. ${all[e.leader]} parle en premier.`;
    }
    case "Declared":
      return e.fold ? `${all[e.seat]} se couche (1 jeton).` : `${all[e.seat]} reste.`;
    case "FoldedOut":
      return `Tous les autres se sont couchés : ${all[e.seat]} prend le pot (${plural(e.pot, "jeton")}).`;
    case "CardsPlayed": {
      const cards = e.cards.flatMap((card, i) => (card === null ? [] : [`${all[i]} ${card}`])).join(", ");
      const caller = e.caller === null ? "pas de carte unique" : `${all[e.caller]} a la plus basse carte unique`;
      return `Cartes jouées : ${cards} ; ${caller}.`;
    }
    case "PassCalled":
      return `${all[e.seat]} fait passer une carte.`;
    case "Passed":
      return "Chacun passe une carte à gauche.";
    case "Showdown": {
      const hands = e.hands.flatMap((hand, i) => (hand === null ? [] : [`${all[i]} ${hand.join("-")}`])).join(", ");
      const rule =
        e.best === null
          ? "Une carte chacun, la plus basse gagne"
          : `${e.caller !== null ? all[e.caller] : "?"} choisit les ${KIND_NAMES[e.best]}`;
      const losers = e.losers.length > 0 ? ` ${names(e.losers, all)} paie 5 jetons.` : "";
      return (
        `Abattage. ${rule}. Mains : ${hands}. ` +
        (e.winners.length > 1
          ? `${names(e.winners, all)} partagent le pot (${plural(e.share, "jeton")} chacun).`
          : `${names(e.winners, all)} prend le pot (${plural(e.share, "jeton")}).`) +
        losers
      );
    }
  }
}

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

export function OctopusBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as OctopusView;
  const legal = legalActions as OctopusAction[];
  const shown = v.events.find((e) => e.type === "Showdown");

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.folded ? " lost" : ""}`;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Jetons depuis le début de la partie">{signed(seat.chips)}</span>
        </h3>
        {seat.folded ? <p className="hint">Couché</p> : <CardBack count={seat.handSize} label="en main" />}
      </section>
    );
  });

  const kindButtons = legal.flatMap((a) => (typeof a === "object" && "Show" in a ? [a] : []));
  const verb = v.phase === "Pass" ? "Passer à gauche" : "Jouer";
  const center = (
    <>
      <p className="pairs-round">
        Main {v.handNo} / {v.handsTotal} · Pot : {v.pot}
      </p>
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            const first = v.hand.indexOf(card) === k;
            const move = first
              ? legal.find(
                  (a) =>
                    typeof a === "object" &&
                    (("Play" in a && a.Play.card === card) || ("Pass" in a && a.Pass.card === card)),
                )
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
      {legal.includes("Stay") && (
        <div className="pairs-moves">
          <button type="button" onClick={() => submit("Stay" satisfies OctopusAction)}>
            Rester
          </button>
          <button type="button" onClick={() => submit("Fold" satisfies OctopusAction)}>
            Se coucher (1 jeton)
          </button>
        </div>
      )}
      {legal.includes("CallPass") && (
        <div className="pairs-moves">
          <span>Tu as la plus basse carte unique :</span>
          <button type="button" onClick={() => submit("CallPass" satisfies OctopusAction)}>
            Faire passer
          </button>
          {kindButtons.map((a) => (
            <button key={a.Show.best} type="button" onClick={() => submit(a)}>
              Abattage, {KIND_NAMES[a.Show.best]} gagnantes
            </button>
          ))}
        </div>
      )}
      {(v.phase === "Play" || v.phase === "Pass") && legal.length > 0 && (
        <p className="hint">
          {v.phase === "Play"
            ? "Choisis la carte à jouer : la plus basse carte unique décide."
            : "Choisis la carte à passer à gauche."}
        </p>
      )}
      {(v.phase === "Play" || v.phase === "Pass") && me !== null && legal.length === 0 && !v.seats[me].folded && (
        <p className="hint">
          {v.placed !== null ? `Ton choix : ${v.placed}. ` : ""}En attente des autres joueurs…
        </p>
      )}
      {me !== null && v.passed !== null && v.received !== null && (
        <p className="hint">
          Tu as passé un {v.passed}, tu as reçu un {v.received}.
        </p>
      )}
      {shown && (
        <div className="pairs-seen">
          {shown.hands.map((hand, i) =>
            hand === null ? null : (
              <span key={i} className="pairs-pile">
                <span>
                  {hand.map((card, k) => (
                    <Card key={k} rank={card} small />
                  ))}
                </span>
                {seatNames[i]}
              </span>
            ),
          )}
        </div>
      )}
      {v.played.length > 0 && !shown && (
        <div className="pairs-seen">
          {v.played.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeOctopusEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
