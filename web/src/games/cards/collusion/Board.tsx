import type { BoardProps } from "../../registry";
import { CardBack, Table } from "../../pairs/cards";
import { PlayingCard } from "../PlayingCard";
import type { Card } from "../types";
import type { CollusionAction, CollusionView, Phrase } from "./types";

function sameCard(a: Card, b: Card): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** The legal play of `card`, if any. */
export function cardAction(card: Card, legal: CollusionAction[]): CollusionAction | null {
  return legal.find((a) => "Play" in a && sameCard(a.Play.card, card)) ?? null;
}

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function phraseText(p: Phrase): string {
  switch (p.type) {
    case "Aim":
      return `Je vise ${plural(p.n, "pli")}`;
    case "Unsure":
      return "Je ne sais pas encore";
    case "WhoAims":
      return `Qui vise ${p.n} ?`;
  }
}

function phrases(legal: CollusionAction[]): Phrase[] {
  return legal.flatMap((a) => ("Say" in a ? [a.Say.phrase] : []));
}

export function CollusionBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as CollusionView;
  const legal = legalActions as CollusionAction[];
  const said = phrases(legal);
  const aims = said.filter((p) => p.type === "Aim");
  const asks = said.filter((p) => p.type === "WhoAims");
  const unsure = said.find((p) => p.type === "Unsure");
  const say = (phrase: Phrase) => submit({ Say: { phrase } });
  const myTurn = me !== null && v.turn === me;

  const seats = v.handCounts.map((count, i) => {
    const phrase = v.lastSaid[i];
    return (
      <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          <span>{plural(v.scores[i], "point")}</span>
        </h3>
        <p className="hint">{`${plural(v.taken[i], "pli")}${v.dealer === i ? " · donneur" : ""}`}</p>
        {i !== me && <CardBack count={count} label="en main" />}
        {phrase && <p className="collusion-bubble">{`« ${phraseText(phrase)} »`}</p>}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">{`Donne ${v.dealNo} · cible ${v.target} points`}</p>
      <div className="collusion-trick">
        {v.trick.length > 0 ? (
          v.trick.map(([seat, card], k) => (
            <figure key={k}>
              <PlayingCard card={card} />
              <figcaption className="hint">{seatNames[seat]}</figcaption>
            </figure>
          ))
        ) : (
          <span className="hint">
            {v.lastTrick ? `Dernier pli pour ${seatNames[v.lastTrick.winner]}` : "Table vide"}
          </span>
        )}
      </div>
      {me !== null && v.hand.length > 0 && (
        <div className="collusion-hand">
          {v.hand.map((card, i) => {
            const play = myTurn ? cardAction(card, legal) : null;
            return (
              <PlayingCard
                key={i}
                card={card}
                dimmed={myTurn && !play}
                onClick={play ? () => submit(play) : undefined}
              />
            );
          })}
        </div>
      )}
      {myTurn && <p className="hint">{v.trick.length === 0 ? "À toi d'entamer." : "À toi : fournis si tu peux."}</p>}
      {said.length > 0 && (
        <details className="collusion-talk">
          <summary>Parler</summary>
          <div className="collusion-phrases">
            <span>Je vise</span>
            {aims.map((p) => (
              <button key={p.n} className="collusion-n" aria-label={phraseText(p)} onClick={() => say(p)}>
                {p.n}
              </button>
            ))}
            <span>pli(s)</span>
          </div>
          <div className="collusion-phrases">
            <span>Qui vise</span>
            {asks.map((p) => (
              <button key={p.n} className="collusion-n" aria-label={phraseText(p)} onClick={() => say(p)}>
                {p.n}
              </button>
            ))}
            <span>?</span>
          </div>
          {unsure && <button onClick={() => say(unsure)}>{phraseText(unsure)}</button>}
        </details>
      )}
      {v.talk.length > 0 && (
        <details className="pairs-events">
          <summary>{`Paroles de la donne (${v.talk.length})`}</summary>
          <ul>
            {v.talk.map(([seat, p], k) => (
              <li key={k}>{`${seatNames[seat]} : « ${phraseText(p)} »`}</li>
            ))}
          </ul>
        </details>
      )}
      {v.lastDeal && (
        <table className="collusion-deal">
          <caption className="hint">Donne précédente</caption>
          <thead>
            <tr>
              <th />
              <th>Plis</th>
              <th>Bonus</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {v.lastDeal.map((line, i) => (
              <tr key={i}>
                <th>{seatNames[i]}</th>
                <td>{line.deducted ? `−${line.tricks} (retranché)` : `+${line.tricks}`}</td>
                <td>{line.bonus > 0 ? `+${line.bonus}` : "—"}</td>
                <td>{line.total}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
