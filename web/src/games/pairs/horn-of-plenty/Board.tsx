import { useState } from "react";
import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { HornOfPlentyAction, HornOfPlentyEvent, HornOfPlentyView, Rank } from "./types";

function list(cards: Rank[], names: string[]): string {
  return cards.map((card, i) => `${names[i]} ${card}`).join(", ");
}

export function describeHornOfPlentyEvent(e: HornOfPlentyEvent, names: string[]): string {
  switch (e.type) {
    case "Dealt": {
      const ties =
        e.tie_breaks.length > 0
          ? ` Départage : ${e.tie_breaks.map(([seat, card]) => `${names[seat]} ${card}`).join(", ")} (défaussées).`
          : "";
      return `Manche ${e.round}. Cartes visibles : ${list(e.upcards, names)}.${ties} ${names[e.leader]} commence.`;
    }
    case "Declared":
      return `${names[e.seat]} pose une carte cachée : « c'est un ${e.declared} ».`;
    case "Passed":
      return `${names[e.seat]} passe.`;
    case "Answered": {
      const said = e.agreed ? "est d'accord" : "n'est pas d'accord";
      const verdict = e.right ? "vu juste" : "raté";
      return `${names[e.seat]} ${said} : c'est un ${e.card}, ${verdict}. ${names[e.chooser]} choisit qui le prend.`;
    }
    case "Unanswered":
      return `Tout le monde passe : c'est un ${e.card}.`;
    case "Given": {
      const move = e.by === e.to ? `${names[e.by]} prend le ${e.card}` : `${names[e.by]} donne le ${e.card} à ${names[e.to]}`;
      return e.paired ? `${move} : paire, les deux ${e.card} sont défaussés.` : `${move}.`;
    }
    case "Scored": {
      const points = list(e.points, names);
      const gained = e.gained.map((g, i) => `${names[i]} +${g} (${e.scores[i]})`).join(", ");
      return `Fin de manche. Points de cartes : ${points}. Points de partie : ${gained}.`;
    }
  }
}

/** The legal plays of `card`, one per declarable rank. */
export function declarations(legal: HornOfPlentyAction[], card: Rank): Rank[] {
  return legal.flatMap((a) =>
    typeof a === "object" && "Play" in a && a.Play.card === card ? [a.Play.declared] : [],
  );
}

/** Choose a face-down card, then the rank to declare. */
export function DeclarePicker({
  hand,
  legal,
  picked,
  onPick,
  submit,
}: {
  hand: Rank[];
  legal: HornOfPlentyAction[];
  picked: Rank | null;
  onPick(card: Rank | null): void;
  submit(action: HornOfPlentyAction): void;
}) {
  if (picked === null) {
    return (
      <div className="pairs-hand">
        <span>Pose une carte :</span>
        {hand.map((card, k) => {
          const playable = hand.indexOf(card) === k && declarations(legal, card).length > 0;
          return (
            <Card
              key={k}
              rank={card}
              title={playable ? `Poser ce ${card}` : undefined}
              onClick={playable ? () => onPick(card) : undefined}
            />
          );
        })}
      </div>
    );
  }
  return (
    <>
      <div className="pairs-hand">
        <span>Tu poses :</span>
        <Card rank={picked} raised />
      </div>
      <div className="pairs-moves">
        <span>Annoncer :</span>
        {declarations(legal, picked).map((declared) => (
          <button
            key={declared}
            type="button"
            onClick={() => submit({ Play: { card: picked, declared } })}
            title={declared === picked ? "La vérité" : "Un bluff"}
          >
            {declared}
          </button>
        ))}
        <button type="button" onClick={() => onPick(null)}>
          Changer de carte
        </button>
      </div>
    </>
  );
}

export function HornOfPlentyBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as HornOfPlentyView;
  const legal = legalActions as HornOfPlentyAction[];
  const [picked, setPicked] = useState<Rank | null>(null);
  const send = (action: HornOfPlentyAction) => {
    setPicked(null);
    submit(action);
  };
  const table = v.table;
  const active = v.phase === "Declare" ? v.turn : v.phase === "Answer" ? (table?.asked ?? null) : v.chooser;

  const seats = v.seats.map((seat, i) => {
    const give = legal.find((a) => typeof a === "object" && "Give" in a && a.Give.to === i);
    return (
      <section key={i} className={`pairs-seat${active === i ? " active" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Points de partie">{seat.score}</span>
        </h3>
        <div className="stack">
          {seat.shown.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        <p className="hint">{seat.cardPoints} points de cartes</p>
        <CardBack count={seat.handSize} label="cachées" />
        {give && table?.card != null && (
          <div className="pairs-moves">
            <button type="button" onClick={() => send(give)}>
              {i === me ? `Prendre le ${table.card}` : `Lui donner le ${table.card}`}
            </button>
          </div>
        )}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} · Partie en {v.target} points
      </p>
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {table && (
        <div className="pairs-hand">
          <span>
            {seatNames[table.poser]} annonce un {table.declared} :
          </span>
          {table.card === null ? <span className="pairs-card back">?</span> : <Card rank={table.card} />}
        </div>
      )}
      {v.phase === "Declare" && legal.length > 0 && (
        <DeclarePicker
          hand={v.hand}
          legal={legal}
          picked={picked !== null && v.hand.includes(picked) ? picked : null}
          onPick={setPicked}
          submit={send}
        />
      )}
      {v.phase !== "Declare" && me !== null && v.hand.length > 0 && (
        <div className="pairs-hand">
          <span>Tes cartes cachées :</span>
          {v.hand.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      )}
      {v.phase === "Answer" && legal.length > 0 && table && (
        <div className="pairs-moves">
          <button type="button" onClick={() => send("Agree")}>
            D'accord, c'est un {table.declared}
          </button>
          <button type="button" onClick={() => send("Disagree")}>
            Pas d'accord
          </button>
          <button type="button" onClick={() => send("Pass")}>
            Passer
          </button>
        </div>
      )}
      {v.phase === "Place" && legal.length > 0 && <p className="hint">Choisis qui prend la carte.</p>}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeHornOfPlentyEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
