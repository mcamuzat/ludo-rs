import type { BoardProps } from "../../registry";
import { CardBack, Table } from "../../pairs/cards";
import { PlayingCard, cardName } from "../PlayingCard";
import type { Card, Rank } from "../types";
import type { Category, LamarckianAction, LamarckianEvent, LamarckianView, PokerHand } from "./types";

function sameCard(a: Card, b: Card): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** The legal mark or pick of `card`, if any. */
export function cardAction(card: Card, legal: LamarckianAction[], kind: "Mark" | "Pick"): LamarckianAction | null {
  return (
    legal.find((a) => (kind === "Mark" ? "Mark" in a && sameCard(a.Mark.card, card) : "Pick" in a && sameCard(a.Pick.card, card))) ??
    null
  );
}

const PLURAL_RANK: Record<Rank, string> = {
  Two: "deux",
  Three: "trois",
  Four: "quatre",
  Five: "cinq",
  Six: "six",
  Seven: "sept",
  Eight: "huit",
  Nine: "neuf",
  Ten: "dix",
  Jack: "valets",
  Queen: "dames",
  King: "rois",
  Ace: "as",
};

const SINGULAR_RANK: Record<Rank, string> = {
  ...PLURAL_RANK,
  Jack: "valet",
  Queen: "dame",
  King: "roi",
};

const CATEGORY: Record<Category, string> = {
  HighCard: "Hauteur",
  Pair: "Paire",
  TwoPair: "Deux paires",
  Trips: "Brelan",
  Straight: "Quinte",
  Flush: "Couleur",
  FullHouse: "Full",
  Quads: "Carré",
  StraightFlush: "Quinte flush",
};

function rankOf(card: Card | undefined): Rank | null {
  return card && card.kind === "Natural" ? card.rank : null;
}

/** "Brelan de rois", "Deux paires, dames et cinq", "Quinte au dix"… */
export function handName(hand: PokerHand): string {
  const first = rankOf(hand.cards[0]);
  if (first === null) return "Rien";
  const of = (r: Rank) => (r === "Ace" ? "d'as" : `de ${PLURAL_RANK[r]}`);
  const high = (r: Rank) => (r === "Ace" ? "à l'as" : `au ${SINGULAR_RANK[r]}`);
  switch (hand.category) {
    case "HighCard":
      return `Hauteur ${SINGULAR_RANK[first]}`;
    case "Pair":
    case "Trips":
    case "Quads":
      return `${CATEGORY[hand.category]} ${of(first)}`;
    case "TwoPair": {
      const second = rankOf(hand.cards[2]);
      return `Deux paires, ${PLURAL_RANK[first]} et ${second ? PLURAL_RANK[second] : "?"}`;
    }
    case "FullHouse": {
      const pair = rankOf(hand.cards[3]);
      return `Full aux ${PLURAL_RANK[first]} par les ${pair ? PLURAL_RANK[pair] : "?"}`;
    }
    case "Straight":
    case "Flush":
    case "StraightFlush":
      return `${CATEGORY[hand.category]} ${high(first)}`;
  }
}

function list(cards: Card[]): string {
  return cards.map(cardName).join(", ");
}

export function describeLamarckianEvent(e: LamarckianEvent, all: string[]): string {
  switch (e.type) {
    case "Competition":
      return `Concours ${e.no} : chacun reçoit 4 cartes.`;
    case "PoolDealt":
      return `Nouvelle réserve : ${list(e.cards)}.`;
    case "Revealed":
      return `On révèle : ${e.cards.flatMap((c, i) => (c === null ? [] : [`${all[i]} ${cardName(c)}`])).join(", ")}.`;
    case "Captured":
      return `${all[e.seat]} (${cardName(e.mark)}) prend ${list(e.cards)}.`;
    case "Picked":
      return `${all[e.seat]} (${cardName(e.mark)}) ne prend rien et choisit ${cardName(e.card)}.`;
    case "Showdown":
      return `Abattage : ${e.points.map((p, i) => `${all[i]} +${p} (${e.scores[i]})`).join(", ")}.`;
  }
}

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function LamarckianBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as LamarckianView;
  const legal = legalActions as LamarckianAction[];
  const picking = v.picking !== null && v.picking[0] === me;
  const marking = legal.some((a) => "Mark" in a);

  const seats = v.seats.map((seat, i) => {
    const hidden = seat.handSize - seat.known.length;
    return (
      <section key={i} className={`pairs-seat${v.picking?.[0] === i ? " active" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          <span>{plural(seat.score, "point")}</span>
        </h3>
        {seat.mark && (
          <p className="hint lamarckian-mark">
            Carte jouée : <PlayingCard card={seat.mark} small dimmed={!seat.waiting} />
          </p>
        )}
        {i !== me && (
          <>
            {seat.known.length > 0 && (
              <div className="lamarckian-known" aria-label={`Cartes prises par ${seatNames[i]}`}>
                {seat.known.map((card, k) => (
                  <PlayingCard key={k} card={card} small />
                ))}
              </div>
            )}
            {hidden > 0 && <CardBack count={hidden} label="cachées" />}
          </>
        )}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        {v.phase === "Over"
          ? "Partie terminée"
          : `Concours ${v.competition}/${v.competitions} · tour ${v.turnNo}/${v.turns}`}
      </p>
      <div className="lamarckian-pool" aria-label="La réserve">
        {v.pool.length === 0 && <span className="hint">Réserve vide</span>}
        {v.pool.map((card, k) => {
          const pick = picking ? cardAction(card, legal, "Pick") : null;
          return <PlayingCard key={k} card={card} onClick={pick ? () => submit(pick) : undefined} />;
        })}
      </div>
      <CardBack count={v.deckSize} label="pioche" />
      {picking && v.picking && (
        <p className="hint">{`Ton ${cardName(v.picking[1])} ne prend rien : choisis une carte de la réserve.`}</p>
      )}
      {marking && <p className="hint">Choisis ta carte, face cachée.</p>}
      {v.chosen !== null && <p className="hint">{`Ta carte : ${cardName(v.chosen)} (face cachée)`}</p>}
      {me !== null && v.hand.length > 0 && (
        <div className="lamarckian-hand">
          {v.hand.map((card, k) => {
            const play = cardAction(card, legal, "Mark");
            const chosen = v.chosen !== null && sameCard(v.chosen, card);
            return (
              <PlayingCard
                key={k}
                card={card}
                selected={chosen}
                onClick={play ? () => submit(play) : undefined}
              />
            );
          })}
        </div>
      )}
      {v.best && <p className="lamarckian-best">{`Ta meilleure main : ${handName(v.best)}`}</p>}
      {v.lastShowdown && (
        <table className="collusion-deal">
          <caption className="hint">Dernier abattage</caption>
          <tbody>
            {v.lastShowdown.map((line, i) => (
              <tr key={i}>
                <th>{seatNames[i]}</th>
                <td className="lamarckian-showdown-cards">
                  {line.hand.cards.map((card, k) => (
                    <PlayingCard key={k} card={card} small />
                  ))}
                </td>
                <td>{handName(line.hand)}</td>
                <td>{`+${line.points}`}</td>
                <td>{line.score}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeLamarckianEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
