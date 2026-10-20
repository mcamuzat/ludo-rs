import { useState } from "react";
import type { BoardProps } from "../registry";
import { Sheet, type ManualMark } from "./Sheet";
import type { BlackViennaAction, BlackViennaEvent, BlackViennaView, CardId, Suspect, Trio } from "./types";
import "./black-vienna.css";

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

/** Three distinct letters, in the sheet's order; otherwise `null`. */
export function pickTrio(letters: Suspect[], suspects: Suspect[]): Trio | null {
  const unique = new Set(letters);
  if (letters.length !== 3 || unique.size !== 3) return null;
  const [a, b, c] = [...letters].sort((x, y) => suspects.indexOf(x) - suspects.indexOf(y));
  return [a, b, c];
}

/** Empty → + → − → ? → empty. */
export function cycleMark(mark: ManualMark | undefined): ManualMark | undefined {
  switch (mark) {
    case undefined:
      return "+";
    case "+":
      return "−";
    case "−":
      return "?";
    case "?":
      return undefined;
  }
}

/** Where a game stands, to notice a new one starting. */
export interface Progress {
  phase: BlackViennaView["phase"];
  answers: number;
}

/** A new game has started (a rematch on the same table): the notes go. */
export function isNewGame(prev: Progress, next: Progress): boolean {
  return (prev.phase === "Over" && next.phase !== "Over") || next.answers < prev.answers;
}

/** The legal question with exactly these choices. */
export function findAsk(
  legal: BlackViennaAction[],
  pile: number | null,
  target: number,
  extra: CardId | null,
): BlackViennaAction | undefined {
  return legal.find(
    (a) => "Ask" in a && a.Ask.pile === pile && a.Ask.target === target && a.Ask.extra === extra,
  );
}

const ENDS = {
  Solved: "",
  Reserve: "Plus assez de jetons : personne ne marque.",
  Stuck: "Plus rien à demander : personne ne marque.",
  Failed: "Le dernier enquêteur s'est trompé : personne ne marque.",
};

export function describeEvent(e: BlackViennaEvent, names: string[], cards: string[]): string {
  switch (e.type) {
    case "Asked": {
      const parts = e.answers
        .map((a) => `${cards[a.card]}${a.special ? " (spéciale)" : ""}, ${plural(a.count, "jeton")}`)
        .join(" ; ");
      return `${names[e.by]} interroge ${names[e.target]} : ${parts}.`;
    }
    case "Accused":
      return e.right ? `${names[e.seat]} accuse… et a raison !` : `${names[e.seat]} accuse… et se trompe.`;
    case "Revealed": {
      const who = e.accuser === null ? ENDS[e.end] : `Démasqué par ${names[e.accuser]}.`;
      const points = e.scores.map((p, i) => `${names[i]} ${p}`).join(", ");
      return `Black Vienna : ${e.trio.join("-")}. ${who} Points : ${points}.`;
    }
  }
}

/** A card of letters; a button when it can be chosen. */
function Letters({ text, onClick, selected, title }: { text: string; onClick?: () => void; selected?: boolean; title?: string }) {
  const cls = `bv-card${selected ? " selected" : ""}`;
  if (!onClick) return <span className={cls}>{text}</span>;
  return (
    <button type="button" className={`${cls} playable`} onClick={onClick} title={title} aria-label={title}>
      {text}
    </button>
  );
}

/** Choose a pile (and maybe an earlier card), then the player to ask. */
export function AskControls({
  view,
  legal,
  me,
  names,
  pile,
  extra,
  onPile,
  onExtra,
  submit,
}: {
  view: BlackViennaView;
  legal: BlackViennaAction[];
  me: number;
  names: string[];
  pile: number | null;
  extra: CardId | null;
  onPile(pile: number | null): void;
  onExtra(extra: CardId | null): void;
  submit(action: BlackViennaAction): void;
}) {
  const targets = names.map((_, t) => t).filter((t) => t !== me);
  const extras = view.specialAllowed ? view.reusable : [];
  return (
    <div className="bv-ask">
      <div className="bv-row">
        <span>Carte :</span>
        {view.piles.map((p, i) =>
          p.top === null ? null : (
            <Letters
              key={i}
              text={view.investigations[p.top]}
              selected={pile === i}
              title={`Prendre ${view.investigations[p.top]}`}
              onClick={() => onPile(i)}
            />
          ),
        )}
      </div>
      {extras.length > 0 && (
        <div className="bv-row">
          <span>Enquête spéciale :</span>
          {extras.map((card) => (
            <Letters
              key={card}
              text={view.investigations[card]}
              selected={extra === card}
              title={`Ajouter ${view.investigations[card]}`}
              onClick={() => onExtra(extra === card ? null : card)}
            />
          ))}
        </div>
      )}
      <div className="pairs-moves">
        {targets.map((t) => {
          const action = findAsk(legal, pile, t, extra);
          return (
            <button key={t} type="button" disabled={!action} onClick={action ? () => submit(action) : undefined}>
              Interroger {names[t]}
            </button>
          );
        })}
      </div>
    </div>
  );
}

