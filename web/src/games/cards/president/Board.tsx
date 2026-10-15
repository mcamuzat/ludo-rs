import { useEffect, useState } from "react";
import type { BoardProps } from "../../registry";
import { CardBack, Table } from "../../pairs/cards";
import { PlayingCard, cardName } from "../PlayingCard";
import type { Card, Face } from "../types";
import type { PresidentAction, PresidentEvent, PresidentView, Title } from "./types";

export const TITLE_LABEL: Record<Title, string> = {
  President: "Président",
  VicePresident: "Vice-président",
  Neutral: "Neutre",
  Secretary: "Secrétaire",
  Scum: "Trou du cul",
};

function faceKey(face: Face): string {
  return face.kind === "Joker" ? "Joker" : face.rank;
}

function cardFaceKey(card: Card): string {
  return card.kind === "Joker" ? "Joker" : card.rank;
}

function sameFaces(faces: Face[], selection: Card[]): boolean {
  const a = faces.map(faceKey).sort().join(",");
  const b = selection.map(cardFaceKey).sort().join(",");
  return a === b;
}

/** The legal action the selected cards make: a play of exactly those cards, or those cards handed back. */
export function selectionAction(selection: Card[], legal: PresidentAction[]): PresidentAction | null {
  if (selection.length === 0) return null;
  const naturals = selection.filter((c) => c.kind === "Natural");
  const jokers = selection.length - naturals.length;
  return (
    legal.find((a) => {
      if (a === "Pass") return false;
      if ("Give" in a) return sameFaces(a.Give.faces, selection);
      const play = a.Play;
      return (
        play.naturals === naturals.length &&
        play.jokers === jokers &&
        naturals.every((c) => c.kind === "Natural" && c.rank === play.rank)
      );
    }) ?? null
  );
}

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describePresidentEvent(e: PresidentEvent, names: string[]): string {
  switch (e.type) {
    case "RoundStarted": {
      const aside = e.aside.length > 0 ? ` Écartées : ${e.aside.map(cardName).join(", ")}.` : "";
      return `Manche ${e.round} : ${names[e.dealer]} distribue, ${names[e.leader]} commence.${aside}`;
    }
    case "Exchanged":
      return `${names[e.from]} donne ${plural(e.count, "carte")} à ${names[e.to]}.`;
    case "Played":
      return `${names[e.seat]} joue ${e.cards.map(cardName).join(", ")}.`;
    case "Passed":
      return `${names[e.seat]} passe.`;
    case "TrickClosed":
      return `Tout le monde passe : ${names[e.leader]} ouvre un nouveau pli.`;
    case "Revolution":
      return e.reversed ? "Révolution : l'ordre des cartes est inversé." : "Contre-révolution : l'ordre normal revient.";
    case "Finished":
      return e.fouled
        ? `${names[e.seat]} finit sur une carte interdite : Trou du cul.`
        : `${names[e.seat]} n'a plus de cartes.`;
    case "RoundScored":
      return `Fin de manche : ${e.order.map((seat) => `${names[seat]} +${e.points[seat]}`).join(", ")}.`;
  }
}

export function PresidentBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as PresidentView;
  const legal = legalActions as PresidentAction[];
  const [picked, setPicked] = useState<number[]>([]);
  // Every update starts a new decision: nothing selected.
  useEffect(() => setPicked([]), [view]);

  const selection = picked.flatMap((i) => (v.hand[i] ? [v.hand[i]] : []));
  const action = selectionAction(selection, legal);
  const gift = legal.find((a): a is { Give: { faces: Face[] } } => a !== "Pass" && "Give" in a);
  const playing = legal.some((a) => a === "Pass" || "Play" in a);
  const toggle = (i: number) => setPicked((p) => (p.includes(i) ? p.filter((x) => x !== i) : [...p, i]));
  const send = (a: PresidentAction) => {
    setPicked([]);
    submit(a);
  };

  const seats = v.handSizes.map((size, i) => {
    const fouled = v.fouled.includes(i);
    const out = fouled || v.out.includes(i);
    return (
      <section key={i} className={`pairs-seat${v.turn === i || v.toGive.includes(i) ? " active" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          <span>{plural(v.scores[i], "point")}</span>
        </h3>
        {v.titles.length > 0 && <p className="hint">{TITLE_LABEL[v.titles[i]]}</p>}
        {out ? (
          <p className="hint">{fouled ? "Sorti sur une carte interdite" : "Sorti"}</p>
        ) : (
          <CardBack count={size} label="en main" />
        )}
        {v.passes.includes(i) && <p className="hint">Passe</p>}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} / {v.roundsTotal}
      </p>
      {v.reversed && <p className="president-banner">Révolution : le 3 est la plus forte, le 2 la plus faible.</p>}
      <div className="president-trick">
        {v.trick ? (
          <>
            {v.trick.cards.map((card, k) => (
              <PlayingCard key={k} card={card} />
            ))}
            <span className="hint">{seatNames[v.trick.seat]}</span>
          </>
        ) : (
          <span className="hint">Table vide</span>
        )}
      </div>
      {me !== null && v.hand.length > 0 && (
        <div className="president-hand">
          {v.hand.map((card, i) => (
            <PlayingCard
              key={i}
              card={card}
              selected={picked.includes(i)}
              onClick={gift || playing ? () => toggle(i) : undefined}
            />
          ))}
        </div>
      )}
      {(gift || playing) && (
        <div className="row">
          <button className="primary" disabled={!action} onClick={() => action && send(action)}>
            {gift ? "Rendre" : "Jouer"}
          </button>
          {legal.includes("Pass") && <button onClick={() => send("Pass")}>Passer</button>}
        </div>
      )}
      {/* Template strings: one text node each, so the hint reads as one string. */}
      {gift && <p className="hint">{`Rends ${plural(gift.Give.faces.length, "carte")} de ton choix.`}</p>}
      {playing && !v.trick && <p className="hint">À toi d'ouvrir : une à quatre cartes de même valeur.</p>}
      {playing && v.trick && (
        <p className="hint">{`Pose ${plural(v.trick.cards.length, "carte")} de valeur plus forte, ou passe.`}</p>
      )}
      {v.phase === "Exchange" && !gift && <p className="hint">Échange des cartes en cours…</p>}
      {me !== null && (v.received.length > 0 || v.given.length > 0) && (
        <p className="hint">
          {`Échange :${v.received.length > 0 ? ` reçu ${v.received.map(cardName).join(", ")}` : ""}${
            v.received.length > 0 && v.given.length > 0 ? " ;" : ""
          }${v.given.length > 0 ? ` donné ${v.given.map(cardName).join(", ")}` : ""}.`}
        </p>
      )}
      {v.setAside.length > 0 && (
        <div className="pairs-seen">
          <span className="hint">Écartées :</span>
          {v.setAside.map((card, k) => (
            <PlayingCard key={k} card={card} small />
          ))}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describePresidentEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
