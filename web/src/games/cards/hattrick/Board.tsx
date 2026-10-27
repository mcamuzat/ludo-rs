import type { BoardProps } from "../../registry";
import { CardBack, Table } from "../../pairs/cards";
import "../cards.css";
import type { Color, HattrickAction, HattrickCard as Card, HattrickView, Played } from "./types";

const COLORS: Color[] = ["Red", "Blue", "Green"];
const COLOR_NAME: Record<Color, string> = {
  Red: "rouge",
  Blue: "bleu",
  Green: "vert",
};
const COLOR_PILE: Record<Color, string> = {
  Red: "rouge",
  Blue: "bleue",
  Green: "verte",
};
const COLOR_PLURAL: Record<Color, string> = {
  Red: "rouges",
  Blue: "bleues",
  Green: "vertes",
};

/** "rouge 7". */
export function cardName(card: Card): string {
  return `${COLOR_NAME[card.color]} ${card.value}`;
}

function sameCard(a: Card, b: Card): boolean {
  return a.color === b.color && a.value === b.value;
}

/** The legal move with `card`, if any: a play, or a pass with it. */
export function cardAction(card: Card, legal: HattrickAction[]): HattrickAction | null {
  return legal.find((a) => ("Play" in a ? sameCard(a.Play.card, card) : sameCard(a.Pass.card, card))) ?? null;
}

function plural(count: number, word: string): string {
  return `${count} ${word}${Math.abs(count) > 1 ? "s" : ""}`;
}

function HCard({
  card,
  small,
  dimmed,
  pass,
  onClick,
}: {
  card: Card;
  small?: boolean;
  dimmed?: boolean;
  /** Clicking passes with it. */
  pass?: boolean;
  onClick?: () => void;
}) {
  const cls = `hattrick-card ${card.color.toLowerCase()}${small ? " small" : ""}${dimmed ? " dimmed" : ""}${
    pass ? " pass" : ""
  }`;
  if (!onClick)
    return (
      <span className={cls} title={cardName(card)}>
        {card.value}
      </span>
    );
  const label = pass ? `Passer avec le ${cardName(card)} (−2)` : cardName(card);
  return (
    <button type="button" className={`${cls} playable`} aria-label={label} title={label} onClick={onClick}>
      {card.value}
    </button>
  );
}

function PileView({
  color,
  cards,
  winner,
  seatNames,
}: {
  color: Color;
  cards: Played[];
  winner?: number;
  seatNames: string[];
}) {
  return (
    <div className="hattrick-pile">
      <p className="hint">
        {`Pile ${COLOR_PILE[color]}`}
        {winner !== undefined && ` → ${seatNames[winner]}`}
      </p>
      <div className="hattrick-pile-cards">
        {cards.map(([seat, card], k) => (
          <figure key={k}>
            <HCard card={card} small={winner !== undefined} />
            <figcaption className="hint">{seatNames[seat]}</figcaption>
          </figure>
        ))}
      </div>
    </div>
  );
}

export function HattrickBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as HattrickView;
  const legal = legalActions as HattrickAction[];
  const myTurn = me !== null && v.turn === me;
  const canPass = legal.some((a) => "Pass" in a);

  const seats = v.handCounts.map((count, i) => (
    <section key={i} className={`pairs-seat${v.turn === i ? " active" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span>{plural(v.scores[i], "point")}</span>
      </h3>
      <p className="hattrick-won">
        {COLORS.map((color, k) => (
          <span
            key={color}
            className={`hattrick-chip ${color.toLowerCase()}`}
            title={`${v.won[i][k]} ${COLOR_PLURAL[color]}`}
          >
            {v.won[i][k]}
          </span>
        ))}
        {v.passed[i] > 0 && (
          <span className="hattrick-chip passed" title="cartes posées en passant">
            {v.passed[i]}
          </span>
        )}
      </p>
      {v.dealer === i && <p className="hint">donneur</p>}
      {i !== me && <CardBack count={count} label="en main" />}
    </section>
  ));

  const last = v.lastTrick;
  const center = (
    <>
      <p className="pairs-round">{`Manche ${v.round} sur ${v.rounds}`}</p>
      <div className="hattrick-piles">
        {v.piles.length > 0 ? (
          v.piles.map((p) => <PileView key={p.color} color={p.color} cards={p.cards} seatNames={seatNames} />)
        ) : last ? (
          <>
            {last.piles.map((p) => (
              <PileView key={p.color} color={p.color} cards={p.cards} winner={p.winner} seatNames={seatNames} />
            ))}
          </>
        ) : (
          <span className="hint">Table vide</span>
        )}
      </div>
      {v.passes.length > 0 && (
        <p className="hint">
          {v.passes.map(([seat, card]) => `${seatNames[seat]} passe (${cardName(card)})`).join(" · ")}
        </p>
      )}
      {v.piles.length === 0 && last && (
        <p className="hint">{`Dernier tour : ${seatNames[last.leader]} a joué la carte la plus forte.`}</p>
      )}
      {me !== null && v.hand.length > 0 && (
        <div className="hattrick-hand">
          {v.hand.map((card, i) => {
            const action = myTurn ? cardAction(card, legal) : null;
            return (
              <HCard
                key={i}
                card={card}
                dimmed={myTurn && !action}
                pass={action !== null && "Pass" in action}
                onClick={action ? () => submit(action) : undefined}
              />
            );
          })}
        </div>
      )}
      {myTurn && (
        <p className="hint">
          {v.piles.length === 0
            ? "À toi de commencer."
            : canPass
              ? "Joue sur une des deux piles, ou passe avec une carte de la troisième couleur (−2)."
              : "À toi : n'importe quelle carte."}
        </p>
      )}
      {v.lastRound && (
        <table className="collusion-deal">
          <caption className="hint">Manche précédente</caption>
          <thead>
            <tr>
              <th />
              {COLORS.map((c) => (
                <th key={c}>{COLOR_PLURAL[c]}</th>
              ))}
              <th>Passées</th>
              <th>Points</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {v.lastRound.map((line, i) => (
              <tr key={i}>
                <th>{seatNames[i]}</th>
                {line.won.map((n, k) => (
                  <td key={k}>{n}</td>
                ))}
                <td>{line.passed}</td>
                <td>{line.points > 0 ? `+${line.points}` : line.points}</td>
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
