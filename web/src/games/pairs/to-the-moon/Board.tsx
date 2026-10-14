import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { ToTheMoonAction, ToTheMoonEvent, ToTheMoonView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeToTheMoonEvent(e: ToTheMoonEvent, names: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `Manche ${e.round} : ${names[e.dealer]} distribue.`;
    case "Led":
      return `${names[e.seat]} lance un compte à rebours avec un ${e.card}.`;
    case "Built":
      return `${names[e.seat]} pose un ${e.card}.`;
    case "Tested":
      return e.ok
        ? `${names[e.seat]} teste : un ${e.card}, ça passe.`
        : `${names[e.seat]} teste : un ${e.card}, déjà là ! Dehors, le ${e.card} en main.`;
    case "Withdrew":
      return e.took === null
        ? `${names[e.seat]} se retire.`
        : `${names[e.seat]} se retire et prend le ${e.took}.`;
    case "TrickWon":
      return e.perfect
        ? `${names[e.seat]} complète le compte à rebours : décollage ! ${plural(e.cards, "point")}.`
        : `${names[e.seat]} gagne le pli : ${plural(e.cards, "point")}.`;
    case "RoundEnded":
      return `La pioche est vide : fin de la manche ${e.round}.`;
  }
}

function isBuild(a: ToTheMoonAction, card: number): boolean {
  return typeof a === "object" && "Build" in a && a.Build.card === card;
}

function isTake(a: ToTheMoonAction, card: number | null): boolean {
  return typeof a === "object" && "Withdraw" in a && a.Withdraw.take === card;
}

export function ToTheMoonBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as ToTheMoonView;
  const legal = legalActions as ToTheMoonAction[];

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.out ? " lost" : ""}`;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>
            {seatNames[i]}
            {v.dealer === i ? " (donneur)" : ""}
          </span>
          <span title="Points depuis le début de la partie">{seat.score}</span>
        </h3>
        <CardBack count={seat.handSize} label="en main" />
        {seat.known.length > 0 && (
          <div className="stack" title="Cartes de sa main que tout le monde a vues">
            {seat.known.map((card, k) => (
              <Card key={k} rank={card} small />
            ))}
          </div>
        )}
        {seat.out && <p className="odds">hors du pli</p>}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} / {v.roundsTotal}
      </p>
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
      </div>
      {v.trick.length > 0 && (
        <div className="pairs-seen" title="Le compte à rebours">
          {v.trick.map((card, k) => {
            const take = legal.find((a) => isTake(a, card));
            const title = take ? `Me retirer en prenant ce ${card}` : undefined;
            return <Card key={k} rank={card} title={title} onClick={take ? () => submit(take) : undefined} />;
          })}
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      <div className="pairs-moves">
        {legal.includes("Test") && (
          <button type="button" onClick={() => submit("Test" satisfies ToTheMoonAction)}>
            Tester la pioche
          </button>
        )}
        {legal.some((a) => isTake(a, null)) && (
          <button type="button" onClick={() => submit({ Withdraw: { take: null } } satisfies ToTheMoonAction)}>
            Me retirer
          </button>
        )}
      </div>
      {legal.some((a) => typeof a === "object" && "Withdraw" in a && a.Withdraw.take !== null) && (
        <p className="hint">Pour te retirer, clique la carte du compte à rebours que tu prends.</p>
      )}
      {me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per rank the engine accepts, on its first card.
            const first = v.hand.indexOf(card) === k;
            const move = first ? legal.find((a) => isBuild(a, card)) : undefined;
            const title = move ? `Poser ce ${card}` : undefined;
            return <Card key={k} rank={card} title={title} onClick={move ? () => submit(move) : undefined} />;
          })}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeToTheMoonEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
