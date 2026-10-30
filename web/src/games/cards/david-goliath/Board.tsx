import type { BoardProps } from "../../registry";
import { CardBack, Table } from "../../pairs/cards";
import "../cards.css";
import type { DgAction, DgCard, DgColor, DgView } from "./types";

const COLOR_NAME: Record<DgColor, string> = {
  Red: "rouge",
  Yellow: "jaune",
  Green: "vert",
  Blue: "bleu",
  Violet: "violet",
};
const COLORS: DgColor[] = ["Red", "Yellow", "Green", "Blue", "Violet"];

/** "violet 12". */
export function cardName(card: DgCard): string {
  return `${COLOR_NAME[card.color]} ${card.value}`;
}

function sameCard(a: DgCard, b: DgCard): boolean {
  return a.color === b.color && a.value === b.value;
}

function DCard({
  card,
  small,
  dimmed,
  onClick,
}: {
  card: DgCard;
  small?: boolean;
  dimmed?: boolean;
  onClick?: () => void;
}) {
  const cls = `dg-card ${card.color.toLowerCase()}${small ? " small" : ""}${dimmed ? " dimmed" : ""}`;
  if (!onClick)
    return (
      <span className={cls} title={cardName(card)}>
        {card.value}
      </span>
    );
  return (
    <button type="button" className={`${cls} playable`} aria-label={cardName(card)} title={cardName(card)} onClick={onClick}>
      {card.value}
    </button>
  );
}

function TrickView({ cards, seatNames }: { cards: [number, DgCard][]; seatNames: string[] }) {
  return (
    <div className="dg-trick">
      {cards.map(([seat, card], k) => (
        <figure key={k}>
          <DCard card={card} />
          <figcaption className="hint">{seatNames[seat]}</figcaption>
        </figure>
      ))}
    </div>
  );
}

export function DavidGoliathBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as DgView;
  const legal = legalActions as DgAction[];
  const myTurn = me !== null && v.turn === me;
  const led = v.trick.length > 0 ? v.trick[0][1].color : null;

  const seats = v.handCounts.map((count, i) => (
    <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span>{`${v.scores[i]} pts`}</span>
      </h3>
      <div className="dg-won">
        {COLORS.map((color) => {
          const cards = v.won[i].filter((c) => c.color === color);
          return cards.length === 0 ? null : (
            <div key={color} className="dg-won-row">
              {cards.map((card, k) => (
                <DCard key={k} card={card} small />
              ))}
            </div>
          );
        })}
      </div>
      <p className="hint">{`Cette manche : ${v.points[i]} pts`}</p>
      {v.dealer === i && <p className="hint">donneur</p>}
      {i !== me && <CardBack count={count} label="en main" />}
    </section>
  ));

  const last = v.lastTrick;
  // The card the lowest took: the one its highest player played.
  const high = last ? (last.cards.find(([seat]) => seat === last.highest) ?? null) : null;
  const center = (
    <>
      <p className="pairs-round">{`Manche ${v.round} sur ${v.rounds}`}</p>
      {v.trick.length > 0 ? (
        <TrickView cards={v.trick} seatNames={seatNames} />
      ) : last ? (
        <TrickView cards={last.cards} seatNames={seatNames} />
      ) : (
        <span className="hint">Table vide</span>
      )}
      {v.trick.length === 0 && last && high && (
        <p className="hint">
          {last.highest === last.lowest
            ? `${seatNames[last.highest]} prend tout et commence.`
            : `${seatNames[last.lowest]} prend le ${cardName(high[1])} ; ${seatNames[last.highest]} prend le reste et commence.`}
        </p>
      )}
      {me !== null && v.hand.length > 0 && (
        <div className="dg-hand">
          {v.hand.map((card, i) => {
            const action = myTurn ? (legal.find((a) => sameCard(a.Play.card, card)) ?? null) : null;
            return (
              <DCard
                key={i}
                card={card}
                dimmed={myTurn && !action}
                onClick={action ? () => submit(action) : undefined}
              />
            );
          })}
        </div>
      )}
      {myTurn && (
        <p className="hint">
          {led === null
            ? "À toi de commencer."
            : legal.every((a) => a.Play.card.color === led)
              ? `Fournis le ${COLOR_NAME[led]}.`
              : `Tu n'as pas de ${COLOR_NAME[led]} : joue n'importe quelle carte.`}
        </p>
      )}
      {v.lastRound && (
        <table className="collusion-deal">
          <caption className="hint">Manche précédente</caption>
          <thead>
            <tr>
              <th />
              <th>Points</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {v.lastRound.map((line, i) => (
              <tr key={i}>
                <th>{seatNames[i]}</th>
                <td>{line.points}</td>
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
