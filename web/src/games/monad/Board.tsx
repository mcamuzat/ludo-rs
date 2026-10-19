import { useState } from "react";
import type { BoardProps } from "../registry";
import { COLOR_HEX, MonadCard, MonadDisc, RANK_NAME, cardName, colorName } from "./MonadCard";
import type { Card, Color, EventView, MonadAction, MonadView, Rank, Target } from "./types";
import "./monad.css";

const RANKS: Rank[] = ["Unit", "Bi", "Tri", "Quad", "Quint"];
const COLORS: Color[] = ["Red", "Orange", "Yellow", "Navy", "Sky", "Green"];
const COLUMN_RANKS: Rank[] = ["Bi", "Tri", "Quad", "Quint"];

const cardKey = (c: Card) => RANKS.indexOf(c.rank) * 10 + COLORS.indexOf(c.color);
const sameCards = (a: Card[], b: Card[]) => {
  const ka = a.map(cardKey).sort((x, y) => x - y);
  const kb = b.map(cardKey).sort((x, y) => x - y);
  return ka.length === kb.length && ka.every((k, i) => k === kb[i]);
};

/** The cards an action takes from the hand (none for the simple moves). */
function actionCards(a: MonadAction): Card[] | null {
  if (typeof a === "string") return null;
  if ("Trade" in a) return a.Trade.cards;
  if ("Buy" in a) return a.Buy.cards;
  return a.Leap.colors.map((color) => ({ rank: "Unit", color }));
}

/** The trades, buys and leaps that use exactly the selected cards. */
export function actionsForSelection(legal: MonadAction[], selected: Card[]): MonadAction[] {
  if (selected.length === 0) return [];
  return legal.filter((a) => {
    const cards = actionCards(a);
    return cards !== null && sameCards(cards, selected);
  });
}

function targetName(t: Target): string {
  return t === "Monad" ? "une Monade" : `un ${RANK_NAME[t.Card]}`;
}

function cardsText(cards: Card[]): string {
  return cards.map(cardName).join(" + ");
}

function gainedText(e: EventView): string {
  const parts = e.gained.map(cardName);
  if (e.monad) parts.push("une Monade");
  if (e.drawn) parts.push(cardName({ rank: "Unit", color: e.drawn }));
  else if (e.drew) parts.push("une Unité");
  return parts.length ? parts.join(", ") : "rien";
}

export function describeEvent(e: EventView, names: string[]): string {
  const who = names[e.seat];
  const a = e.action;
  if (a === "Draw") return e.drawn ? `${who} pioche ${cardName({ rank: "Unit", color: e.drawn })}` : `${who} pioche une Unité`;
  if (a === "Flip") return `${who} retourne la défausse`;
  if (a === "Pass") return `${who} passe`;
  if (a === "EndTurn") return `${who} finit son tour`;
  if (a === "DeclineBonus") return `${who} renonce au bonus`;
  if (a === "TakeBonus") return `${who} prend le bonus : ${gainedText(e)}`;
  if ("Trade" in a) return `${who} échange ${cardsText(a.Trade.cards)} contre ${gainedText(e)}`;
  if ("Buy" in a) return `${who} achète ${gainedText(e)} avec ${cardsText(a.Buy.cards)}`;
  return `${who} accapare ${gainedText(e)} avec ${a.Leap.colors.length} Unités`;
}

function actionLabel(a: MonadAction): string {
  if (a === "TakeBonus") return "Prendre le bonus";
  if (a === "DeclineBonus") return "Renoncer";
  if (a === "EndTurn") return "Finir le tour";
  if (a === "Draw") return "Piocher";
  if (a === "Flip") return "Retourner la défausse";
  if (a === "Pass") return "Passer";
  if ("Trade" in a) return "Échanger";
  if ("Buy" in a) return `Acheter ${targetName(a.Buy.target)}`;
  return "Accaparer";
}

function countsText(counts: number[]): string {
  const parts = counts.flatMap((n, i) =>
    n === 0 ? [] : [`${n} ${RANKS[i] === "Unit" ? (n > 1 ? "Unités" : "Unité") : RANK_NAME[RANKS[i]]}`],
  );
  return parts.length ? parts.join(" · ") : "main vide";
}

function Seat({
  v,
  seat,
  name,
  children,
}: {
  v: MonadView;
  seat: number;
  name: string;
  children?: React.ReactNode;
}) {
  const active = v.current === seat;
  return (
    <section className={`monad-seat${active ? " active" : ""}`}>
      <h3>
        <span>{name}</span>
        <span
          className="monad-identity"
          style={{ background: COLOR_HEX[v.identities[seat]] }}
          title={`Identité ${colorName(v.identities[seat])}`}
        />
        <span className={`monad-bonus${active && v.turn.bonusUsed ? " used" : ""}`}>BONUS</span>
        <span className="monad-monads">
          <MonadDisc size={18} />
          {`${v.monads[seat]}/${v.target}`}
        </span>
      </h3>
      {children ?? <p className="counts">{countsText(v.handCounts[seat])}</p>}
    </section>
  );
}

