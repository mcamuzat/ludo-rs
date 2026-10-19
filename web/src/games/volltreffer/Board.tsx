import { useState } from "react";
import type { BoardProps } from "../registry";
import type { Card, Place, Tribe, VolltrefferAction, VolltrefferView } from "./types";
import "./volltreffer.css";

const TRIBES: Record<Tribe, { name: string; mark: string }> = {
  Red: { name: "rouge", mark: "▲" },
  Green: { name: "vert", mark: "●" },
  Blue: { name: "bleu", mark: "■" },
  Yellow: { name: "jaune", mark: "★" },
  Purple: { name: "violet", mark: "◆" },
};

export function cardName(card: Card): string {
  return `${card.value} ${TRIBES[card.tribe].name}`;
}

function sameCard(a: Card, b: Card): boolean {
  return a.tribe === b.tribe && a.value === b.value;
}

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

/** The legal publication of exactly these combinations, with its points. */
export function publishAction(
  legal: VolltrefferAction[],
  points: number[],
  groups: number[],
  tribes: boolean,
): { action: VolltrefferAction; points: number } | null {
  const wanted = [...groups].sort((a, b) => a - b).join(",");
  const publishes = legal.filter((a) => "Publish" in a);
  const i = publishes.findIndex(
    (a) => "Publish" in a && a.Publish.tribes === tribes && a.Publish.groups.join(",") === wanted,
  );
  return i < 0 ? null : { action: publishes[i], points: points[i] ?? 0 };
}

/** The combinations on offer: every group value and the five tribes. */
export function offers(legal: VolltrefferAction[]): { groups: number[]; tribes: boolean } {
  const groups = new Set<number>();
  let tribes = false;
  for (const a of legal) {
    if ("Publish" in a) {
      a.Publish.groups.forEach((g) => groups.add(g));
      tribes ||= a.Publish.tribes;
    }
  }
  return { groups: [...groups].sort((a, b) => a - b), tribes };
}

export function VCard({
  card,
  onClick,
  dimmed,
  small,
  trump,
}: {
  card: Card;
  onClick?: () => void;
  dimmed?: boolean;
  small?: boolean;
  trump?: boolean;
}) {
  const cls = `vt-card vt-${card.tribe.toLowerCase()}${small ? " small" : ""}${dimmed ? " dimmed" : ""}${trump ? " trump" : ""}`;
  const body = (
    <>
      <span className="vt-value">{card.value}</span>
      <span className="vt-mark" aria-hidden="true">
        {TRIBES[card.tribe].mark}
      </span>
    </>
  );
  if (!onClick) {
    return (
      <span className={cls} role="img" aria-label={cardName(card)}>
        {body}
      </span>
    );
  }
  return (
    <button type="button" className={`${cls} playable`} onClick={onClick} aria-label={cardName(card)}>
      {body}
    </button>
  );
}

const PAWNS = ["#f4f1e8", "#222222", "#e07b24", "#1fa39a", "#d14f8f"];

function Track({ v, seatNames }: { v: VolltrefferView; seatNames: string[] }) {
  const [low, high] = v.sanctuary;
  const pawns = (n: number) =>
    v.seats.flatMap((s, i) =>
      s.score === n ? [<span key={i} className="vt-pawn" style={{ background: PAWNS[i] }} title={seatNames[i]} />] : [],
    );
  const zero = pawns(0);
  return (
    <div className="vt-track" role="group" aria-label="Piste de score">
      {Array.from({ length: 80 }, (_, k) => {
        const n = k + 1;
        const cls = `vt-space${n >= low && n <= high ? " sanctuary" : ""}${n === 66 ? " goal" : ""}${n > 70 ? " lava" : ""}`;
        return (
          <div key={n} className={cls}>
            <span className="vt-num">{n}</span>
            <span className="vt-pawns">{pawns(n)}</span>
          </div>
        );
      })}
      {zero.length > 0 && <p className="hint vt-zero">À 0 : {zero}</p>}
    </div>
  );
}