/** Pick three letters, then submit the matching legal accusation or guess. */
export function TrioPicker({
  suspects,
  chosen,
  onToggle,
  label,
  action,
  submit,
}: {
  suspects: Suspect[];
  chosen: Suspect[];
  onToggle(s: Suspect): void;
  label: string;
  action(trio: Trio): BlackViennaAction | undefined;
  submit(action: BlackViennaAction): void;
}) {
  const trio = pickTrio(chosen, suspects);
  const move = trio ? action(trio) : undefined;
  return (
    <div className="bv-picker">
      <div className="bv-row">
        {suspects.map((s) => (
          <Letters key={s} text={s} selected={chosen.includes(s)} title={`Choisir ${s}`} onClick={() => onToggle(s)} />
        ))}
      </div>
      <div className="pairs-moves">
        <button type="button" disabled={!move} onClick={move ? () => submit(move) : undefined}>
          {label}
          {trio ? ` ${trio.join("-")}` : ""}
        </button>
      </div>
    </div>
  );
}

export function BlackViennaBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as BlackViennaView;
  const legal = legalActions as BlackViennaAction[];
  // Manual marks stay in this browser, one sheet per seat (several people
  // may share the screen).
  const [marks, setMarks] = useState<Record<number, Record<string, ManualMark>>>({});
  const [pile, setPile] = useState<number | null>(null);
  const [extra, setExtra] = useState<CardId | null>(null);
  const [chosen, setChosen] = useState<Suspect[]>([]);
  const [accusing, setAccusing] = useState(false);
  // A rematch keeps this board mounted: the old notes would mislead.
  const [seen, setSeen] = useState<Progress>({ phase: v.phase, answers: v.answers.length });
  if (seen.phase !== v.phase || seen.answers !== v.answers.length) {
    const next = { phase: v.phase, answers: v.answers.length };
    if (isNewGame(seen, next)) setMarks({});
    setSeen(next);
  }
  const send = (action: BlackViennaAction) => {
    setPile(null);
    setExtra(null);
    setChosen([]);
    setAccusing(false);
    submit(action);
  };
  const toggle = (s: Suspect) =>
    setChosen((c) => (c.includes(s) ? c.filter((x) => x !== s) : c.length < 3 ? [...c, s] : c));
  const mine = me === null ? {} : (marks[me] ?? {});
  const cycle = (key: string) => {
    if (me === null) return;
    setMarks((all) => {
      const next = { ...(all[me] ?? {}) };
      const mark = cycleMark(next[key]);
      if (mark) next[key] = mark;
      else delete next[key];
      return { ...all, [me]: next };
    });
  };
  const asks = legal.filter((a) => "Ask" in a);
  const guessing = v.phase === "Guess" && legal.some((a) => "Guess" in a);
  const sameTrio = (t: Trio) => (x: Trio) => x.join("") === t.join("");

  return (
    <div className="bv-board">
      <div className="bv-top felt">
        <p className="pairs-round">
          Réserve : {v.reserve} · {v.phase === "Over" ? "Partie terminée" : v.turn !== null ? `À ${seatNames[v.turn]}` : "Chacun nomme son trio"}
          {v.last !== null && v.phase !== "Over" ? ` · Dernier enquêteur : ${seatNames[v.last]}` : ""}
        </p>
        <div className="bv-row">
          {v.piles.map((p, i) => (
            <span key={i} className="bv-pile">
              {p.top === null ? <span className="bv-card empty">—</span> : <Letters text={v.investigations[p.top]} />}
              <small>{plural(p.left, "carte")}</small>
            </span>
          ))}
        </div>
        {v.trio && <p className="bv-trio">Black Vienna : {v.trio.join("-")}</p>}
      </div>

      <div className="bv-seats">
        {seatNames.map((name, i) => (
          <section key={i} className={`vt-seat${v.turn === i ? " active" : ""}`}>
            <h3>{name}</h3>
            <p className="hint">
              {plural(v.handSizes[i] ?? 0, "carte")}
              {v.wrong[i] ? " · s'est trompé" : ""}
              {v.phase === "Over" ? ` · ${v.scores[i]} points` : ""}
            </p>
          </section>
        ))}
      </div>

      {me !== null && v.hand.length > 0 && (
        <div className="bv-row">
          <span>Tes cartes :</span>
          {v.hand.map((s) => (
            <Letters key={s} text={s} />
          ))}
        </div>
      )}

      {me !== null && asks.length > 0 && (
        <AskControls
          view={v}
          legal={legal}
          me={me}
          names={seatNames}
          pile={pile}
          extra={extra}
          onPile={setPile}
          onExtra={setExtra}
          submit={send}
        />
      )}
      {me !== null && v.canAccuse && legal.some((a) => "Accuse" in a) && (
        <div className="pairs-moves">
          <button type="button" className="bv-accuse" onClick={() => setAccusing((a) => !a)}>
            Black Vienna !
          </button>
        </div>
      )}
      {me !== null && (accusing || guessing) && (
        <TrioPicker
          suspects={v.suspects}
          chosen={chosen}
          onToggle={toggle}
          label={guessing ? "Mon trio :" : "Accuser"}
          action={(t) =>
            legal.find(
              (a) => ("Accuse" in a && sameTrio(t)(a.Accuse.trio)) || ("Guess" in a && sameTrio(t)(a.Guess.trio)),
            )
          }
          submit={send}
        />
      )}

      <Sheet
        suspects={v.suspects}
        players={v.players}
        names={seatNames}
        notes={v.notes}
        marks={mine}
        onCycle={me === null ? undefined : cycle}
      />

      <ol className="bv-history">
        {v.answers.map((a, i) => (
          <li key={i}>
            {seatNames[a.by]} → {seatNames[a.target]} : {v.investigations[a.card]}
            {a.special ? " (spéciale)" : ""} — {plural(a.count, "jeton")}
          </li>
        ))}
      </ol>
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeEvent(e, seatNames, v.investigations)}</li>
        ))}
      </ul>
    </div>
  );
}