/** What a selection belongs to: the position and my choices. */
function decisionKey(v: MonadView, legal: MonadAction[]): string {
  return JSON.stringify([v.thisTurn.length, v.current, v.hand, legal.length]);
}

export function MonadBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as MonadView;
  const legal = legalActions as MonadAction[];
  const key = decisionKey(v, legal);
  const [chosen, setChosen] = useState<{ key: string; picked: number[] }>({ key, picked: [] });
  const picked = chosen.key === key ? chosen.picked : [];
  const hand = v.hand ?? [];
  const selected = picked.map((i) => hand[i]).filter(Boolean);
  const myTurn = me !== null && v.current === me && legal.length > 0;

  const toggle = (i: number) =>
    setChosen({ key, picked: picked.includes(i) ? picked.filter((x) => x !== i) : [...picked, i] });
  const send = (a: MonadAction) => {
    setChosen({ key, picked: [] });
    submit(a);
  };

  const matching = actionsForSelection(legal, selected);
  const simple = legal.filter((a) => typeof a === "string");
  const others = Array.from({ length: v.players }, (_, i) => i).filter((i) => i !== me);

  const ranking =
    v.ranks &&
    v.ranks
      .map((rank, seat) => ({ rank, seat }))
      .sort((a, b) => a.rank - b.rank || a.seat - b.seat);
  const log = v.thisTurn.length > 0 ? v.thisTurn : v.lastTurn;

  return (
    <div className="monad-board">
      <div className="monad-seats">
        {others.map((i) => (
          <Seat key={i} v={v} seat={i} name={seatNames[i]} />
        ))}
      </div>

      <div className="monad-table felt">
        <div className="monad-pile">
          {v.deck > 0 ? <span className="monad-back" /> : <span className="monad-empty" />}
          <span>{`Pioche : ${v.deck}`}</span>
        </div>
        <div className="monad-pile">
          {v.discard.length > 0 ? (
            <MonadCard card={{ rank: "Unit", color: v.discard[v.discard.length - 1] }} />
          ) : (
            <span className="monad-empty" />
          )}
          <span>{`Défausse : ${v.discard.length}`}</span>
        </div>
        {COLUMN_RANKS.map((rank, i) => (
          <div key={rank} className="monad-column">
            <h4>{RANK_NAME[rank]}</h4>
            {v.columns[i].length === 0 ? (
              <span className="monad-empty" />
            ) : (
              // The top card lies over the others, which show their lower edge.
              v.columns[i].map((color, j) => (
                <span key={`${color}${j}`} className="monad-slot" style={{ zIndex: v.columns[i].length - j }}>
                  <MonadCard card={{ rank, color }} />
                </span>
              ))
            )}
          </div>
        ))}
        <div className="monad-pile">
          <MonadDisc size={44} />
          <span>{`Monades : ${v.monadStock}`}</span>
        </div>
      </div>

      {log.length > 0 && (
        <ul className="monad-log">
          {log.map((e, i) => (
            <li key={i}>{describeEvent(e, seatNames)}</li>
          ))}
        </ul>
      )}

      {ranking && (
        <>
          <p className="status">
            {v.winner !== null
              ? `${seatNames[v.winner]} gagne avec ${v.monads[v.winner]} Monades.`
              : "Partie arrêtée : plus personne ne progresse."}
          </p>
          <ol className="monad-ranking">
            {ranking.map(({ rank, seat }) => (
              <li key={seat}>{`${rank + 1}. ${seatNames[seat]} (${v.monads[seat]} Monades)`}</li>
            ))}
          </ol>
        </>
      )}

      {me !== null && (
        <Seat v={v} seat={me} name={seatNames[me]}>
          <div className="monad-hand">
            {hand.map((card, i) => (
              <MonadCard
                key={i}
                card={card}
                selected={picked.includes(i)}
                onClick={myTurn && v.turn.bonusOffer === null ? () => toggle(i) : undefined}
              />
            ))}
          </div>
        </Seat>
      )}

      {myTurn && (
        <div className="monad-actions">
          {v.turn.bonusOffer !== null && <p className="hint">{`Bonus de l'échange de ${RANK_NAME[v.turn.bonusOffer]} :`}</p>}
          {matching.map((a, i) => (
            <button key={i} className="primary" onClick={() => send(a)}>
              {actionLabel(a)}
            </button>
          ))}
          {simple.map((a) => (
            <button key={String(a)} className={a === "TakeBonus" ? "primary" : undefined} onClick={() => send(a)}>
              {actionLabel(a)}
            </button>
          ))}
          {v.turn.bonusOffer === null && matching.length === 0 && (
            <p className="hint">Choisis des cartes de ta main pour échanger, acheter ou accaparer.</p>
          )}
        </div>
      )}
    </div>
  );
}
