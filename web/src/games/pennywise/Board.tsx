import { useState } from "react";
import type { BoardProps } from "../registry";
import { Table } from "../pairs/cards";
import type { PennywiseAction, PennywiseView, Stack } from "./types";
import "./pennywise.css";

export function describeTake(take: number[]): string {
  return take.length === 0 ? "rien" : take.map((c) => `${c}¢`).join(" + ");
}

/** The legal plays of `coin`, one per assortment the engine allows to take back. */
export function takesFor(legal: PennywiseAction[], coin: number): PennywiseAction[] {
  return legal.filter((a) => a.Play.coin === coin);
}

function Coins({
  stacks,
  playable = [],
  selected = null,
  onPick,
}: {
  stacks: Stack[];
  playable?: number[];
  selected?: number | null;
  onPick?(cents: number): void;
}) {
  return (
    <div className="pennywise-coins">
      {stacks.map(({ cents, count }) => {
        const canPlay = playable.includes(cents);
        const classes = ["coin", `coin-${cents}`, canPlay ? "playable" : "", selected === cents ? "selected" : ""];
        return (
          <span key={cents} className="pennywise-stack">
            {canPlay && onPick ? (
              <button className={classes.join(" ").trim()} onClick={() => onPick(cents)} aria-label={`Poser ${cents}¢`}>
                {cents}
              </button>
            ) : (
              <span className={classes.join(" ").trim()}>{cents}</span>
            )}
            {count > 1 && <span className="count">×{count}</span>}
          </span>
        );
      })}
    </div>
  );
}

/** What a picked coin belongs to: the position (last move) and my choices. */
export function decisionKey(view: PennywiseView, legal: PennywiseAction[]): string {
  return JSON.stringify([view.last, view.current, legal]);
}

/** The assortments a coin may take back, when the engine offers several. */
export function TakeChoices({
  choices,
  onSend,
  onCancel,
}: {
  choices: PennywiseAction[];
  onSend(a: PennywiseAction): void;
  onCancel(): void;
}) {
  return (
    <div className="pennywise-choices">
      {choices.map((a) => (
        <button key={a.Play.take.join("+")} className="primary" onClick={() => onSend(a)}>
          {`Reprendre ${describeTake(a.Play.take)}`}
        </button>
      ))}
      <button onClick={onCancel}>Annuler</button>
    </div>
  );
}

export function PennywiseBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as PennywiseView;
  const legal = legalActions as PennywiseAction[];
  // A picked coin is kept only for the decision it was made for (a timeout
  // or any move clears it).
  const key = decisionKey(v, legal);
  const [chosen, setChosen] = useState<{ key: string; coin: number | null }>({ key, coin: null });
  const picked = chosen.key === key ? chosen.coin : null;
  const setPicked = (coin: number | null) => setChosen({ key, coin });
  const playable = [...new Set(legal.map((a) => a.Play.coin))];
  const choices = picked === null ? [] : takesFor(legal, picked);

  const send = (a: PennywiseAction) => {
    setPicked(null);
    submit(a);
  };
  const pick = (coin: number) => {
    const options = takesFor(legal, coin);
    if (options.length === 1) send(options[0]);
    else setPicked(coin);
  };

  const seats = v.purses.map((stacks, i) => (
    <section key={i} className={`pairs-seat${v.current === i ? " active" : ""}${v.ruined === i ? " lost" : ""}`}>
      <h3>
        <span>{seatNames[i]}</span>
        <span>{v.totals[i]}¢</span>
      </h3>
      {stacks.length === 0 ? (
        <p className="hint">Ruiné</p>
      ) : (
        <Coins
          stacks={stacks}
          playable={i === me ? playable : []}
          selected={i === me ? picked : null}
          onPick={i === me ? pick : undefined}
        />
      )}
    </section>
  ));

  const ranking =
    v.ranks &&
    v.ranks
      .map((rank, seat) => ({ rank, seat }))
      .sort((a, b) => a.rank - b.rank || a.seat - b.seat);

  const center = (
    <>
      <p className="pennywise-pot">Pot : {v.potTotal}¢</p>
      <Coins stacks={v.pot} />
      {v.last && (
        <p className="hint">
          {`${seatNames[v.last.seat]} a posé ${v.last.coin}¢ et repris ${describeTake(v.last.take)}`}
        </p>
      )}
      {picked !== null && choices.length > 1 && (
        <TakeChoices choices={choices} onSend={send} onCancel={() => setPicked(null)} />
      )}
      {me !== null && v.current === me && picked === null && (
        <p className="hint">Pose une pièce : tu reprendras le plus de monnaie possible en dessous.</p>
      )}
      {v.ruined !== null && ranking && (
        <>
          <p className="status">{seatNames[v.ruined]} est ruiné : fin de la partie.</p>
          <ol className="pennywise-ranking">
            {ranking.map(({ rank, seat }) => (
              <li key={seat}>{`${rank + 1}. ${seatNames[seat]} (${v.totals[seat]}¢)`}</li>
            ))}
          </ol>
        </>
      )}
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
