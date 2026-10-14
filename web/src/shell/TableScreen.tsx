import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { GameMeta, Update } from "../engine/types";
import { clockLabel } from "../session/clock";
import { gameUi } from "../games/registry";
import { HostSession } from "../session/host";
import type { SeatInfo, Session, SessionState } from "../session/session";
import { HostLobby } from "./HostLobby";
import { RulesReminder } from "./RulesReminder";

interface Props {
  session: Session;
  games: GameMeta[];
  onLeave(): void;
}

export function TableScreen({ session, games, onLeave }: Props) {
  const state = useSyncExternalStore(session.subscribe, session.getState);
  const meta = games.find((g) => g.id === state.gameId);
  return (
    <main>
      <div className="topbar">
        <button className="ghost" onClick={onLeave}>
          ← Menu
        </button>
        <h2>{meta?.name ?? "…"}</h2>
        <span className="spacer" />
        {state.latencyMs !== null && <span className="pill mono">{state.latencyMs} ms</span>}
      </div>
      {state.error && <p className="error">{state.error}</p>}
      {state.status && <p className="status">{state.status}</p>}
      {state.notice && <p className="notice">{state.notice}</p>}
      {state.phase === "lobby" &&
        (session instanceof HostSession ? <HostLobby session={session} state={state} /> : <GuestLobby state={state} />)}
      {state.phase === "playing" && <Playing session={session} state={state} />}
    </main>
  );
}

function Playing({ session, state }: { session: Session; state: SessionState }) {
  const update = state.update;
  // Clocks count down from when the update arrived.
  const receivedAt = useMemo(() => performance.now(), [update]);
  const ui = state.gameId ? gameUi(state.gameId) : undefined;
  if (state.gameId && !ui) return <p className="error">Jeu inconnu : mets la page à jour.</p>;
  const Board = ui?.Board;
  return (
    <>
      <SeatList seats={state.seats} update={update} me={state.me} receivedAt={receivedAt} />
      <RulesReminder rules={ui?.rules} />
      {update && Board && (
        <Board
          view={update.view}
          legalActions={update.legalActions}
          pending={update.pending}
          me={state.me}
          seatNames={state.seats.map((s) => s.name)}
          submit={(action) => {
            if (state.me !== null) session.submit(state.me, action);
          }}
        />
      )}
      {update?.outcome && (
        <div className="end">
          <strong>Partie terminée.</strong>{" "}
          {session.canRestart && (
            <button className="primary" onClick={() => session.restart()}>
              Revanche
            </button>
          )}
        </div>
      )}
    </>
  );
}

function rankLabel(rank: number): string {
  return rank === 0 ? "1er" : `${rank + 1}ᵉ`;
}

/** Re-renders every `ms` while `active`. */
function useNow(ms: number, active: boolean): number {
  const [now, setNow] = useState(() => performance.now());
  useEffect(() => {
    if (!active) return;
    const id = setInterval(() => setNow(performance.now()), ms);
    return () => clearInterval(id);
  }, [ms, active]);
  return now;
}


function SeatList({
  seats,
  update,
  me,
  receivedAt,
}: {
  seats: SeatInfo[];
  update: Update | null;
  me: number | null;
  receivedAt: number;
}) {
  const now = useNow(250, (update?.clocks.length ?? 0) > 0);
  return (
    <ol className="seatbar">
      {seats.map((seat, i) => {
        const pending = update?.pending.includes(i) ?? false;
        const rank = update?.outcome?.[i];
        const clock = update?.clocks.find((x) => x.seat === i);
        return (
          <li key={i} className={pending ? "active" : undefined}>
            <strong>
              {seat.name}
              {i === me ? " (toi)" : ""}
            </strong>
            {!seat.connected && seat.kind === "remote" && <span className="tag">déconnecté</span>}
            {pending && <span className="tag">{seat.kind === "ai" ? "réfléchit…" : "à jouer"}</span>}
            {clock && <span className="clock">{clockLabel(clock, now - receivedAt)}</span>}
            {rank !== undefined && <span className="tag">{rankLabel(rank)}</span>}
          </li>
        );
      })}
    </ol>
  );
}

function GuestLobby({ state }: { state: SessionState }) {
  return (
    <section className="panel">
      <h3>Sièges</h3>
      <ol className="seats">
        {state.seats.map((seat, i) => (
          <li key={i}>
            {seat.name}
            {i === state.me ? " (toi)" : ""}
          </li>
        ))}
      </ol>
      <p className="hint">L'hôte lance la partie.</p>
    </section>
  );
}
