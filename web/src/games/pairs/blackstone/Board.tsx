import { useState } from "react";
import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { BlackstoneAction, BlackstoneEvent, BlackstoneStatus, BlackstoneView } from "./types";

/** « 1 pierre noire », « 2 pierres noires ». */
function plural(count: number, words: string): string {
  return `${count} ${count > 1 ? words.replace(/(\S+)/g, "$1s") : words}`;
}

export function describeBlackstoneEvent(e: BlackstoneEvent, names: string[]): string {
  switch (e.type) {
    case "Opened": {
      const cards = e.cards.map((c, i) => `${names[i]} ${c.join("-")}`).join(", ");
      const back = e.returned > 0 ? ` ${plural(e.returned, "pierre noire")} en trop retourne${e.returned > 1 ? "nt" : ""} au paquet.` : "";
      return `Main ${e.hand} : chacun mise 5. Ouverture : ${cards}. ${names[e.first]} commence.${back}`;
    }
    case "Took":
      return `${names[e.seat]} annonce ${e.count} et tire ${e.cards.join(", ")}.`;
    case "Folded":
      return `${names[e.seat]} se couche et paie ${plural(e.paid, "jeton")}.`;
    case "Lost":
      return `${names[e.seat]} a deux pierres noires : perdu, ${plural(e.paid, "jeton")} au pot.`;
    case "PotWon": {
      if (e.seats.length === 0) return "Personne ne survit.";
      const left = e.left > 0 ? ` ${plural(e.left, "jeton")} reste${e.left > 1 ? "nt" : ""} au pot.` : "";
      return `${e.seats.map((s) => names[s]).join(", ")} reprennent leur mise et partagent le pot : ${plural(e.share, "jeton")} chacun.${left}`;
    }
  }
}

const STATUS: Record<BlackstoneStatus, string> = { In: "", Folded: "Couché", Lost: "Perdu" };

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

/** Choose how many cards to take, or fold. */
export function TakeControls({
  legal,
  count,
  foldCost,
  onCount,
  submit,
}: {
  legal: BlackstoneAction[];
  count: number | null;
  foldCost: number;
  onCount(count: number): void;
  submit(action: BlackstoneAction): void;
}) {
  const counts = legal.flatMap((a) => (typeof a === "object" ? [a.Take.count] : []));
  const chosen = counts.includes(count ?? -1) ? (count as number) : counts[0];
  return (
    <div className="pairs-moves">
      {counts.length > 0 && (
        <>
          <select aria-label="Nombre de cartes" value={chosen} onChange={(e) => onCount(Number(e.target.value))}>
            {counts.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
          <button type="button" onClick={() => submit({ Take: { count: chosen } })}>
            Prendre {plural(chosen, "carte")}
          </button>
        </>
      )}
      {legal.includes("Fold") && (
        <button type="button" onClick={() => submit("Fold")}>
          Se coucher ({plural(foldCost, "jeton")})
        </button>
      )}
    </div>
  );
}

export function BlackstoneBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as BlackstoneView;
  const legal = legalActions as BlackstoneAction[];
  const [count, setCount] = useState<number | null>(null);

  const seats = v.seats.map((seat, i) => (
    <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}${seat.status === "In" ? "" : " lost"}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span title="Jetons depuis le début de la partie">{signed(seat.chips)}</span>
      </h3>
      <div className="stack">
        {Array.from({ length: seat.blacks }, (_, k) => (
          <Card key={k} rank={10} />
        ))}
      </div>
      <p className="hint">
        {plural(seat.blacks, "pierre noire")}
        {STATUS[seat.status] ? ` · ${STATUS[seat.status]}` : ""}
      </p>
    </section>
  ));

  const center = (
    <>
      <p className="pairs-round">
        Main {v.handNo} / {v.handsTotal} · Pot : {v.pot}
      </p>
      <CardBack count={v.deck} label="Paquet" />
      <p className="hint">{plural(v.whites.length, "pierre blanche")} :</p>
      <div className="pairs-seen">
        {v.whites.map((card, k) => (
          <Card key={k} rank={card} small />
        ))}
      </div>
      {legal.length > 0 && me !== null && (
        <>
          <p className="hint">Prends au moins {plural(v.minTake, "carte")}, annoncées d'avance, ou couche-toi.</p>
          <TakeControls
            legal={legal}
            count={count}
            foldCost={v.foldCost}
            onCount={setCount}
            submit={(a) => {
              setCount(null);
              submit(a);
            }}
          />
        </>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeBlackstoneEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
