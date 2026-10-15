import { useState } from "react";
import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { MonsterAction, MonsterEvent, MonsterView, Rank } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

function who(seats: number[], names: string[]): string {
  return seats.map((s) => names[s]).join(" et ");
}

export function describeMonsterEvent(e: MonsterEvent, names: string[]): string {
  switch (e.type) {
    case "GameStarted":
      return `Partie ${e.game} : chacun mise ${plural(e.ante, "jeton")}. ${names[e.button]} a le bouton.`;
    case "Dealt":
      return `Manche ${e.round} : six cartes à ${who(e.seats, names)}.`;
    case "Discarded":
      return e.count === 0 ? `${names[e.seat]} garde sa main.` : `${names[e.seat]} donne ${plural(e.count, "carte")} au Monstre.`;
    case "Folded":
      return `${names[e.seat]} se couche.`;
    case "Showdown": {
      const hands = e.hands.flatMap((h, i) => (h === null ? [] : [`${names[i]} ${h.join("-")}`])).join(", ");
      const monster = e.monster.length > 0 ? e.monster.join("-") : "rien";
      return `Abattage. ${hands}. Monstre : ${monster}. ${who(e.champions, names)} ${e.won ? "bat le Monstre" : "perd contre le Monstre"}.`;
    }
    case "Penalty":
      return `${who(e.seats, names)} paie 5 jetons et sort.`;
    case "PotWon":
      return `${who(e.seats, names)} ${e.seats.length > 1 ? "se partagent" : "prend"} le pot : ${plural(e.share, "jeton")}${e.seats.length > 1 ? " chacun" : ""}.`;
    case "Replay":
      return "Plus personne en jeu : les mêmes joueurs rejouent la manche.";
  }
}

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

/** The legal discard matching the picked cards, if any. */
export function matchingDiscard(legal: MonsterAction[], hand: Rank[], picked: number[]): MonsterAction | undefined {
  const cards = picked.map((k) => hand[k]).sort((a, b) => a - b);
  return legal.find(
    (a) =>
      typeof a === "object" &&
      a.Discard.cards.length === cards.length &&
      a.Discard.cards.every((c, i) => c === cards[i]),
  );
}

/** My hand and its controls: pick cards, then discard them or fold. */
export function DiscardPicker({
  hand,
  legal,
  picked,
  onToggle,
  submit,
}: {
  hand: Rank[];
  legal: MonsterAction[];
  picked: number[];
  onToggle(k: number): void;
  submit(action: MonsterAction): void;
}) {
  const acting = legal.length > 0;
  const discard = acting ? matchingDiscard(legal, hand, picked) : undefined;
  return (
    <>
      <div className="pairs-hand">
        <span>Ta main :</span>
        {hand.map((card, k) => (
          <Card
            key={k}
            rank={card}
            raised={picked.includes(k)}
            title={acting ? (picked.includes(k) ? `Garder ce ${card}` : `Écarter ce ${card}`) : undefined}
            onClick={acting ? () => onToggle(k) : undefined}
          />
        ))}
      </div>
      {acting && (
        <div className="pairs-moves">
          <button type="button" disabled={!discard} onClick={() => discard && submit(discard)}>
            {picked.length === 0 ? "Garder ma main" : `Donner ${plural(picked.length, "carte")} au Monstre`}
          </button>
          <button type="button" onClick={() => submit("Fold")}>
            Se coucher
          </button>
        </div>
      )}
    </>
  );
}

export function MonsterBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as MonsterView;
  const legal = legalActions as MonsterAction[];
  const [picked, setPicked] = useState<number[]>([]);
  const send = (action: MonsterAction) => {
    setPicked([]);
    submit(action);
  };

  const seats = v.seats.map((seat, i) => {
    let status = "";
    if (!seat.inGame) status = "Hors jeu";
    else if (seat.folded) status = "Couché";
    else if (seat.acted) status = seat.discards === 0 ? "Garde sa main" : `${plural(seat.discards, "carte")} au Monstre`;
    return (
      <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}${seat.inGame ? "" : " lost"}`}>
        <h3>
          <span>
            {seatNames[i]}
            {v.button === i ? " ●" : ""}
          </span>
          <span title="Jetons depuis le début">{signed(seat.chips)}</span>
        </h3>
        {seat.inRound && !seat.folded && <CardBack count={seat.handSize} label="en main" />}
        {status && <p className="hint">{status}</p>}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Partie {v.gameNo} / {v.gamesTotal} · Manche {v.roundNo} · Pot : {v.pot}
      </p>
      <CardBack count={v.monsterSize} label="Monstre" />
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {me !== null && v.hand.length > 0 && (
        <DiscardPicker
          hand={v.hand}
          legal={legal}
          picked={picked.filter((k) => k < v.hand.length)}
          onToggle={(k) => setPicked((p) => (p.includes(k) ? p.filter((x) => x !== k) : [...p, k]))}
          submit={send}
        />
      )}
      {legal.length > 0 && (
        <p className="hint">Écarte 0 à 3 cartes pour garder une main sans paire, ou couche-toi.</p>
      )}
      {v.discarded.length > 0 && <p className="hint">Tu as donné au Monstre : {v.discarded.join(", ")}.</p>}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeMonsterEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