function DisplayPlace({
  place,
  onHire,
  trump,
  dimmed,
}: {
  place: Place;
  onHire?: () => void;
  trump: Tribe;
  dimmed: boolean;
}) {
  return (
    <figure className="vt-place">
      <figcaption>{place.cost}</figcaption>
      <VCard card={place.card} trump={place.card.tribe === trump} onClick={onHire} dimmed={dimmed} />
    </figure>
  );
}

function PublishPanel({ v, legal, submit }: { v: VolltrefferView; legal: VolltrefferAction[]; submit(a: unknown): void }) {
  const offer = offers(legal);
  const [groups, setGroups] = useState<number[]>(offer.groups);
  const [tribes, setTribes] = useState(offer.tribes);
  const chosen = publishAction(legal, v.publicationPoints, groups, tribes);
  const toggle = (g: number) => setGroups((gs) => (gs.includes(g) ? gs.filter((x) => x !== g) : [...gs, g]));
  const count = (g: number) => v.hand.filter((c) => c.value === g).length;
  return (
    <div className="vt-publish">
      {offer.groups.length === 0 && !offer.tribes && <p className="hint">Aucune combinaison à montrer.</p>}
      {offer.groups.map((g) => (
        <label key={g}>
          <input type="checkbox" checked={groups.includes(g)} onChange={() => toggle(g)} />
          {`Groupe de ${count(g)} × ${g}`}
        </label>
      ))}
      {offer.tribes && (
        <label>
          <input type="checkbox" checked={tribes} onChange={() => setTribes(!tribes)} />5 tribus
        </label>
      )}
      {chosen && (
        <button className="primary" onClick={() => submit(chosen.action)}>
          {chosen.points > 0 ? `Montrer (+${chosen.points})` : "Ne rien montrer"}
        </button>
      )}
    </div>
  );
}

const PHASES: Record<VolltrefferView["phase"], string> = {
  Hire: "Engager des aventuriers",
  Publish: "Montrer un groupe",
  Tricks: "Plis",
  Over: "Partie finie",
};

