import type { ReactNode } from "react";
import type { BoardProps } from "../../registry";
import { CardBack, Table } from "../../pairs/cards";
import "../cards.css";
import type { CbAction, CbCard, CbSuit, CbView, Play, Share, TrickResult, Who } from "./types";

const SUIT_NAME: Record<CbSuit, string> = {
  Red: "rouge",
  Green: "vert",
  Blue: "bleu",
  Yellow: "jaune",
};

/** "vert 9". */
export function cardName(card: CbCard): string {
  return `${SUIT_NAME[card.suit]} ${card.value}`;
}

/** 1 cobra for 1–4, 2 for 5–9, 3 for 10–13: printed on the card. */
function cobrasOn(card: CbCard): number {
  return card.value <= 4 ? 1 : card.value <= 9 ? 2 : 3;
}

function plural(n: number, word: string): string {
  return `${n} ${word}${n > 1 ? "s" : ""}`;
}

function sameCard(a: CbCard, b: CbCard): boolean {
  return a.suit === b.suit && a.value === b.value;
}

function Snake() {
  return (
    <svg className="cb-snake" viewBox="0 0 10 14" aria-hidden="true">
      <path
        d="M7 3c-2-1.5-5 0-4 2.2s5 1.8 4 4.8-5 2.6-6 1.6"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
      />
      <circle cx="7.2" cy="2.6" r="1.7" fill="currentColor" />
    </svg>
  );
}

function CCard({
  card,
  small,
  dimmed,
  onClick,
}: {
  card: CbCard;
  small?: boolean;
  dimmed?: boolean;
  onClick?: () => void;
}) {
  const cls = `cb-card ${card.suit.toLowerCase()}${small ? " small" : ""}${dimmed ? " dimmed" : ""}`;
  const face = (
    <>
      <span className="cb-value">{card.value}</span>
      <span className="cb-snakes">
        {Array.from({ length: cobrasOn(card) }, (_, k) => (
          <Snake key={k} />
        ))}
      </span>
    </>
  );
  if (!onClick)
    return (
      <span className={cls} title={cardName(card)}>
        {face}
      </span>
    );
  return (
    <button type="button" className={`${cls} playable`} aria-label={cardName(card)} title={cardName(card)} onClick={onClick}>
      {face}
    </button>
  );
}

function KingCard({ share }: { share: Share }) {
  return (
    <span className="cb-card king" title="Cobra Royal">
      <span className="cb-king-name">Cobra Royal</span>
      <span className="cb-king-share">{share === "Win" ? "victoire" : "défaite"}</span>
    </span>
  );
}

function PlayView({ play, name }: { play: Play; name: string }) {
  return (
    <figure>
      {"Card" in play.played ? <CCard card={play.played.Card} /> : <KingCard share={play.played.King} />}
      <figcaption className="hint">{name}</figcaption>
    </figure>
  );
}

/** What happened in a trick, in a sentence or two. */
export function trickSummary(t: TrickResult, name: (w: Who) => string): string {
  const parts: string[] = [];
  parts.push(t.winner === null ? "Personne ne gagne le pli avec une carte." : `${name(t.winner)} gagne le pli.`);
  for (const s of t.scored) {
    const who = name({ Seat: s.seat });
    parts.push(
      s.cobras === 0
        ? `${who} n'a rien à rendre.`
        : `${who} rend ${plural(s.cobras, "cobra")} : +${plural(s.points, "pt")}.`,
    );
  }
  if (t.losers.length > 0 && t.share > 0) {
    const names = t.losers.map(name).join(", ");
    parts.push(
      t.losers.length === 1
        ? `${names} reçoit ${plural(t.share, "cobra")}.`
        : `${names} reçoivent ${plural(t.share, "cobra")} chacun.`,
    );
  }
  if (t.center > 0) parts.push(`${plural(t.center, "cobra")} au centre.`);
  return parts.join(" ");
}

