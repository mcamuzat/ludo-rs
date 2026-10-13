import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { GoblinPokerAction, GoblinPokerEvent, GoblinPokerView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

/** "Ana", "Ana et Bob", "Ana, Bob et Cléo". */
function list(names: string[]): string {
  if (names.length <= 1) return names.join("");
  return `${names.slice(0, -1).join(", ")} et ${names[names.length - 1]}`;
}

export function describeGoblinPokerEvent(e: GoblinPokerEvent, names: string[]): string {
  switch (e.type) {
    case "Placed":
      return `${names[e.seat]} commence sa pile avec un ${e.card}.`;
    case "TieBreak":
      return e.kept
        ? `Départage : ${names[e.seat]} ajoute un ${e.card} à sa pile.`
        : `Départage : ${names[e.seat]} tire un ${e.card}, une paire : défaussé.`;
    case "Starts":
      return `${names[e.seat]} a la pile la plus basse et commence.`;
    case "Pitched":
      return `${names[e.seat]} jette une carte.`;
    case "Folded":
      return `${names[e.seat]} se couche et paie ${plural(e.cost, "jeton")}.`;
    case "Caught":
      return e.paired
        ? `${names[e.seat]} reçoit un ${e.card} : paire ! ${plural(e.card, "jeton")} au pot, et dehors.`
        : `${names[e.seat]} reçoit un ${e.card}.`;
    case "HandEnded":
      return e.winners.length === 1
        ? `${names[e.winners[0]]} reste seul et prend le pot : ${plural(e.share, "jeton")}.`
        : `${list(e.winners.map((w) => names[w]))} se partagent le pot : ${plural(e.share, "jeton")} chacun.`;
  }
}

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

export function GoblinPokerBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as GoblinPokerView;
  const legal = legalActions as GoblinPokerAction[];

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.out ? " lost" : ""}`;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Jetons depuis le début de la partie">{signed(seat.chips)}</span>
        </h3>
        <div className="stack">
          {seat.stack.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        {seat.out ? <p className="odds">hors de la main</p> : <CardBack count={seat.handSize} label="en main" />}
        {seat.pitched && <p className="odds">a jeté</p>}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Main {v.handNo} / {v.handsTotal}
        {v.round > 0 ? ` · Tour ${v.round}` : ""} · Pot : {v.pot}
      </p>
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
        <CardBack count={v.discard} label="Défausse" />
      </div>
      {v.discardSeen.length > 0 && (
        <div className="pairs-seen" title="Cartes défaussées face visible">
          {v.discardSeen.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      <div className="pairs-moves">
        {legal.includes("Fold") && (
          <button type="button" onClick={() => submit("Fold" satisfies GoblinPokerAction)}>
            Se coucher{v.foldCost !== null ? ` (${plural(v.foldCost, "jeton")})` : ""}
          </button>
        )}
      </div>
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per rank the engine accepts, on its first card.
            const first = v.hand.indexOf(card) === k;
            const move = first
              ? legal.find(
                  (a) =>
                    typeof a === "object" &&
                    (("Place" in a && a.Place.card === card) || ("Pitch" in a && a.Pitch.card === card)),
                )
              : undefined;
            let title: string | undefined;
            if (move && typeof move === "object") {
              title = "Place" in move ? `Commencer ma pile avec ce ${card}` : `Jeter ce ${card}`;
            }
            return <Card key={k} rank={card} title={title} onClick={move ? () => submit(move) : undefined} />;
          })}
        </div>
      )}
      {me !== null && v.pitched !== null && <p className="hint">Tu as jeté un {v.pitched}.</p>}
      {v.phase === "Setup" && legal.length > 0 && <p className="hint">Choisis la première carte de ta pile.</p>}
      {v.phase === "Setup" && me !== null && legal.length === 0 && (
        <p className="hint">
          {v.placed !== null ? `Tu as posé un ${v.placed}. ` : ""}En attente des autres joueurs…
        </p>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeGoblinPokerEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
