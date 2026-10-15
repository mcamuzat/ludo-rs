import { useState } from "react";
import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { Rank, TrollGutsAction, TrollGutsEvent, TrollGutsView } from "./types";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describeTrollGutsEvent(e: TrollGutsEvent, names: string[]): string {
  switch (e.type) {
    case "HandStarted":
      return `Main ${e.hand} : chacun mise 1 jeton. ${names[e.first]} jouera en premier.`;
    case "Dealt":
      return `${names[e.seat]} reçoit un ${e.card} visible.`;
    case "Replaced":
      return `Départage : le ${e.card} de ${names[e.seat]} ferait une paire, il est remplacé.`;
    case "Arranged":
      return "Chacun a rangé sa pile.";
    case "Took": {
      let from = "pioche";
      if (e.from.kind === "Stack") from = e.from.seat === e.seat ? "achète le dessus de sa pile" : `vole le dessus de la pile de ${names[e.from.seat]}`;
      return from === "pioche" ? `${names[e.seat]} pioche un ${e.card}.` : `${names[e.seat]} ${from} : un ${e.card}.`;
    }
    case "Busted":
      return `${names[e.seat]} prend un ${e.card} : paire, paie ${plural(e.paid, "jeton")} et sort.`;
    case "Reshuffled":
      return "La défausse est remélangée.";
    case "PotWon":
      if (e.seats.length === 0) return "Personne ne survit : le pot reste.";
      return `${e.seats.map((s) => names[s]).join(", ")} ${e.seats.length > 1 ? "se partagent" : "prend"} le pot : ${plural(e.share, "jeton")}${e.seats.length > 1 ? " chacun" : ""}.`;
  }
}

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

/** The legal order matching the picked cards, once all are picked. */
export function matchingOrder(legal: TrollGutsAction[], stack: Rank[], picked: number[]): TrollGutsAction | undefined {
  if (picked.length !== stack.length) return undefined;
  const order = picked.map((k) => stack[k]);
  return legal.find(
    (a) => typeof a === "object" && "Arrange" in a && a.Arrange.order.every((c, i) => c === order[i]),
  );
}

/** Pick the stack's cards in the order they will be revealed. */
export function StackOrderPicker({
  stack,
  legal,
  picked,
  onToggle,
  onReset,
  submit,
}: {
  stack: Rank[];
  legal: TrollGutsAction[];
  picked: number[];
  onToggle(k: number): void;
  onReset(): void;
  submit(action: TrollGutsAction): void;
}) {
  const order = matchingOrder(legal, stack, picked);
  return (
    <>
      <div className="pairs-hand">
        <span>Ta pile, à ranger :</span>
        {stack.map((card, k) =>
          picked.includes(k) ? null : (
            <Card key={k} rank={card} title={`Placer ce ${card} ensuite`} onClick={() => onToggle(k)} />
          ),
        )}
      </div>
      <div className="pairs-hand">
        <span>Du dessus au fond :</span>
        {picked.map((k, i) => (
          <Card key={i} rank={stack[k]} small />
        ))}
      </div>
      <div className="pairs-moves">
        <button type="button" disabled={!order} onClick={() => order && submit(order)}>
          Valider l'ordre
        </button>
        <button type="button" disabled={picked.length === 0} onClick={onReset}>
          Recommencer
        </button>
      </div>
    </>
  );
}

export function TrollGutsBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as TrollGutsView;
  const legal = legalActions as TrollGutsAction[];
  const [picked, setPicked] = useState<number[]>([]);
  const send = (action: TrollGutsAction) => {
    setPicked([]);
    submit(action);
  };

  const seats = v.seats.map((seat, i) => {
    const steal = legal.find((a) => typeof a === "object" && "Steal" in a && a.Steal.from === i);
    return (
      <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}${seat.out ? " lost" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Jetons depuis le début de la partie">{signed(seat.chips)}</span>
        </h3>
        <div className="stack">
          {seat.faceup.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        {seat.out ? <p className="hint">Éliminé</p> : <CardBack count={seat.stackSize} label="pile cachée" />}
        {steal && (
          <div className="pairs-moves">
            <button type="button" onClick={() => send(steal)}>
              Voler le dessus
            </button>
          </div>
        )}
      </section>
    );
  });

  const arranging = v.phase === "Arrange" && legal.length > 0;
  const center = (
    <>
      <p className="pairs-round">
        Main {v.handNo} / {v.handsTotal} · Pot : {v.pot}
      </p>
      <CardBack count={v.deck} label="Pioche" />
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {arranging && me !== null && (
        <p className="hint">Clique tes cartes dans l'ordre où elles seront révélées, le dessus d'abord.</p>
      )}
      {arranging && me !== null && (
        <StackOrderPicker
          stack={v.stack}
          legal={legal}
          picked={picked.filter((k) => k < v.stack.length)}
          onToggle={(k) => setPicked((p) => [...p, k])}
          onReset={() => setPicked([])}
          submit={send}
        />
      )}
      {v.phase === "Arrange" && me !== null && legal.length === 0 && (
        <p className="hint">Ta pile est rangée. En attente des autres joueurs…</p>
      )}
      {!arranging && me !== null && v.stack.length > 0 && (
        <div className="pairs-hand">
          <span>Ta pile (dessus d'abord) :</span>
          {v.stack.map((card, k) => (
            <Card key={k} rank={card} small={k > 0} />
          ))}
        </div>
      )}
      {v.phase === "Play" && legal.length > 0 && (
        <div className="pairs-moves">
          {legal.includes("Draw") && (
            <button type="button" onClick={() => send("Draw")}>
              Piocher
            </button>
          )}
          {legal.includes("Buy") && (
            <button type="button" onClick={() => send("Buy")}>
              Acheter mon {v.stack[0]} (1 jeton)
            </button>
          )}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeTrollGutsEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