export function VolltrefferBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as VolltrefferView;
  const legal = legalActions as VolltrefferAction[];
  const myTurn = me !== null && v.current === me;
  const hire = (card: Card) => legal.find((a) => "Hire" in a && sameCard(a.Hire.card, card));
  const play = (card: Card) => legal.find((a) => "Play" in a && sameCard(a.Play.card, card));

  return (
    <div className="vt-board">
      <p className="pairs-round">{`Manche ${v.round} · ${PHASES[v.phase]}`}</p>
      <div className="vt-top felt">
        <figure className="vt-blessing">
          <VCard card={v.blessing} />
          <figcaption>{`Bénédiction : atout ${TRIBES[v.trump].name}`}</figcaption>
        </figure>
        {v.phase === "Hire" && (
          <div className="vt-display">
            {v.display.map((place) => {
              const action = myTurn ? hire(place.card) : undefined;
              return (
                <DisplayPlace
                  key={`${place.card.tribe}${place.card.value}`}
                  place={place}
                  trump={v.trump}
                  dimmed={myTurn && !place.hireable}
                  onHire={action ? () => submit(action) : undefined}
                />
              );
            })}
            <span className="hint">{`Pioche : ${v.deck}`}</span>
          </div>
        )}
        {(v.phase === "Tricks" || v.phase === "Over") && (
          <div className="vt-trick">
            {v.trick.length > 0 ? (
              v.trick.map(([seat, card], k) => (
                <figure key={k}>
                  <VCard card={card} trump={card.tribe === v.trump} />
                  <figcaption className="hint">{seatNames[seat]}</figcaption>
                </figure>
              ))
            ) : (
              <span className="hint">
                {v.lastTrick ? `Dernier pli pour ${seatNames[v.lastTrick.winner]}` : "Table vide"}
              </span>
            )}
          </div>
        )}
      </div>

      <div className="vt-seats">
        {v.seats.map((s, i) => (
          <section key={i} className={`vt-seat${v.current === i ? " active" : ""}`}>
            <h3>
              <span className="vt-pawn" style={{ background: PAWNS[i] }} aria-hidden="true" />
              <span>{seatNames[i]}</span>
              <strong>{s.score}</strong>
            </h3>
            <p className="hint">
              {[
                plural(s.cards, "carte"),
                v.phase === "Tricks" || s.line.tricks > 0 ? plural(s.line.tricks, "pli") : null,
                v.bargaining === i ? "négociation" : null,
                v.preemptive === i ? "action préventive" : null,
              ]
                .filter(Boolean)
                .join(" · ")}
            </p>
            {s.publication && (
              <div className="vt-row">
                {s.publication.cards.length > 0 ? (
                  s.publication.cards.map((c, k) => <VCard key={k} card={c} small />)
                ) : (
                  <span className="hint">rien montré</span>
                )}
                {s.publication.points > 0 && <span className="hint">{`+${s.publication.points}`}</span>}
              </div>
            )}
          </section>
        ))}
      </div>

      {v.recentHires.length > 0 && (
        <ul className="vt-log" aria-label="Derniers engagements">
          {v.recentHires.map((h, k) => (
            <li key={k}>
              {`${seatNames[h.seat]} engage `}
              <VCard card={h.card} small />
              {h.cost > 0 ? ` (−${h.cost})` : " (gratuit)"}
            </li>
          ))}
        </ul>
      )}

      {me !== null && v.hand.length > 0 && (
        <div className="vt-hand">
          {v.hand.map((card, i) => {
            const action = myTurn && v.phase === "Tricks" ? play(card) : undefined;
            return (
              <VCard
                key={i}
                card={card}
                trump={card.tribe === v.trump}
                dimmed={myTurn && v.phase === "Tricks" && !action}
                onClick={action ? () => submit(action) : undefined}
              />
            );
          })}
        </div>
      )}
      {myTurn && v.phase === "Hire" && <p className="hint">À toi : engage une carte de l'étal.</p>}
      {myTurn && v.phase === "Tricks" && (
        <p className="hint">{v.trick.length === 0 ? "À toi d'entamer." : "À toi : fournis la tribu si tu peux."}</p>
      )}
      {myTurn && v.phase === "Publish" && <PublishPanel key={v.round} v={v} legal={legal} submit={submit} />}

      <Track v={v} seatNames={seatNames} />

      {v.ranks && (
        <ol className="vt-ranking">
          {v.ranks
            .map((rank, i) => ({ rank, i }))
            .sort((a, b) => a.rank - b.rank)
            .map(({ rank, i }) => (
              <li key={i}>{`${rank + 1}. ${seatNames[i]} — ${v.seats[i].score}${rank === 0 ? " · victoire" : ""}`}</li>
            ))}
        </ol>
      )}

      {v.lastRound && (
        <table className="vt-round">
          <caption className="hint">{v.phase === "Over" ? "Dernière manche" : "Manche précédente"}</caption>
          <thead>
            <tr>
              <th />
              <th>Engagés</th>
              <th>Groupes</th>
              <th>Préventive</th>
              <th>Plis</th>
              <th>Score</th>
            </tr>
          </thead>
          <tbody>
            {v.lastRound.map((line, i) => (
              <tr key={i}>
                <th>{seatNames[i]}</th>
                <td>{line.spent > 0 ? `−${line.spent}` : "0"}</td>
                <td>{line.melds > 0 ? `+${line.melds}` : "—"}</td>
                <td>{line.preemptive ? "+5" : "—"}</td>
                <td>{line.tricks > 0 ? `+${line.tricks * 5}` : "—"}</td>
                <td>{line.score}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