export function CobrasBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as CbView;
  const legal = legalActions as CbAction[];
  const name = (w: Who) => (w === "Dummy" ? "Fictif" : seatNames[w.Seat]);
  const isTurn = (w: Who) =>
    v.turn !== null && (w === "Dummy" ? v.turn === "Dummy" : v.turn !== "Dummy" && v.turn.Seat === w.Seat);

  const playerSeat = (i: number) => (
    <section key={`p${i}`} className={`pairs-seat${isTurn({ Seat: i }) ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span>{`${v.scores[i]} pts`}</span>
      </h3>
      <p className="cb-basket">
        {`Panier : ${plural(v.baskets[i], "cobra")}`}
        {v.baskets[i] > 0 && <span className="hint">{` (${plural(v.basketPoints[i], "pt")})`}</span>}
      </p>
      <p className="hint">{v.kings[i] ? "Cobra Royal en main" : "Cobra Royal joué"}</p>
      {v.revealed[i] && (
        <p className="hint cb-seed">
          amorce <CCard card={v.revealed[i]} small />
        </p>
      )}
      {i !== me && <CardBack count={v.handCounts[i]} label="en main" />}
    </section>
  );

  const seats: ReactNode[] = [];
  for (let i = 0; i < v.handCounts.length; i++) {
    seats.push(playerSeat(i));
    // The dummy sits at the dealer's left.
    if (v.dummy && i === v.dealer) {
      seats.push(
        <section key="dummy" className={`pairs-seat${isTurn("Dummy") ? " active" : ""}`}>
          <h3>
            <span>Fictif</span>
          </h3>
          {v.dummy.top ? (
            <p className="hint cb-seed">
              dessus <CCard card={v.dummy.top} small />
            </p>
          ) : null}
          <CardBack count={v.dummy.count} label="dans sa pile" />
        </section>,
      );
    }
  }
  const myPlace = me === null ? 0 : me + (v.dummy && me > v.dealer ? 1 : 0);

  const mySeed = v.phase === "Seed" && legal.some((a) => "Seed" in a);
  const myTurn = v.phase === "Trick" && me !== null && isTurn({ Seat: me });
  const actionFor = (card: CbCard): CbAction | null =>
    legal.find((a) => ("Seed" in a ? sameCard(a.Seed.card, card) : "Play" in a && sameCard(a.Play.card, card))) ??
    null;
  const kings = legal.filter((a): a is { King: { share: Share } } => "King" in a);
  const followsOnly = myTurn && v.led !== null && legal.every((a) => !("Play" in a) || a.Play.card.suit === v.led);

  const center = (
    <>
      <p className="pairs-round">{`Manche ${v.round} · ${v.target} points pour finir`}</p>
      <p className="cb-status">
        <span>{`Triples : ${v.triples} / ${v.triplesToEnd}`}</span>
        <span>{`Centre : ${plural(v.center, "cobra")}`}</span>
      </p>
      {v.phase === "Seed" ? (
        <p className="hint">{`Chacun amorce son panier en secret. ${seatNames[v.firstLeader]} commencera le premier pli.`}</p>
      ) : v.trick.length > 0 ? (
        <div className="cb-trick">
          {v.trick.map((p, k) => (
            <PlayView key={k} play={p} name={name(p.who)} />
          ))}
        </div>
      ) : v.lastTrick ? (
        <>
          <div className="cb-trick">
            {v.lastTrick.plays.map((p, k) => (
              <PlayView key={k} play={p} name={name(p.who)} />
            ))}
          </div>
          <p className="hint">{trickSummary(v.lastTrick, name)}</p>
        </>
      ) : (
        <span className="hint">Table vide</span>
      )}
      {v.trick.length > 0 && v.lastTrick && (
        <p className="hint">{`Pli précédent : ${trickSummary(v.lastTrick, name)}`}</p>
      )}
      {me !== null && v.hand.length > 0 && (
        <div className="cb-hand">
          {v.hand.map((card, i) => {
            const action = mySeed || myTurn ? actionFor(card) : null;
            const chosen = v.chosen !== null && sameCard(v.chosen, card);
            return (
              <span key={i} className={chosen ? "cb-chosen" : undefined}>
                <CCard
                  card={card}
                  dimmed={myTurn && !action}
                  onClick={action ? () => submit(action) : undefined}
                />
              </span>
            );
          })}
        </div>
      )}
      {kings.length > 0 && (
        <div className="cb-kings">
          {kings.map((a) => (
            <button key={a.King.share} type="button" className="cb-king-button" onClick={() => submit(a)}>
              {a.King.share === "Win" ? "Cobra Royal : partager la victoire" : "Cobra Royal : partager la défaite"}
            </button>
          ))}
        </div>
      )}
      {mySeed && <p className="hint">Choisis la carte qui amorce ton panier.</p>}
      {v.phase === "Seed" && v.chosen && (
        <p className="hint">{`Ton panier sera amorcé avec le ${cardName(v.chosen)}. On attend les autres.`}</p>
      )}
      {myTurn && (
        <p className="hint">
          {v.led === null
            ? "À toi d'ouvrir le pli."
            : followsOnly
              ? `Fournis le ${SUIT_NAME[v.led]}.`
              : `Tu n'as pas de ${SUIT_NAME[v.led]} : joue n'importe quelle carte.`}
        </p>
      )}
      {v.lastRound && (
        <table className="collusion-deal">
          <caption className="hint">Manche précédente</caption>
          <thead>
            <tr>
              <th />
              <th>Plis</th>
              <th>Panier</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {v.lastRound.map((line, i) => (
              <tr key={i}>
                <th>{seatNames[i]}</th>
                <td>{line.gained}</td>
                <td>{line.penalty > 0 ? `−${line.penalty}` : "0"}</td>
                <td>{line.total}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );

  return <Table seats={seats} first={myPlace} center={center} />;
}
