import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { GhostTownAction, GhostTownEvent, GhostTownView } from "./types";

export function describeGhostTownEvent(e: GhostTownEvent, names: string[]): string {
  switch (e.type) {
    case "Placed":
      return `${names[e.seat]} pose un ${e.card} dans la rue.`;
    case "TieBreak":
      return `Départage : ${names[e.seat]} tire un ${e.card}.`;
    case "Starts":
      return `${names[e.seat]} commence.`;
    case "Played":
      return e.seat === e.to
        ? `${names[e.seat]} pose un ${e.card} dans sa maison.`
        : `${names[e.seat]} pose un ${e.card} chez ${names[e.to]}.`;
    case "Drew":
      return `${names[e.seat]} pioche un ${e.card} et gagne ${e.won} jetons${e.won < e.card ? " : le pot est vide" : ""}.`;
    case "Busted":
      return `${names[e.seat]} pioche un ${e.card} : paire ! ${e.card} jetons au pot, et dehors.`;
    case "Folded":
      return `${names[e.seat]} se couche.`;
    case "HandEnded":
      return e.winner === null
        ? "Le pot est vide : la main est finie."
        : `${names[e.winner]} reste seul et prend le pot : ${e.pot} jetons.`;
    case "Reshuffled":
      return "La défausse est remélangée.";
  }
}

/** A balance with its sign: +7, −14, 0. */
function signed(chips: number): string {
  if (chips > 0) return `+${chips}`;
  return chips < 0 ? `−${-chips}` : "0";
}

export function GhostTownBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as GhostTownView;
  const legal = legalActions as GhostTownAction[];

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.out ? " lost" : ""}`;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span title="Jetons depuis le début de la partie">{signed(seat.chips)}</span>
        </h3>
        <div className="stack">
          {seat.house.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        {seat.out ? <p className="odds">hors de la main</p> : <CardBack count={seat.handSize} label="en main" />}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Main {v.handNo} / {v.handsTotal} · Pot : {v.pot}
      </p>
      <div className="pairs-street" title="La rue : aucune maison ne peut contenir ces rangs">
        <span>Rue :</span>
        {v.street.map((card, k) => (
          <Card key={k} rank={card} />
        ))}
      </div>
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
        {legal.includes("Draw") && (
          <button type="button" onClick={() => submit("Draw" satisfies GhostTownAction)}>
            Piocher
          </button>
        )}
        {legal.includes("Fold") && (
          <button type="button" onClick={() => submit("Fold" satisfies GhostTownAction)}>
            Se coucher
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
                    (("Place" in a && a.Place.card === card) || ("Play" in a && a.Play.card === card)),
                )
              : undefined;
            let title: string | undefined;
            if (move && typeof move === "object" && "Play" in move) {
              const where = move.Play.to === me ? "chez toi" : `chez ${seatNames[move.Play.to]}`;
              title = `Poser ce ${card} ${where} (${v.playCost} jetons)`;
            } else if (move) {
              title = `Poser ce ${card} dans la rue`;
            }
            return <Card key={k} rank={card} title={title} onClick={move ? () => submit(move) : undefined} />;
          })}
        </div>
      )}
      {v.phase === "Setup" && legal.length > 0 && <p className="hint">Choisis la carte à poser dans la rue.</p>}
      {v.phase === "Setup" && me !== null && legal.length === 0 && (
        <p className="hint">
          {v.placed !== null ? `Tu as posé un ${v.placed}. ` : ""}En attente des autres joueurs…
        </p>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeGhostTownEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
