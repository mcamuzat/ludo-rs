import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { DeadfallAction, DeadfallEvent, DeadfallView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeDeadfallEvent(e: DeadfallEvent, names: string[]): string {
  switch (e.type) {
    case "DoorsRevealed": {
      const doors = e.cards.map((card, i) => `${names[i]} ${card}`).join(", ");
      return e.leader === null
        ? `Portes : ${doors}. Égalité : chacun joue une autre porte.`
        : `Portes : ${doors}. ${names[e.leader]} commence.`;
    }
    case "Played":
      return `${names[e.seat]} joue un ${e.card}.`;
    case "Showdown": {
      const hands = e.hands
        .map((hand, i) => `${names[i]} ${hand.length > 0 ? hand.join("-") : "rien"}`)
        .join(", ");
      const verdict = e.dead
        ? `les ${e.rank} sont morts, appel réussi`
        : `il reste des ${e.rank}, appel raté`;
      const loser = e.winner === e.caller ? e.target : e.caller;
      return (
        `${names[e.caller]} appelle le ${e.rank} de ${names[e.target]}. Mains : ${hands}. ` +
        `Verdict : ${verdict}. ${names[loser]} paie ${plural(e.paid, "jeton")} à ${names[e.winner]}, ` +
        `qui prend le pot (${plural(e.pot, "jeton")}).`
      );
    }
  }
}

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

export function DeadfallBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as DeadfallView;
  const legal = legalActions as DeadfallAction[];

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}`;
    const top = seat.stack.length > 0 ? seat.stack[seat.stack.length - 1] : null;
    const callMove = legal.find((a) => "Call" in a && a.Call.seat === i);
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Jetons depuis le début de la partie">{signed(seat.chips)}</span>
        </h3>
        <div className="stack">
          {seat.stack.map((card, k) => (
            <Card key={k} rank={card} small={k < seat.stack.length - 1} />
          ))}
        </div>
        <CardBack count={seat.handSize} label="en main" />
        {callMove && top !== null && (
          <div className="pairs-moves">
            <button type="button" onClick={() => submit(callMove)}>
              Appeler ce {top} : mort !
            </button>
          </div>
        )}
      </section>
    );
  });

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
            // One button per rank the engine accepts, on its first card.
            const first = v.hand.indexOf(card) === k;
            const move = first
              ? legal.find(
                  (a) => ("Door" in a && a.Door.card === card) || ("Play" in a && a.Play.card === card),
                )
              : undefined;
            let title: string | undefined;
            if (move) title = "Door" in move ? `Choisir ce ${card} comme porte` : `Jouer ce ${card}`;
            return <Card key={k} rank={card} title={title} onClick={move ? () => submit(move) : undefined} />;
          })}
        </div>
      )}
      {v.phase === "Door" && legal.length > 0 && <p className="hint">Choisis ta carte de porte.</p>}
      {v.phase === "Door" && me !== null && legal.length === 0 && (
        <p className="hint">
          {v.placed !== null ? `Ta porte : ${v.placed}. ` : ""}En attente des autres joueurs…
        </p>
      )}
      {v.phase === "Play" && legal.some((a) => "Call" in a) && (
        <p className="hint">Joue une carte, ou appelle une carte du dessus que tu crois morte.</p>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeDeadfallEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
